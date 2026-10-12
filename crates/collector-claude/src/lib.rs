//! Claude Code collector. Slice `claude`: statusLine shim data (official), settings.json
//! installer, optional OAuth usage (opt-in, unofficial).
//!
//! # Where the numbers come from, and how far to trust them
//!
//! - **`statusline` (official).** Claude Code pipes a JSON document to its `statusLine` command;
//!   since v2.1.80 it carries `rate_limits.five_hour` and `.seven_day` (`used_percentage` 0..=100,
//!   `resets_at` epoch seconds), and behind a Claude apps gateway `rate_limits.spend_limit`. The
//!   field names, units, run triggers (after assistant messages, `/compact`, mode changes, at a
//!   window's reset, optional `refreshInterval`; 300 ms debounce, in-flight runs cancelled) and the
//!   Windows shells (Git Bash, else PowerShell) were read from
//!   <https://code.claude.com/docs/en/statusline> on 2026-10-11. A bundled `statusline-shim`
//!   records only `rate_limits` into `<data dir>/claude/rate_limits.json` (a config dir other
//!   than `~/.claude` gets files of its own, `rate_limits-<key>.json` with a hash of that dir,
//!   so two profiles never share a recording). The channel only updates while Claude Code runs
//!   and after a session's first API response, so quiet periods show stale data with its age,
//!   and some plans report no `rate_limits` at all.
//! - **`oauth_usage` (unofficial, single origin).** `GET https://api.anthropic.com/api/oauth/usage`
//!   with the access token Claude Code already holds adds the per-model weekly windows
//!   (Opus, Sonnet, ...). The path, the `anthropic-beta: oauth-2025-04-20` header, both response
//!   shapes (flat `seven_day_*` keys and the newer `limits[]`) and the credentials file layout
//!   come from community tools and issue reports, not from Anthropic; none of it has been
//!   exercised against a live account here, only against a fabricated loopback server. It is off
//!   by default, polled at most every five minutes, and a failure never hides the statusLine data.
//!
//! Collectors are stateless; the only memory kept between polls is a small cache file for the
//! OAuth throttle. Nothing in this crate reads the process environment: the caller resolves the
//! Claude config directory (`CLAUDE_CONFIG_DIR`) into `ClaudeSettings::config_dir`.

mod datafile;
mod lenient;
mod local;
mod notice;
mod oauth;
mod paths;
mod shim;
mod view;

#[cfg(test)]
#[path = "../tests/it/common/mod.rs"]
mod testenv;

use usage_model::settings::ClaudeSettings;
use usage_model::{Ctx, ErrorCode, HarnessError, HarnessId, HarnessView, ViewState};

pub use shim::{install_shim, refresh_shim_copy, shim_status, uninstall_shim};

/// Every notice code this crate can emit (docs/ipc.md, `Notice.code`).
pub const NOTICE_CODES: &[&str] = &[
    notice::WAITING_FIRST_SESSION,
    notice::NO_RATE_LIMITS,
    notice::SHIM_OUTDATED,
    notice::OAUTH_PARTIAL,
];

/// One read of Claude Code's limits. Gates run before any I/O: a disabled tab or a missing consent
/// returns at once without touching a file or the network.
pub async fn collect(settings: &ClaudeSettings, ctx: &Ctx) -> HarnessView {
    collect_with(settings, ctx, &oauth::Endpoint::production()).await
}

/// `collect` with the OAuth endpoint injectable, so tests can talk to a loopback server.
pub(crate) async fn collect_with(
    settings: &ClaudeSettings,
    ctx: &Ctx,
    endpoint: &oauth::Endpoint,
) -> HarnessView {
    if !settings.enabled {
        return HarnessView::disabled(HarnessId::Claude);
    }
    if !settings.shim_consent {
        return HarnessView::needs_consent(HarnessId::Claude);
    }
    let (owned_settings, owned_ctx) = (settings.clone(), ctx.clone());
    let joined =
        tokio::task::spawn_blocking(move || local::view(&owned_settings, &owned_ctx)).await;
    let mut view = joined.unwrap_or_else(task_lost);
    // Extras ride on a usable statusLine view; an idle machine does not poll an unofficial API.
    if settings.oauth_usage && view.state == ViewState::Ready && !view.stale {
        oauth::add_extras(&mut view, settings, ctx, endpoint).await;
    }
    view
}

/// The blocking read was cancelled or panicked: nothing was read, and the next tick tries again.
fn task_lost(_: tokio::task::JoinError) -> HarnessView {
    view::failed(HarnessError::new(ErrorCode::Internal, true))
}

#[cfg(test)]
mod tests;
