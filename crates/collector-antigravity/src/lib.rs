//! Antigravity collector: how much of the Gemini and the Claude/GPT-OSS pools is left, read from
//! the language server the Antigravity app runs on this PC.
//!
//! # Where the data comes from
//!
//! First choice is the `agy` CLI: `agy -p "/usage" --output-format json` from its fixed install
//! path (see `cli`), verified against a live agy 1.3.3 on 2026-10-12. Only when that is absent or
//! fails does the IDE language server below get asked; everything in the rest of these docs about
//! the IDE path remains unverified.
//!
//! Antigravity starts a local language server per window. It listens on `127.0.0.1`, speaks
//! Connect (JSON over HTTP) and refuses a request without the CSRF token it was started with. The
//! collector reads that token and the ports from the server's own command line, then calls
//! `GetUserStatus` (plan, and a quota per model on older builds) and `RetrieveUserQuotaSummary`
//! (a five hour and a weekly bucket per model group, on newer builds).
//!
//! The interface is **unofficial**: Google documents the plans, not this server. The view says so
//! (`source.official = false`) and the collector only runs once the user consented.
//!
//! # Confidence, and where the shapes were verified
//!
//! **Likely, never confirmed against a live Antigravity.** Nothing here has talked to a real
//! language server; the shapes come from community clients that read the same endpoints and from
//! issue threads that quote real payloads (read on 2026-10-10 and 2026-10-11):
//!
//! - request and response fields, version differences, the 404 of `RetrieveUserQuotaSummary` on
//!   the IDE server, token flags: <https://github.com/steipete/CodexBar/blob/main/docs/antigravity.md>
//!   and its parsers under `Sources/CodexBarCore/Providers/Antigravity/`
//! - `GetUserStatus` and summary fixtures, the oneof form `{"case", "value"}` of the share:
//!   <https://github.com/OFK0/ai-usage-tracker/pull/81> (issues 73, 74 and 76; issue 76 records
//!   that a live capture is still missing there too)
//! - a payload from the 2.x hub (`window`, `remainingFraction`, `resetTime` per bucket):
//!   <https://github.com/vibeislandapp/vibe-island/issues/250>
//! - `quotaInfo` seen live with only `remainingFraction` and `resetTime`, and the `--app_data_dir`
//!   flag telling two instances apart: <https://github.com/niflheim77/AICycle-Widget/pull/1>
//! - the `X-Codeium-Csrf-Token` check on every request and the random `extension_server_port`:
//!   <https://www.hacktron.ai/blog/hacking-google-antigravity>
//! - `--https_server_port` as the port flag of the 2.x server:
//!   <https://github.com/nesszer/Win-CodexBar/issues/192>
//! - pools and cadences (Pro and Ultra refresh every five hours until a weekly cap, other plans
//!   weekly): <https://antigravity.google/docs/plans>
//!
//! What a live run could still change: field names, whether the backend reports its real
//! `remainingFraction` (one report says it was seen as 1 whatever the use), the flags on the
//! command line of a given build, and which of the listening ports speaks HTTPS.
//!
//! # Behaviour
//!
//! - Gates first: disabled, then no consent, with no process, port or file touched.
//! - Same-user servers only, Antigravity only (`--app_data_dir antigravity*`, or an install path
//!   that says so). No server is a normal state (`NotRunning`, retryable).
//! - The CSRF token lives in a `Secret` from the command line to the request header and appears
//!   in no error, notice, fact or log.
//! - Pools: Gemini Pro and Flash share one (`gemini`), Claude and GPT-OSS another (`claude_gpt`).
//!   Windows are `<pool>:five_hour` and `<pool>:weekly`; `used_pct` is `(1 - remainingFraction)
//!   * 100`. A missing or out-of-range share is unknown (`None`), never 0 %, with the notice
//!   `antigravity_pool_unknown`.
//! - The status is asked first and the summary second. A status in a shape this collector does not
//!   know (or a 404) still leaves the summary a chance, since the quota is what matters; a refusal
//!   or silence ends the attempt at once.
//! - Older IDE builds state no window length; it is guessed from the reset time and then carries
//!   no `window_minutes`, so a wrong guess cannot raise a false pace alarm.
//! - `stale` is set when every window has already reset: the server kept serving last cycle.

mod cli;
mod collector;
mod discovery;
mod errors;
mod quota;
mod rpc;
mod view;

#[cfg(test)]
mod fake;
#[cfg(test)]
mod fixtures;

use usage_model::settings::AntigravitySettings;
use usage_model::{Ctx, HarnessView};

/// Notice codes this collector emits (see `docs/ipc.md`).
pub const NOTICE_CODES: &[&str] = &[view::POOL_UNKNOWN];

/// One poll. Always returns a view for `HarnessId::Antigravity`; failures are in the view.
pub async fn collect(settings: &AntigravitySettings, ctx: &Ctx) -> HarnessView {
    let (discovery, cli) = (discovery::SystemDiscovery::new(), cli::SystemCli::new());
    collector::collect_with(settings, ctx, &discovery, &cli, &collector::Config::default()).await
}
