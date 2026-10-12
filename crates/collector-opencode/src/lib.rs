//! OpenCode collector: real quota windows from the Go usage endpoint plus a local consumption
//! estimate read from OpenCode's own database.
//!
//! # Sources and confidence (both unofficial)
//!
//! - **`go_usage`**: `GET https://opencode.ai/zen/go/v1/usage` with the user's Go key as Bearer.
//!   Built by the OpenCode maintainers but undocumented. Single origin: the body shape comes from
//!   their source and one live response pasted in a public issue, never from a call this project
//!   made with a real key, so a change surfaces as `EndpointChanged`, not as made-up numbers. The
//!   service has rejected valid keys, hence 401/403 are `Unauthorized` with `ambiguous = true`.
//! - **`local_db`**: `opencode.db` opened read-only. Internal format, undocumented: the layout comes
//!   from the project's research notes (a read-only aggregate check on 2026-10-11); this crate's
//!   tests only ever open fabricated databases. The numbers are a list-price estimate for this PC,
//!   never quota, and are never turned into a percentage or a reset time. Assistant messages
//!   without any readable `time.created` are reported as a changed layout, not as a zero.
//!
//! # Behaviour
//!
//! - Gates run before any I/O: disabled; neither a key nor local consent is `NeedsSetup`.
//! - A key reads the windows (`rolling` 5 h, `weekly`, `monthly`); consent reads the facts; both
//!   do both concurrently. A remote failure is an `Error` view; a local failure never is: it is
//!   the `opencode_db_unreadable` notice (or `NoData` when the estimate is all there is).
//! - The collector is stateless and issues one GET per call. app-core's `timing::cadence` keeps
//!   OpenCode at 300 s or slower (docs/ipc.md, Polling); any other caller must do the same.
//! - The key is only ever sent to `https://opencode.ai`: the host is a constant and release builds
//!   have no way to point the client elsewhere.

mod compose;
mod local;
mod notice;
mod remote;
#[cfg(test)]
mod tests;

use usage_model::settings::OpencodeSettings;
use usage_model::{Ctx, HarnessId, HarnessView};
use usage_support::Secret;

/// Every notice code this crate can emit; the UI catalog (docs/ipc.md) must know each one.
pub const NOTICE_CODES: &[&str] = &[
    notice::WINDOW_IDLE,
    notice::RATE_LIMITED,
    notice::NO_KEY,
    notice::LOCAL_ESTIMATE,
    notice::DB_UNREADABLE,
];

pub async fn collect(
    settings: &OpencodeSettings,
    api_key: Option<&Secret>,
    ctx: &Ctx,
) -> HarnessView {
    collect_with(settings, api_key, ctx, &remote::Endpoint::production()).await
}

/// What one poll reads, decided from settings alone, before any file or socket is touched.
enum Plan<'a> {
    /// Boxed: a view is large and the other variant is two words.
    Done(Box<HarnessView>),
    Read {
        key: Option<&'a Secret>,
        local: bool,
    },
}

fn plan<'a>(settings: &OpencodeSettings, api_key: Option<&'a Secret>) -> Plan<'a> {
    if !settings.enabled {
        return Plan::Done(Box::new(HarnessView::disabled(HarnessId::Opencode)));
    }
    let key = api_key.filter(|secret| !secret.expose().trim().is_empty());
    if key.is_none() && !settings.local_consent {
        return Plan::Done(Box::new(HarnessView::needs_setup(HarnessId::Opencode)));
    }
    Plan::Read { key, local: settings.local_consent }
}

async fn collect_with(
    settings: &OpencodeSettings,
    api_key: Option<&Secret>,
    ctx: &Ctx,
    endpoint: &remote::Endpoint,
) -> HarnessView {
    let (key, with_local) = match plan(settings, api_key) {
        Plan::Done(view) => return *view,
        Plan::Read { key, local } => (key, local),
    };
    let remote = async {
        match key {
            Some(key) => Some(remote::fetch(endpoint, key, ctx.now_secs).await),
            None => None,
        }
    };
    let local = async { if with_local { Some(local::read(settings, ctx).await) } else { None } };
    let (remote, local) = tokio::join!(remote, local);
    compose::compose(remote, local, ctx.now_secs)
}
