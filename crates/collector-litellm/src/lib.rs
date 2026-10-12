//! LiteLLM collector: how much of a virtual key's, user's or team's budget is left.
//!
//! Source `key_info`, `official = true`: the documented management API of a LiteLLM proxy, read
//! with the user's own virtual key. Scope `key` calls `GET {base}/key/info` (no `key` parameter:
//! the caller identifies itself), scope `user` calls `GET {base}/user/info?user_id=`, scope
//! `team` calls `GET {base}/team/info?team_id=&key_limit=1`.
//!
//! Confidence: single origin. The shapes were read from the proxy source at v1.104.2
//! (`info_key_fn`, `user_info`, `team_info`, `build_model_max_budget_usage`, `duration_parser`)
//! and docs.litellm.ai on 2026-10-11; no live proxy was queried and the test fixtures are
//! fabricated from those shapes, so a proxy that differs shows up as `endpoint_changed`, never as
//! a made-up number.
//!
//! What the numbers mean:
//! - `spend` and `max_budget` are USD for the current budget period. A missing, `null` or zero
//!   `max_budget` is *no cap configured*, not unlimited, and yields no window.
//! - `budget_reset_at` is when the period ends. The proxy sweeps resets about every ten minutes,
//!   so a window can read as reset while its spend is still the old period's; such a view is
//!   `stale`.
//! - `window_minutes` is the nominal length from `budget_duration`; months stay unknown. The
//!   proxy aligns resets to calendar boundaries, so the first window of a new key can be shorter.
//! - TPM, RPM and parallel limits are static caps, not live usage; the proxy exposes no usage.
//!
//! A key with no budget data of its own asks its user, then its team (at most two extra
//! requests, same host and credential); `meta.level` says which record supplied the windows.
//! Per-model caps and concurrent budget windows come from the same record. Token hashes, key
//! names, metadata and member lists are never read, so they cannot reach a view.
//!
//! This crate reads no file except the optional private CA bundle named in the settings.

mod budget;
mod duration;
mod entries;
mod lenient;
mod plan;
mod record;
mod sanitize;
mod transport;
mod view;
mod walk;

use usage_model::settings::LitellmSettings;
use usage_model::{Ctx, HarnessId, HarnessView};
use usage_support::Secret;

use plan::Plan;
use transport::Transport;

/// Every `Notice::code` this collector can emit.
pub const NOTICE_CODES: &[&str] =
    &[view::NO_BUDGET, view::LEVEL_UNREADABLE, view::WINDOWS_TRUNCATED];

const ID: HarnessId = HarnessId::Litellm;

/// One poll. Disabled, unconfigured and invalid-input cases return before any file or network
/// access; after that each request is bounded by the configured timeout and the whole poll by
/// three times that, never more than 25 s.
pub async fn collect(
    settings: &LitellmSettings,
    api_key: Option<&Secret>,
    ctx: &Ctx,
) -> HarnessView {
    if !settings.enabled {
        return HarnessView::disabled(ID);
    }
    let base = settings.base_url.as_deref().map(str::trim).filter(|url| !url.is_empty());
    let secret = api_key.filter(|key| !key.expose().trim().is_empty());
    let (Some(base), Some(secret)) = (base, secret) else {
        return HarnessView::needs_setup(ID);
    };
    match poll(settings, base, secret, ctx.now_secs).await {
        Ok(view) => view,
        Err(error) => HarnessView::error(ID, error),
    }
}

async fn poll(
    settings: &LitellmSettings,
    base: &str,
    secret: &Secret,
    now_secs: i64,
) -> Result<HarnessView, usage_model::HarnessError> {
    let plan = Plan::new(settings, base, secret)?;
    let transport = Transport::open(&plan).await?;
    let resolved = walk::resolve(&transport, &plan).await?;
    Ok(view::ready(&plan, &resolved, now_secs))
}
