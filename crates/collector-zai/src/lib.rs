//! z.ai (GLM Coding Plan) collector: the plan's quota windows from the monitor endpoint.
//!
//! # Source and confidence (unofficial)
//!
//! `GET https://api.z.ai/api/monitor/usage/quota/limit` with the user's API key. Not in the public
//! API reference, but z.ai's own coding agent (zai-org/ZCode, `docs/coding-plan-quota-remaining-api.md`,
//! 2026-06-29) calls it, and two community monitors read the same body. The key goes in the
//! `Authorization` header **without** a `Bearer` prefix, as that document states. The shapes here
//! come from those sources, never from a call this project made with a real key
//! (docs/data-sources.md), so a change surfaces as `EndpointChanged`, not as made-up numbers.
//!
//! # Behaviour
//!
//! - The gate runs before any I/O: disabled, or no key, is a state without a socket.
//! - One GET per call; app-core's `timing::cadence` keeps z.ai at 300 s or slower.
//! - The key is only ever sent to `https://api.z.ai`: the host is a constant and release builds
//!   have no way to point the client elsewhere. The mainland-China host is not supported.
//! - The service reports failures inside a 200 response (`success: false` and a `code`); those are
//!   mapped like the matching HTTP status. 401/403 are `Unauthorized` with `ambiguous = true`.

mod parse;
mod remote;
#[cfg(test)]
mod tests;

use usage_model::settings::ZaiSettings;
use usage_model::{Ctx, HarnessId, HarnessView, Source, UsageWindow, ViewState};
use usage_support::Secret;

const ID: HarnessId = HarnessId::Zai;

/// This collector raises no notices; kept so the shell's notice-table test can iterate every crate.
pub const NOTICE_CODES: &[&str] = &[];

pub async fn collect(settings: &ZaiSettings, api_key: Option<&Secret>, ctx: &Ctx) -> HarnessView {
    collect_with(settings, api_key, ctx, &remote::Endpoint::production()).await
}

/// What one poll does, decided from settings and the stored key alone, before any socket opens.
enum Plan<'a> {
    /// Boxed: a view is large and the other variant is one word.
    Done(Box<HarnessView>),
    Read(&'a Secret),
}

fn plan<'a>(settings: &ZaiSettings, api_key: Option<&'a Secret>) -> Plan<'a> {
    if !settings.enabled {
        return Plan::Done(Box::new(HarnessView::disabled(ID)));
    }
    match api_key.filter(|secret| !secret.expose().trim().is_empty()) {
        Some(key) => Plan::Read(key),
        None => Plan::Done(Box::new(HarnessView::needs_setup(ID))),
    }
}

async fn collect_with(
    settings: &ZaiSettings,
    api_key: Option<&Secret>,
    ctx: &Ctx,
    endpoint: &remote::Endpoint,
) -> HarnessView {
    let key = match plan(settings, api_key) {
        Plan::Done(view) => return *view,
        Plan::Read(key) => key,
    };
    match remote::fetch(endpoint, key, ctx.now_secs).await {
        Ok(parsed) => ready(parsed, ctx.now_secs),
        Err(error) => HarnessView::error(ID, error),
    }
}

fn ready(parsed: parse::Parsed, now_secs: i64) -> HarnessView {
    let mut view = HarnessView::with_state(ID, ViewState::Ready);
    view.source = Some(Source { id: "quota_limit".to_owned(), official: false });
    view.read_at = Some(now_secs);
    view.stale = all_reset(&parsed.windows, now_secs);
    view.plan = parsed.plan;
    view.windows = parsed.windows;
    view
}

/// Every window has a known reset time that already passed: the numbers are history. One window
/// without a reset time keeps the view fresh, since it cannot be shown to have expired.
fn all_reset(windows: &[UsageWindow], now_secs: i64) -> bool {
    !windows.is_empty()
        && windows.iter().all(|window| window.resets_at.is_some_and(|reset| reset <= now_secs))
}
