//! The whole poll against a stand-in server on loopback: gates, success paths, failures.

mod failures;
mod flow;
mod gates;
mod probing;
mod security;
mod transport;

use std::time::Duration;

use usage_model::settings::AntigravitySettings;
use usage_model::{Ctx, HarnessView};

use super::{Config, collect_with};
use crate::discovery::Target;
use crate::fake::{FakeServer, Script, StaticDiscovery};
use crate::fixtures::{NOW, text};
use crate::rpc::Limits;

const TOKEN: &str = "fake-csrf-token-0001";

fn on() -> AntigravitySettings {
    AntigravitySettings { enabled: true, consent: true }
}

/// The collector never touches these paths; tests that must prove it use a real temp dir.
fn ctx() -> Ctx {
    Ctx::new(NOW, "data-dir-the-collector-never-uses", "home-dir-the-collector-never-uses")
}

/// Deadlines chosen by the test, so a hang fails in seconds instead of at the 30 s poll limit.
fn timed(request_timeout: Duration, budget: Duration) -> Config {
    Config { limits: Limits { request_timeout, max_body_bytes: 1 << 20 }, budget }
}

fn config() -> Config {
    timed(Duration::from_secs(3), Duration::from_secs(20))
}

async fn poll(targets: Vec<Target>) -> HarnessView {
    poll_with(&on(), &StaticDiscovery::new(Ok(targets)), &config()).await
}

async fn poll_with(
    settings: &AntigravitySettings,
    discovery: &StaticDiscovery,
    config: &Config,
) -> HarnessView {
    collect_with(settings, &ctx(), discovery, config).await
}

/// An Antigravity stand-in over plain HTTP serving these fixtures (`summary: None` is the IDE).
async fn serve(status: &str, summary: Option<&str>) -> FakeServer {
    Script::new(TOKEN, text(status), summary.map(text)).serve_http().await
}

/// `(id, used_pct, resets_at)` of every window, in order.
fn rows(view: &HarnessView) -> Vec<(&str, Option<f64>, Option<i64>)> {
    view.windows.iter().map(|w| (w.id.as_str(), w.used_pct, w.resets_at)).collect()
}
