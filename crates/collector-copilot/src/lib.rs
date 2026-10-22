//! GitHub Copilot collector: the monthly premium-request quota from `copilot_internal/user`.
//!
//! # Source and confidence (unofficial)
//!
//! `GET https://api.github.com/copilot_internal/user` with a GitHub OAuth token as
//! `Authorization: token <t>` and the header set the Copilot editor extensions send. The route is
//! internal and undocumented; the shape here comes from the CodexBar project's docs and community
//! monitors, never from a call this project made with a real token (docs/data-sources.md), so a
//! change surfaces as `EndpointChanged`, not as made-up numbers.
//!
//! # Token and gates
//!
//! 1. Disabled: a disabled view, before any I/O.
//! 2. A pasted token (`SecretKind::Copilot`): saving it is the consent; nothing local is read.
//! 3. Otherwise, with `local_consent`: the `github.com` entry's `oauth_token` of the extension's
//!    `apps.json`, then `hosts.json` (`<home>\AppData\Local\github-copilot` on Windows,
//!    `~/.config/github-copilot` elsewhere). No such token is `NotSignedIn` with a fixed `reason`.
//! 4. No token and no consent: `NeedsSetup` (as OpenCode does: the UI offers both the paste field
//!    and the consent switch there). `NeedsConsent` is never returned, because the collector cannot
//!    know that the local file would be the user's only way in.
//!
//! # Windows
//!
//! `premium_requests` (monthly; used share from `percent_remaining`, else from entitlement and
//! remaining). A finite `chat` or `completions` quota is a window of its own. A quota with
//! `unlimited: true` is a fact (`*_unlimited`), never a window and never zero. The reset is the
//! response's date when it is usable; otherwise GitHub's documented cadence (the 1st at 00:00:00
//! UTC) is derived, flagged with `meta.reset_derived = "true"` and the `copilot_reset_derived`
//! notice. No window states a length, so no pace alarm can fire on it.
//!
//! The token is only ever sent to `https://api.github.com`: the host is a constant and release
//! builds have no way to point the client elsewhere. 401/403 are `Unauthorized` with
//! `ambiguous = true` (a fine-grained token or an account without Copilot is refused the same way).

mod calendar;
mod local;
mod notice;
mod parse;
mod remote;
#[cfg(test)]
mod tests;

use usage_model::settings::CopilotSettings;
use usage_model::{
    Ctx, ErrorCode, HarnessError, HarnessId, HarnessView, Source, UsageWindow, ViewState,
};
use usage_support::Secret;

const ID: HarnessId = HarnessId::Copilot;

/// Every notice code this crate can emit; the UI catalog (docs/ipc.md) must know each one.
pub const NOTICE_CODES: &[&str] = &[notice::RESET_DERIVED];

pub async fn collect(settings: &CopilotSettings, token: Option<&Secret>, ctx: &Ctx) -> HarnessView {
    collect_with(settings, token, ctx, &remote::Endpoint::production()).await
}

/// What one poll does, decided from settings and the stored token alone, before any I/O.
enum Plan {
    /// Boxed: a view is large and the other variants are small.
    Done(Box<HarnessView>),
    Pasted(Secret),
    Local,
}

fn plan(settings: &CopilotSettings, token: Option<&Secret>) -> Plan {
    if !settings.enabled {
        return Plan::Done(Box::new(HarnessView::disabled(ID)));
    }
    // Surrounding whitespace from a paste is dropped; a blank paste is no paste.
    let pasted = token.map(|secret| secret.expose().trim()).filter(|text| !text.is_empty());
    match pasted {
        Some(text) => Plan::Pasted(Secret::new(text)),
        None if settings.local_consent => Plan::Local,
        None => Plan::Done(Box::new(HarnessView::needs_setup(ID))),
    }
}

async fn collect_with(
    settings: &CopilotSettings,
    token: Option<&Secret>,
    ctx: &Ctx,
    endpoint: &remote::Endpoint,
) -> HarnessView {
    let (token, origin) = match plan(settings, token) {
        Plan::Done(view) => return *view,
        Plan::Pasted(token) => (token, "pasted"),
        Plan::Local => match local::find_token(ctx.home_dir.clone()).await {
            Ok(token) => (token, "local"),
            Err(reason) => {
                let error = HarnessError::new(ErrorCode::NotSignedIn, false)
                    .with_param("reason", reason.as_str());
                return HarnessView::error(ID, error);
            }
        },
    };
    match remote::fetch(endpoint, &token, ctx.now_secs).await {
        Ok(parsed) => ready(parsed, origin, ctx.now_secs),
        Err(error) => HarnessView::error(ID, error),
    }
}

fn ready(parsed: parse::Parsed, origin: &str, now_secs: i64) -> HarnessView {
    let mut view = HarnessView::with_state(ID, ViewState::Ready);
    view.source = Some(Source { id: "copilot_internal_user".to_owned(), official: false });
    view.read_at = Some(now_secs);
    view.stale = all_reset(&parsed.windows, now_secs);
    view.plan = parsed.plan;
    view.windows = parsed.windows;
    view.facts = parsed.facts;
    view.meta.insert("token_source".to_owned(), origin.to_owned());
    if parsed.reset_derived {
        view.meta.insert("reset_derived".to_owned(), "true".to_owned());
        view.notices.push(notice::reset_derived());
    }
    view
}

/// Every window has a known reset time that already passed: the numbers are history.
fn all_reset(windows: &[UsageWindow], now_secs: i64) -> bool {
    !windows.is_empty()
        && windows.iter().all(|window| window.resets_at.is_some_and(|reset| reset <= now_secs))
}
