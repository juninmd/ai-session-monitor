//! Codex collector.
//!
//! Source `app_server` (official): spawn `codex app-server`, talk JSON-RPC over its stdio
//! (`initialize`, `initialized`, `account/read`, `account/rateLimits/read`) inside a hard budget,
//! then kill the whole process tree. Source `rollout` (unofficial, "last seen"): the newest
//! `token_count` event that carries `rate_limits` in `~/.codex/sessions/**/rollout-*.jsonl`,
//! used only when the app-server cannot be reached.
//!
//! Confidence. The protocol is documented (developers.openai.com/codex/app-server) although the
//! command is labelled experimental. Shapes here come from that page, from the JSON Schema that
//! `codex app-server generate-json-schema` emits for codex-cli 0.162.1 and from the openai/codex
//! sources; the handshake, `account/read` and the signed-out error were also observed against an
//! empty throwaway profile. A signed-in `account/rateLimits/read` answer was never observed by this
//! crate (no real account is touched), so that shape rests on the schema and the docs. The rollout
//! layout is an internal format read from the sources, never from a real session file.

mod account;
mod app_server;
mod binary;
mod failure;
mod limits;
mod plan;
mod rollout;
mod rpc;
mod view;

use std::time::Duration;

use usage_model::settings::CodexSettings;
use usage_model::{Ctx, HarnessId, HarnessView};

pub use app_server::APP_SERVER_BUDGET;

/// Every notice code this crate can emit (catalog: `docs/ipc.md`).
pub const NOTICE_CODES: &[&str] = &[view::ROLLOUT_FALLBACK, view::NO_LIMITS];

const ID: HarnessId = HarnessId::Codex;

/// One read of Codex. Never panics and never leaves a child process behind.
pub async fn collect(settings: &CodexSettings, ctx: &Ctx) -> HarnessView {
    collect_with_budget(settings, ctx, APP_SERVER_BUDGET).await
}

/// [`collect`] with another total budget for the app-server conversation (tests, diagnostics).
pub async fn collect_with_budget(
    settings: &CodexSettings,
    ctx: &Ctx,
    budget: Duration,
) -> HarnessView {
    // Gates first: nothing below may run (no file, process or network access) before them.
    if !settings.enabled {
        return HarnessView::disabled(ID);
    }
    if !settings.consent {
        return HarnessView::needs_consent(ID);
    }
    match app_server::read(settings, budget).await {
        Ok(reading) => view::from_app_server(reading, ctx),
        Err(error) if failure::allows_fallback(error.code) => {
            match rollout::latest(&ctx.home_dir).await {
                Some(found) => view::from_rollout(found, error.code, ctx),
                None => HarnessView::error(ID, error),
            }
        }
        Err(error) => HarnessView::error(ID, error),
    }
}
