//! The optional, unofficial OAuth usage source (off unless `ClaudeSettings::oauth_usage`).
//!
//! It only ever adds to a statusLine view: the per-model weekly windows (Opus, Sonnet, ...) and
//! the plan name. The access token is borrowed from Claude Code's own credentials file, kept in
//! memory inside a [`Secret`](usage_support::Secret), sent to one fixed host over HTTPS and never
//! refreshed, written or logged. A failure becomes a `claude_oauth_partial` notice, not an error.
//!
//! The endpoint is polled at most once per 300 s (see [`cache`]); credential problems are cheap
//! local checks and are reported on every poll so that signing in again shows up at once.

mod cache;
mod credentials;
mod usage;

pub(crate) use usage::{bounded, weekly};

#[cfg(test)]
pub(crate) mod fake_server;
#[cfg(test)]
mod tests;

use std::time::Duration;

use serde::{Deserialize, Serialize};
use usage_model::settings::ClaudeSettings;
use usage_model::status::is_expired;
use usage_model::{Ctx, ErrorCode, HarnessError, HarnessView, UsageWindow};

use crate::{notice, paths::Paths};

const PRODUCTION_BASE: &str = "https://api.anthropic.com";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
/// `view.meta` key and value marking that some windows come from the unofficial source.
const META_KEY: &str = "extra_source";
const META_VALUE: &str = "oauth_usage";
const STALE_PRIMARY_SECS: i64 = 600;

/// Where the usage API lives. Production is a constant; tests point it at a loopback server.
pub(crate) struct Endpoint {
    pub base: String,
    pub allow_loopback_http: bool,
    pub timeout: Duration,
}

impl Endpoint {
    pub(crate) fn production() -> Self {
        Self {
            base: PRODUCTION_BASE.to_owned(),
            allow_loopback_http: false,
            timeout: REQUEST_TIMEOUT,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct Extras {
    pub plan: Option<String>,
    pub windows: Vec<UsageWindow>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub primary: Vec<UsageWindow>,
}

/// Adds the OAuth windows and plan to a usable statusLine view, or notes why it could not.
pub(crate) async fn add_extras(
    view: &mut HarnessView,
    settings: &ClaudeSettings,
    ctx: &Ctx,
    endpoint: &Endpoint,
) {
    let Some(paths) = Paths::resolve(settings, ctx) else { return };
    let min_attempted_at = view
        .windows
        .iter()
        .filter(|w| is_expired(w, ctx.now_secs))
        .filter_map(|w| w.resets_at)
        .max();
    match extras(&paths, ctx.now_secs, min_attempted_at, endpoint).await {
        Ok(extras) => {
            if view.plan.is_none() {
                view.plan = extras.plan;
            }
            if needs_primary_refresh(view, ctx.now_secs)
                && apply_primary(view, &extras.primary, ctx.now_secs)
            {
                view.read_at = Some(ctx.now_secs);
                view.meta.insert(META_KEY.to_owned(), META_VALUE.to_owned());
            }
            if !extras.windows.is_empty() {
                view.windows.extend(extras.windows);
                view.meta.insert(META_KEY.to_owned(), META_VALUE.to_owned());
            }
        }
        Err(error) => view.notices.push(notice::oauth_partial(error.code)),
    }
}

/// True when `five_hour` or `weekly` in `view` has no `resets_at` (for instance after the CLI
/// refreshed an expired statusLine window, since the CLI text carries no epoch).
pub(crate) fn missing_primary_resets(view: &HarnessView) -> bool {
    view.windows
        .iter()
        .any(|w| matches!(w.id.as_str(), "five_hour" | "weekly") && w.resets_at.is_none())
}

/// Quietly fills in missing `resets_at` timestamps and the plan from the OAuth endpoint when
/// credentials are available; never emits a notice or hides what the CLI already added.
pub(crate) async fn fill_resets(
    view: &mut HarnessView,
    settings: &ClaudeSettings,
    ctx: &Ctx,
    endpoint: &Endpoint,
) {
    let Some(paths) = Paths::resolve(settings, ctx) else { return };
    let Ok(extras) = extras(&paths, ctx.now_secs, None, endpoint).await else { return };
    if view.plan.is_none() {
        view.plan = extras.plan;
    }
    for fresh in
        extras.primary.into_iter().chain(extras.windows).filter(|w| !is_expired(w, ctx.now_secs))
    {
        if let Some(existing) = view.windows.iter_mut().find(|w| w.id == fresh.id)
            && existing.resets_at.is_none()
        {
            existing.resets_at = fresh.resets_at;
        }
    }
}

fn needs_primary_refresh(view: &HarnessView, now: i64) -> bool {
    view.source.is_some()
        && (view.windows.iter().any(|w| is_expired(w, now))
            || view.read_at.is_some_and(|t| now.saturating_sub(t) >= STALE_PRIMARY_SECS))
}

fn apply_primary(view: &mut HarnessView, primary: &[UsageWindow], now: i64) -> bool {
    let mut changed = false;
    for fresh in primary.iter().filter(|w| !is_expired(w, now)) {
        if let Some(existing) = view.windows.iter_mut().find(|w| w.id == fresh.id) {
            if fresh.used_pct.is_some() {
                existing.used_pct = fresh.used_pct;
                changed = true;
            }
            if fresh.resets_at.is_some() {
                existing.resets_at = fresh.resets_at;
                changed = true;
            }
        } else {
            let pos = if fresh.id == "five_hour" {
                0
            } else {
                usize::from(view.windows.first().is_some_and(|w| w.id == "five_hour"))
            };
            view.windows.insert(pos, fresh.clone());
            changed = true;
        }
    }
    changed
}

async fn extras(
    paths: &Paths,
    now: i64,
    min_attempted_at: Option<i64>,
    endpoint: &Endpoint,
) -> Result<Extras, HarnessError> {
    let cache_path = paths.oauth_cache.clone();
    let hit =
        blocking(move || cache::fresh_since(&cache_path, now, min_attempted_at)).await.flatten();
    if let Some(outcome) = hit {
        return outcome;
    }
    let credentials_path = paths.credentials.clone();
    let credentials = blocking(move || credentials::load(&credentials_path))
        .await
        .ok_or_else(|| HarnessError::new(ErrorCode::Internal, true))??;
    if credentials.is_expired(now) {
        return Err(HarnessError::new(ErrorCode::OauthExpired, false));
    }
    let outcome = request(endpoint, &credentials).await.map(|(windows, primary)| Extras {
        plan: credentials.plan.clone(),
        windows: usage::bounded(windows),
        primary: usage::bounded(primary),
    });
    let (cache_path, stored) = (paths.oauth_cache.clone(), outcome.clone());
    blocking(move || cache::store(&cache_path, now, &stored)).await;
    outcome
}

async fn request(
    endpoint: &Endpoint,
    credentials: &credentials::Credentials,
) -> Result<(Vec<UsageWindow>, Vec<UsageWindow>), HarnessError> {
    let body = usage::fetch(endpoint, &credentials.token).await?;
    let windows = usage::parse(&body)?;
    let primary = usage::parse_primary(&body);
    Ok((windows, primary))
}

/// Runs blocking file work off the async runtime; `None` if the task was lost.
pub(crate) async fn blocking<T: Send + 'static>(
    work: impl FnOnce() -> T + Send + 'static,
) -> Option<T> {
    tokio::task::spawn_blocking(work).await.ok()
}
