//! MiniMax collector: the Token Plan quota buckets, one pool per model.
//!
//! # Source and confidence (UNOFFICIAL: the route has no published schema)
//!
//! `GET https://api.minimax.io/v1/token_plan/remains` with the Token Plan key as Bearer. The route
//! is used by MiniMax's own tooling (`mmx quota`) and community clients, but MiniMax's public
//! documentation (opened 2026-10-12) only says usage is "shown in the console" and does not
//! publish this route or its fields, so the view is badged unofficial and the parse is tolerant:
//! a document of another shape is `endpoint_changed`, never guessed. Field names and meanings
//! (including that the `*_usage_count` fields hold what is *left*) come from community reports
//! and must be checked against the console; see `docs/data-sources.md`.
//!
//! # Behaviour
//!
//! - The gate runs before any I/O: disabled, or no key, is a state without a socket.
//! - One GET per call; app-core's `timing::cadence` keeps MiniMax at 300 s or slower.
//! - The key is only ever sent to `https://api.minimax.io` (the global host; never the China one):
//!   the host is a constant and release builds have no way to point the client elsewhere.
//! - Per model bucket: an interval window (its length read from the start and end times when
//!   given, never assumed) and a weekly window. A total of zero means the plan has no such quota:
//!   no window, not 0 %.

mod parse;
mod remote;
#[cfg(test)]
mod tests;

use usage_model::settings::MinimaxSettings;
use usage_model::{Ctx, HarnessId, HarnessView, Source, ViewState};
use usage_support::Secret;

const ID: HarnessId = HarnessId::Minimax;

/// This collector raises no notices; kept so the shell's notice-table test can iterate every crate.
pub const NOTICE_CODES: &[&str] = &[];

pub async fn collect(
    settings: &MinimaxSettings,
    api_key: Option<&Secret>,
    ctx: &Ctx,
) -> HarnessView {
    collect_with(settings, api_key, ctx, &remote::Endpoint::production()).await
}

/// What one poll does, decided from settings and the stored key alone, before any socket opens.
enum Plan<'a> {
    /// Boxed: a view is large and the other variant is one word.
    Done(Box<HarnessView>),
    Read(&'a Secret),
}

fn plan<'a>(settings: &MinimaxSettings, api_key: Option<&'a Secret>) -> Plan<'a> {
    if !settings.enabled {
        return Plan::Done(Box::new(HarnessView::disabled(ID)));
    }
    match api_key.filter(|secret| !secret.expose().trim().is_empty()) {
        Some(key) => Plan::Read(key),
        None => Plan::Done(Box::new(HarnessView::needs_setup(ID))),
    }
}

async fn collect_with(
    settings: &MinimaxSettings,
    api_key: Option<&Secret>,
    ctx: &Ctx,
    endpoint: &remote::Endpoint,
) -> HarnessView {
    let key = match plan(settings, api_key) {
        Plan::Done(view) => return *view,
        Plan::Read(key) => key,
    };
    match remote::fetch(endpoint, key, ctx.now_secs).await {
        Ok(windows) => {
            let mut view = HarnessView::with_state(ID, ViewState::Ready);
            view.source = Some(Source { id: "token_plan_remains".to_owned(), official: false });
            view.read_at = Some(ctx.now_secs);
            view.windows = windows;
            view
        }
        Err(error) => HarnessView::error(ID, error),
    }
}
