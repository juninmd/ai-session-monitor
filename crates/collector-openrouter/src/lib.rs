//! OpenRouter collector: the API key's spending limit, its usage and the account credits.
//!
//! # Source and confidence (official)
//!
//! Two documented GETs with the user's API key as Bearer:
//!
//! - `https://openrouter.ai/api/v1/key` (<https://openrouter.ai/docs/api/reference/limits>): the
//!   key's `limit`, `limit_remaining` and `usage*`. Works with an ordinary key.
//! - `https://openrouter.ai/api/v1/credits` (<https://openrouter.ai/docs/api/api-reference/credits/get-credits>):
//!   `total_credits` and `total_usage`. The reference says it needs a **management** key, so an
//!   ordinary key gets a 403 here; that is expected and only leaves the balance unknown. The
//!   activity endpoint (management key too) is not used.
//!
//! # Behaviour
//!
//! - The gate runs before any I/O: disabled, or no key, is a state without a socket.
//! - The key endpoint decides the poll; a failure of the credits call never fails it.
//! - A percent window exists only when the key has a real limit. A key without one is facts only:
//!   `limit: null` means unlimited, which is not zero and not a percentage.
//! - The key is only ever sent to `https://openrouter.ai`: the host is a constant and release
//!   builds have no way to point the client elsewhere.

mod parse;
mod remote;
#[cfg(test)]
mod tests;
mod view;

use usage_model::settings::OpenrouterSettings;
use usage_model::{Ctx, HarnessId, HarnessView};
use usage_support::Secret;

const ID: HarnessId = HarnessId::Openrouter;

/// This collector raises no notices; kept so the shell's notice-table test can iterate every crate.
pub const NOTICE_CODES: &[&str] = &[];

pub async fn collect(
    settings: &OpenrouterSettings,
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

fn plan<'a>(settings: &OpenrouterSettings, api_key: Option<&'a Secret>) -> Plan<'a> {
    if !settings.enabled {
        return Plan::Done(Box::new(HarnessView::disabled(ID)));
    }
    match api_key.filter(|secret| !secret.expose().trim().is_empty()) {
        Some(key) => Plan::Read(key),
        None => Plan::Done(Box::new(HarnessView::needs_setup(ID))),
    }
}

async fn collect_with(
    settings: &OpenrouterSettings,
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
