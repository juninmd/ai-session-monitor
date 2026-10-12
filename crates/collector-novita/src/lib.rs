//! Novita collector: the account balance from the documented billing endpoint.
//!
//! # Source and confidence (official)
//!
//! `GET https://api.novita.ai/openapi/v1/billing/balance/detail` with the user's API key as Bearer,
//! documented at <https://docs.novita.ai/api-reference/basic-get-user-balance-detail>. Every amount
//! is a decimal string in units of 1/10000 US dollar (`"10000"` is $1.00). The answer is money, not
//! a quota: it has no percentage and no reset time, so the view carries facts only.
//!
//! # Behaviour
//!
//! - The gate runs before any I/O: disabled, or no key, is a state without a socket.
//! - One GET per call; app-core's `timing::cadence` keeps Novita at 300 s or slower.
//! - The key is only ever sent to `https://api.novita.ai`: the host is a constant and release
//!   builds have no way to point the client elsewhere.
//! - `balance` and `cash` may be negative (money owed); the other figures are never negative.

mod parse;
mod remote;
#[cfg(test)]
mod tests;

use usage_model::settings::NovitaSettings;
use usage_model::{Ctx, HarnessId, HarnessView, Source, ViewState};
use usage_support::Secret;

const ID: HarnessId = HarnessId::Novita;

/// This collector raises no notices; kept so the shell's notice-table test can iterate every crate.
pub const NOTICE_CODES: &[&str] = &[];

pub async fn collect(
    settings: &NovitaSettings,
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

fn plan<'a>(settings: &NovitaSettings, api_key: Option<&'a Secret>) -> Plan<'a> {
    if !settings.enabled {
        return Plan::Done(Box::new(HarnessView::disabled(ID)));
    }
    match api_key.filter(|secret| !secret.expose().trim().is_empty()) {
        Some(key) => Plan::Read(key),
        None => Plan::Done(Box::new(HarnessView::needs_setup(ID))),
    }
}

async fn collect_with(
    settings: &NovitaSettings,
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
            view.source = Some(Source { id: "balance_detail".to_owned(), official: true });
            view.read_at = Some(ctx.now_secs);
            view.facts = facts;
            view
        }
        Err(error) => HarnessView::error(ID, error),
    }
}
