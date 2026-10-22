//! Cursor collector: included usage and on-demand spend from the `usage-summary` endpoint.
//!
//! # Source and confidence (unofficial)
//!
//! `GET https://cursor.com/api/usage-summary` with the session cookie the web app itself sends,
//! `WorkosCursorSessionToken=<userId>%3A%3A<jwt>`, built from the access token Cursor keeps in
//! its local `state.vscdb` (`ItemTable`, key `cursorAuth/accessToken`). Undocumented as an API;
//! the route, the cookie and the response shape come from the CodexBar project's docs, never from
//! a call this project made with a real account (docs/data-sources.md), so a change surfaces as
//! `EndpointChanged`, not as made-up numbers.
//!
//! # Behaviour
//!
//! - The gates run before any I/O: disabled, then no consent, are states without a file or a
//!   socket. No usable token (no Cursor install, no sign-in, a malformed token, or one whose JWT
//!   `exp` is under 60 s away) is `NeedsSetup`: open Cursor and sign in.
//! - The database is opened read-only behind an authorizer (see `store`); the token is a
//!   [`Secret`] from the moment it leaves the file and is never logged or serialized.
//! - One GET per call; app-core's `timing::cadence` should keep Cursor at 300 s or slower.
//! - The cookie is only ever sent to `https://cursor.com`: the host is a constant and release
//!   builds have no way to point the client elsewhere.
//! - 401/403 are `Unauthorized` with `ambiguous = true`: a stale or unrelated session is refused
//!   the same way, so the UI never claims the credentials are certainly bad.

mod jwt;
mod parse;
mod path;
mod remote;
mod store;
#[cfg(test)]
mod tests;

use std::path::PathBuf;

use usage_model::settings::CursorSettings;
use usage_model::{
    Ctx, ErrorCode, HarnessError, HarnessId, HarnessView, Source, UsageWindow, ViewState,
};

use store::{Limits, Reason};

const ID: HarnessId = HarnessId::Cursor;

/// This collector raises no notices; kept so the shell's notice-table test can iterate every crate.
pub const NOTICE_CODES: &[&str] = &[];

pub async fn collect(settings: &CursorSettings, ctx: &Ctx) -> HarnessView {
    let db_path = path::default_state_db(&ctx.home_dir);
    collect_with(settings, ctx, db_path, Limits::PRODUCTION, &remote::Endpoint::production()).await
}

/// The gates, decided from settings alone, before any file or socket is touched.
fn gate(settings: &CursorSettings) -> Option<HarnessView> {
    if !settings.enabled {
        Some(HarnessView::disabled(ID))
    } else if !settings.consent {
        Some(HarnessView::needs_consent(ID))
    } else {
        None
    }
}

async fn collect_with(
    settings: &CursorSettings,
    ctx: &Ctx,
    db_path: PathBuf,
    limits: Limits,
    endpoint: &remote::Endpoint,
) -> HarnessView {
    if let Some(view) = gate(settings) {
        return view;
    }
    let token = match store::read_token(db_path, limits).await {
        Ok(Some(token)) => token,
        Ok(None) => return HarnessView::needs_setup(ID),
        Err(reason) => return HarnessView::error(ID, unreadable(reason)),
    };
    let Ok(cookie) = jwt::cookie(&token, ctx.now_secs) else {
        return HarnessView::needs_setup(ID);
    };
    drop(token);
    match remote::fetch(endpoint, &cookie, ctx.now_secs).await {
        Ok(parsed) => ready(parsed, ctx.now_secs),
        Err(error) => HarnessView::error(ID, error),
    }
}

/// A store that exists but cannot be read. `reason` is a fixed identifier, never file content.
fn unreadable(reason: Reason) -> HarnessError {
    let (code, retryable) = match reason {
        Reason::Locked | Reason::Timeout => (ErrorCode::Timeout, true),
        Reason::Schema => (ErrorCode::EndpointChanged, false),
        Reason::Permission | Reason::NotADatabase | Reason::Other => {
            (ErrorCode::SettingsUnreadable, false)
        }
    };
    HarnessError::new(code, retryable).with_param("reason", reason.as_str())
}

fn ready(parsed: parse::Parsed, now_secs: i64) -> HarnessView {
    let mut view = HarnessView::with_state(ID, ViewState::Ready);
    view.source = Some(Source { id: "usage_summary".to_owned(), official: false });
    view.read_at = Some(now_secs);
    view.stale = all_reset(&parsed.windows, now_secs);
    view.plan = parsed.plan;
    view.windows = parsed.windows;
    view.facts = parsed.facts;
    view
}

/// Every window has a known reset time that already passed: the numbers are history. One window
/// without a reset time keeps the view fresh, since it cannot be shown to have expired.
fn all_reset(windows: &[UsageWindow], now_secs: i64) -> bool {
    !windows.is_empty()
        && windows.iter().all(|window| window.resets_at.is_some_and(|reset| reset <= now_secs))
}
