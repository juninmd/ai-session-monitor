//! DeepInfra collector: the prepaid balance, the spend limit and the month's spend.
//!
//! # Source and confidence (official routes, one unit unverified)
//!
//! Two GETs with the user's API key as Bearer, both in DeepInfra's API reference:
//!
//! - `https://api.deepinfra.com/payment/checklist` (<https://docs.deepinfra.com/api-reference/billing/get-checklist>):
//!   `stripe_balance` (negative = funds ready to spend, positive = money owed), `recent` (usage
//!   since the last invoice) and `limit` (the spending limit, or `null`). The page names no unit
//!   for these three; they are read as US dollars, like the limit setter, and that is unverified.
//! - `https://api.deepinfra.com/payment/usage?from=current` (<https://docs.deepinfra.com/api-reference/billing/usage>):
//!   `months[].total_cost` in cents. Best effort: its failure only leaves the month spend out.
//!
//! # Behaviour
//!
//! - The gate runs before any I/O: disabled, or no key, is a state without a socket.
//! - The checklist decides the poll; a failure of the usage call never fails it.
//! - A percent window exists only when the account has a positive spending limit.
//! - The key is only ever sent to `https://api.deepinfra.com`: the host is a constant and release
//!   builds have no way to point the client elsewhere.

mod parse;
mod remote;
#[cfg(test)]
mod tests;
mod view;

use usage_model::settings::DeepinfraSettings;
use usage_model::{Ctx, HarnessId, HarnessView};
use usage_support::Secret;

const ID: HarnessId = HarnessId::Deepinfra;

/// This collector raises no notices; kept so the shell's notice-table test can iterate every crate.
pub const NOTICE_CODES: &[&str] = &[];

pub async fn collect(
    settings: &DeepinfraSettings,
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

fn plan<'a>(settings: &DeepinfraSettings, api_key: Option<&'a Secret>) -> Plan<'a> {
    if !settings.enabled {
        return Plan::Done(Box::new(HarnessView::disabled(ID)));
    }
    match api_key.filter(|secret| !secret.expose().trim().is_empty()) {
        Some(key) => Plan::Read(key),
        None => Plan::Done(Box::new(HarnessView::needs_setup(ID))),
    }
}

async fn collect_with(
    settings: &DeepinfraSettings,
    api_key: Option<&Secret>,
    ctx: &Ctx,
    endpoint: &remote::Endpoint,
) -> HarnessView {
    let key = match plan(settings, api_key) {
        Plan::Done(view) => return *view,
        Plan::Read(key) => key,
    };
    match remote::fetch(endpoint, key).await {
        Ok(fetched) => view::ready(&fetched, ctx.now_secs),
        Err(error) => HarnessView::error(ID, error),
    }
}
