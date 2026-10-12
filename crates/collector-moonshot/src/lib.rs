//! Moonshot collector: the platform account balance from the documented balance endpoint.
//!
//! # Source and confidence (official)
//!
//! `GET https://api.moonshot.ai/v1/users/me/balance` with the user's API key as Bearer, documented at
//! <https://platform.kimi.ai/docs/api/balance> (the old `platform.moonshot.ai` address redirects
//! there). The answer is money in US dollars, not a quota: it has no percentage and no reset time,
//! so the view carries facts only. Only the `.ai` host is used: keys of the other platform
//! (`platform.kimi.com`) are independent and a wrong one answers 401.
//!
//! # Behaviour
//!
//! - The gate runs before any I/O: disabled, or no key, is a state without a socket.
//! - One GET per call; app-core's `timing::cadence` keeps Moonshot at 300 s or slower.
//! - The key is only ever sent to `https://api.moonshot.ai`: the host is a constant and release
//!   builds have no way to point the client elsewhere.
//! - `balance` (available), `voucher` and `cash` are reported as they come; `cash` may be negative.

mod parse;
mod remote;
#[cfg(test)]
mod tests;

use usage_model::settings::MoonshotSettings;
use usage_model::{Ctx, HarnessId, HarnessView, Source, ViewState};
use usage_support::Secret;

const ID: HarnessId = HarnessId::Moonshot;

/// This collector raises no notices; kept so the shell's notice-table test can iterate every crate.
pub const NOTICE_CODES: &[&str] = &[];

pub async fn collect(
    settings: &MoonshotSettings,
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

fn plan<'a>(settings: &MoonshotSettings, api_key: Option<&'a Secret>) -> Plan<'a> {
    if !settings.enabled {
        return Plan::Done(Box::new(HarnessView::disabled(ID)));
    }
    match api_key.filter(|secret| !secret.expose().trim().is_empty()) {
        Some(key) => Plan::Read(key),
        None => Plan::Done(Box::new(HarnessView::needs_setup(ID))),
    }
}

async fn collect_with(
    settings: &MoonshotSettings,
    api_key: Option<&Secret>,
    ctx: &Ctx,
    endpoint: &remote::Endpoint,
) -> HarnessView {
    let key = match plan(settings, api_key) {
        Plan::Done(view) => return *view,
        Plan::Read(key) => key,
    };
    match remote::fetch(endpoint, key).await {
        Ok(facts) => {
            let mut view = HarnessView::with_state(ID, ViewState::Ready);
            view.source = Some(Source { id: "users_balance".to_owned(), official: true });
            view.read_at = Some(ctx.now_secs);
            view.facts = facts;
            view
        }
        Err(error) => HarnessView::error(ID, error),
    }
}
