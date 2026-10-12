//! The OAuth option through the public `collect`, in the situations where no request can be made:
//! the login is missing, expired or unreadable. The request itself is tested against a loopback
//! server in the crate's unit tests; no test ever contacts the real host.

use collector_claude::collect;
use usage_model::settings::ClaudeSettings;
use usage_model::{Severity, ViewState};

use crate::common::{Env, FAKE_TOKEN, NOW, notice_codes, wire};
use crate::setup::recording;

fn with_oauth(env: &Env) -> ClaudeSettings {
    ClaudeSettings { oauth_usage: true, ..env.settings() }
}

async fn partial_reason(env: &Env) -> String {
    let view = collect(&with_oauth(env), &env.ctx()).await;
    assert_eq!(
        (view.state, view.windows.len()),
        (ViewState::Ready, 2),
        "the statusLine data stays"
    );
    assert_eq!(notice_codes(&view), ["claude_oauth_partial"]);
    assert_eq!(view.notices[0].severity, Severity::Info);
    assert!(view.plan.is_none() && !view.meta.contains_key("extra_source"));
    assert!(!wire(&view).contains(FAKE_TOKEN), "the token must never show");
    view.notices[0].params["reason"].clone()
}

#[tokio::test]
async fn without_a_login_the_statusline_data_stays_and_a_notice_says_why() {
    let env = recording();
    assert_eq!(partial_reason(&env).await, "not_signed_in");
}

#[tokio::test]
async fn an_expired_login_is_reported_before_any_request() {
    let env = recording();
    env.write_credentials((NOW - 1) * 1000);
    assert_eq!(partial_reason(&env).await, "oauth_expired");
}

#[tokio::test]
async fn a_login_file_that_is_not_understood_is_a_changed_format() {
    let env = recording();
    env.write(
        &env.credentials(),
        format!("{{\"claudeAiOauth\": {{\"accessToken\": \"{FAKE_TOKEN}\""),
    );
    assert_eq!(partial_reason(&env).await, "endpoint_changed");
}

#[tokio::test]
async fn with_the_option_off_the_login_is_not_even_looked_at() {
    let env = recording();
    env.write_credentials((NOW - 1) * 1000);
    let view = collect(&env.settings(), &env.ctx()).await;
    assert!(view.notices.is_empty() && view.meta.is_empty(), "{view:?}");
    assert!(!env.oauth_cache().exists());
}

#[tokio::test]
async fn the_option_is_ignored_when_there_is_no_statusline_view_to_add_to() {
    let env = Env::new();
    env.write_credentials((NOW + 3600) * 1000);
    let view = collect(&with_oauth(&env), &env.ctx()).await;
    assert_eq!(view.state, ViewState::NeedsSetup);
    assert!(view.notices.is_empty());
}
