//! Test support and suites. All fixtures and tokens are fabricated; nothing reads a real profile
//! and nothing touches the network (a loopback fake stands in for api.github.com).

mod fake_server;
mod flow;
mod http;
mod local;
mod parse;

use std::path::{Path, PathBuf};
use std::time::Duration;

use usage_model::settings::CopilotSettings;
use usage_model::{Ctx, HarnessView};
use usage_support::Secret;

use crate::collect_with;
use crate::local::candidate_dirs;
use crate::remote::Endpoint;
use fake_server::FakeServer;

/// 2027-01-15T08:00:00Z. The fixtures' reset dates are relative to it.
const NOW: i64 = 1_800_000_000;
/// 2027-02-01T00:00:00Z: what the fixtures report and what the derived reset must equal.
const FIRST_OF_FEBRUARY: i64 = 1_801_440_000;
/// A fabricated pasted token: if it shows up anywhere but the Authorization header, a test fails.
const TOKEN: &str = "ghp_fabricatedPastedToken0000";
/// The tokens in `tests/fixtures/{apps,hosts}.json`, plus the enterprise one that must never go out.
const APPS_TOKEN: &str = "gho_fabricatedAppsToken0001";
const HOSTS_TOKEN: &str = "gho_fabricatedHostsToken0002";
const ENTERPRISE_TOKEN: &str = "gho_fabricatedEnterpriseToken0003";
const ALL_TOKENS: [&str; 4] = [TOKEN, APPS_TOKEN, HOSTS_TOKEN, ENTERPRISE_TOKEN];

fn fixture(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join(name);
    std::fs::read_to_string(path).unwrap()
}

fn secret() -> Secret {
    Secret::new(TOKEN)
}

fn ctx(home: &Path) -> Ctx {
    Ctx::new(NOW, home.join("app-data-not-used"), home)
}

fn settings(local_consent: bool) -> CopilotSettings {
    CopilotSettings { enabled: true, local_consent }
}

fn loopback(server: &FakeServer) -> Endpoint {
    Endpoint::loopback(server.base_url(), Duration::from_secs(2))
}

/// The extension directory the collector reads under a fabricated home.
fn copilot_dir(home: &Path) -> PathBuf {
    candidate_dirs(home, cfg!(windows)).remove(0)
}

/// Puts a fixture (or any text) into the fabricated extension directory.
fn put(home: &Path, file: &str, text: &str) {
    let dir = copilot_dir(home);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(file), text).unwrap();
}

fn no_token_anywhere(view: &HarnessView) {
    let json = serde_json::to_string(view).unwrap();
    let debug = format!("{view:?}");
    for token in ALL_TOKENS {
        assert!(!json.contains(token), "{token} leaked into the serialized view");
        assert!(!debug.contains(token), "{token} leaked into the debug output");
    }
}

/// One poll plus the check every view must pass: no token is anywhere in it.
async fn poll(
    settings: &CopilotSettings,
    token: Option<&Secret>,
    home: &Path,
    server: &FakeServer,
) -> HarnessView {
    let view = collect_with(settings, token, &ctx(home), &loopback(server)).await;
    no_token_anywhere(&view);
    view
}
