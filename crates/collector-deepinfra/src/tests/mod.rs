//! Test support and suites. All fixtures and keys are fabricated; nothing reads a real profile.

mod fake_server;
mod flow;
mod http;
mod parse;

use std::path::Path;
use std::time::Duration;

use usage_model::settings::DeepinfraSettings;
use usage_model::{Ctx, HarnessView};
use usage_support::Secret;

use crate::collect_with;
use crate::remote::Endpoint;
use fake_server::{FakeServer, Reply, Request};

/// 2027-01-15T08:00:00Z.
const NOW: i64 = 1_800_000_000;
/// A fabricated key: if it shows up anywhere but the Authorization header, a test fails.
const KEY: &str = "fab-deepinfra-key-3b9e57";

fn fixture(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join(name);
    std::fs::read_to_string(path).unwrap()
}

fn secret() -> Secret {
    Secret::new(KEY)
}

fn ctx() -> Ctx {
    Ctx::new(NOW, "app-data-not-used", "home-not-used")
}

fn enabled() -> DeepinfraSettings {
    DeepinfraSettings { enabled: true }
}

fn loopback(server: &FakeServer) -> Endpoint {
    Endpoint::loopback(server.base_url(), Duration::from_secs(2))
}

/// Routes by path: the checklist and the usage document, each with its own status.
fn routed(checklist: (u16, &'static str), usage: (u16, &'static str)) -> FakeServer {
    FakeServer::start(move |request: &Request| {
        let (status, name) =
            if request.target.starts_with("/payment/usage") { usage } else { checklist };
        Reply::json(status, &fixture(name))
    })
}

/// One poll plus the checks every view must pass: the key is nowhere in it.
async fn poll(
    settings: &DeepinfraSettings,
    key: Option<&Secret>,
    server: &FakeServer,
) -> HarnessView {
    let view = collect_with(settings, key, &ctx(), &loopback(server)).await;
    let json = serde_json::to_string(&view).unwrap();
    assert!(!json.contains(KEY), "the key leaked into the serialized view");
    assert!(!format!("{view:?}").contains(KEY), "the key leaked into the debug output");
    view
}
