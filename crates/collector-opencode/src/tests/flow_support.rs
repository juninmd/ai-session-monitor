//! Helpers for whole-poll tests: a loopback endpoint, one poll, and the checks every poll gets.

use std::path::Path;
use std::time::Duration;

use usage_model::settings::OpencodeSettings;
use usage_model::{Ctx, HarnessView};
use usage_support::Secret;

use super::fake_server::{FakeServer, Reply};
use super::{assert_clean, ctx, fixture};
use crate::remote::Endpoint;
use crate::{NOTICE_CODES, collect_with};

pub fn loopback(server: &FakeServer) -> Endpoint {
    Endpoint::loopback(server.base_url(), Duration::from_secs(2))
}

pub fn ok_server() -> FakeServer {
    serving("ok.json")
}

/// Answers 200 with a fixture body.
pub fn serving(name: &'static str) -> FakeServer {
    FakeServer::always(move || Reply::json(200, &fixture(name)))
}

pub fn replying(status: u16, body: &'static str) -> FakeServer {
    FakeServer::always(move || Reply::json(status, body))
}

pub async fn run(
    settings: &OpencodeSettings,
    key: Option<&Secret>,
    server: &FakeServer,
    home: &Path,
) -> HarnessView {
    run_at(settings, key, server, &ctx(home)).await
}

/// One poll plus the checks every view must pass: no secret anywhere, only catalogued notices.
pub async fn run_at(
    settings: &OpencodeSettings,
    key: Option<&Secret>,
    server: &FakeServer,
    ctx: &Ctx,
) -> HarnessView {
    let view = collect_with(settings, key, ctx, &loopback(server)).await;
    assert_clean(&view);
    for notice in &view.notices {
        assert!(NOTICE_CODES.contains(&notice.code.as_str()), "uncatalogued {}", notice.code);
    }
    view
}

pub fn window_ids(view: &HarnessView) -> Vec<&str> {
    view.windows.iter().map(|window| window.id.as_str()).collect()
}

pub fn notice_codes(view: &HarnessView) -> Vec<&str> {
    view.notices.iter().map(|notice| notice.code.as_str()).collect()
}
