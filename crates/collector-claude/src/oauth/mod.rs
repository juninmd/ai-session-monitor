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

#[cfg(test)]
mod fake_server;
#[cfg(test)]
mod tests;

use std::time::Duration;

use serde::{Deserialize, Serialize};
use usage_model::settings::ClaudeSettings;
use usage_model::{Ctx, ErrorCode, HarnessError, HarnessView, UsageWindow};

use crate::{notice, paths::Paths};

const PRODUCTION_BASE: &str = "https://api.anthropic.com";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
/// `view.meta` key and value marking that some windows come from the unofficial source.
const META_KEY: &str = "extra_source";
const META_VALUE: &str = "oauth_usage";

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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct Extras {
    pub plan: Option<String>,
    pub windows: Vec<UsageWindow>,
}

/// Adds the OAuth windows and plan to a usable statusLine view, or notes why it could not.
pub(crate) async fn add_extras(
    view: &mut HarnessView,
    settings: &ClaudeSettings,
    ctx: &Ctx,
    endpoint: &Endpoint,
) {
    let Some(paths) = Paths::resolve(settings, ctx) else { return };
    match extras(&paths, ctx.now_secs, endpoint).await {
        Ok(extras) => {
            if view.plan.is_none() {
                view.plan = extras.plan;
            }
            if !extras.windows.is_empty() {
                view.windows.extend(extras.windows);
                view.meta.insert(META_KEY.to_owned(), META_VALUE.to_owned());
            }
        }
        Err(error) => view.notices.push(notice::oauth_partial(error.code)),
    }
}

async fn extras(paths: &Paths, now: i64, endpoint: &Endpoint) -> Result<Extras, HarnessError> {
    let cache_path = paths.oauth_cache.clone();
    let hit = blocking(move || cache::fresh(&cache_path, now)).await.flatten();
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
    let outcome = request(endpoint, &credentials)
        .await
        .map(|windows| Extras { plan: credentials.plan.clone(), windows: usage::bounded(windows) });
    let (cache_path, stored) = (paths.oauth_cache.clone(), outcome.clone());
    blocking(move || cache::store(&cache_path, now, &stored)).await;
    outcome
}

async fn request(
    endpoint: &Endpoint,
    credentials: &credentials::Credentials,
) -> Result<Vec<UsageWindow>, HarnessError> {
    let body = usage::fetch(endpoint, &credentials.token).await?;
    usage::parse(&body)
}

/// Runs blocking file work off the async runtime; `None` if the task was lost.
async fn blocking<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> Option<T> {
    tokio::task::spawn_blocking(work).await.ok()
}
