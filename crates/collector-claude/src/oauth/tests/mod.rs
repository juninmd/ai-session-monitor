//! The OAuth source against a loopback fake server: what is sent, how every outcome is mapped,
//! the five-minute throttle and the gates that keep the network quiet. No test ever points at
//! the production host, and the only token in play is a fabricated one.

mod errors;
mod gates;
mod login;
mod success;
mod throttle;

use std::time::Duration;

use usage_model::settings::ClaudeSettings;
use usage_model::{HarnessView, ViewState};

use super::Endpoint;
use super::fake_server::{FakeServer, Reply};
use crate::testenv::{Env, FAKE_TOKEN, NOW, notice_codes, ok_both, wire};

/// Long enough for a loaded CI machine, short enough to fail fast on a hang.
const TEST_TIMEOUT: Duration = Duration::from_secs(5);

/// Consented, shim installed, a fresh recording on disk and a login valid for two days.
fn ready_env() -> Env {
    let env = installed_env();
    env.write_credentials((NOW + 2 * 86_400) * 1000);
    env
}

/// Like `ready_env` but without a login file.
fn installed_env() -> Env {
    let env = Env::new();
    crate::install_shim(&env.settings(), &env.ctx(), &env.shim_source()).unwrap();
    env.write_rate_limits(&ok_both());
    env
}

fn oauth_on(env: &Env) -> ClaudeSettings {
    ClaudeSettings { oauth_usage: true, ..env.settings() }
}

fn endpoint(server: &FakeServer) -> Endpoint {
    Endpoint { base: server.base(), allow_loopback_http: true, timeout: TEST_TIMEOUT }
}

/// One poll at `now` with OAuth on. Whatever happens, the token must not reach the wire view.
async fn poll_with(env: &Env, endpoint: &Endpoint, now: i64) -> HarnessView {
    let view = crate::collect_with(&oauth_on(env), &env.ctx_at(now), endpoint).await;
    assert!(!wire(&view).contains(FAKE_TOKEN), "the token leaked into the view");
    view
}

async fn poll(env: &Env, server: &FakeServer, now: i64) -> HarnessView {
    poll_with(env, &endpoint(server), now).await
}

/// The statusLine windows stand and a `claude_oauth_partial` notice names the reason.
fn assert_partial(view: &HarnessView, reason: &str) {
    assert_eq!(view.state, ViewState::Ready, "the statusLine data must survive: {view:?}");
    assert_eq!(view.windows.len(), 2, "only the statusLine windows");
    assert_eq!(notice_codes(view), ["claude_oauth_partial"]);
    assert_eq!(view.notices[0].params["reason"], reason);
    assert!(view.plan.is_none() && !view.meta.contains_key("extra_source"));
}
