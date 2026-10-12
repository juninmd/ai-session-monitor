//! Ollama Cloud collector: the remaining included and purchased credits.
//!
//! # Source and confidence (official)
//!
//! `GET https://ollama.com/api/balance` with an API key as Bearer, documented at
//! <https://docs.ollama.com/api/balance> (opened 2026-10-12). The key is the one minted at
//! <https://ollama.com/settings/keys>; the CLI's own signing key is never read. The shape varies
//! by plan: current plans report `included.{balance_usd, allowance_usd, period}`, legacy plans
//! `included.{session, weekly}.{remaining_percent, resets_at}`; `purchased.balance_usd` is
//! separate. The service allows 10 requests a minute per user across all keys.
//!
//! # Behaviour
//!
//! - The gate runs before any I/O: disabled, or no key, is a state without a socket.
//! - One GET per call; app-core's `timing::cadence` keeps Ollama at 300 s or slower.
//! - The key is only ever sent to `https://ollama.com`: the host is a constant and release builds
//!   have no way to point the client elsewhere.
//! - A percentage is shown only when the document states one (or an allowance to divide by);
//!   a missing figure stays unknown, a purchased balance is a fact and never a percentage.

mod parse;
mod remote;
#[cfg(test)]
mod tests;

use usage_model::settings::OllamaSettings;
use usage_model::{Ctx, HarnessId, HarnessView, Source, ViewState};
use usage_support::Secret;

const ID: HarnessId = HarnessId::Ollama;

/// This collector raises no notices; kept so the shell's notice-table test can iterate every crate.
pub const NOTICE_CODES: &[&str] = &[];

pub async fn collect(
    settings: &OllamaSettings,
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

fn plan<'a>(settings: &OllamaSettings, api_key: Option<&'a Secret>) -> Plan<'a> {
    if !settings.enabled {
        return Plan::Done(Box::new(HarnessView::disabled(ID)));
    }
    match api_key.filter(|secret| !secret.expose().trim().is_empty()) {
        Some(key) => Plan::Read(key),
        None => Plan::Done(Box::new(HarnessView::needs_setup(ID))),
    }
}

async fn collect_with(
    settings: &OllamaSettings,
    api_key: Option<&Secret>,
    ctx: &Ctx,
    endpoint: &remote::Endpoint,
) -> HarnessView {
    let key = match plan(settings, api_key) {
        Plan::Done(view) => return *view,
        Plan::Read(key) => key,
    };
    match remote::fetch(endpoint, key).await {
        Ok(balance) => {
            let mut view = HarnessView::with_state(ID, ViewState::Ready);
            view.source = Some(Source { id: "balance".to_owned(), official: true });
            view.read_at = Some(ctx.now_secs);
            view.windows = balance.windows;
            view.facts = balance.facts;
            view
        }
        Err(error) => HarnessView::error(ID, error),
    }
}
