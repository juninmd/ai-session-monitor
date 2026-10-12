//! The network stays quiet unless there is a usable statusLine view to add to. Every case has a
//! valid login and a ready server, so a single stray request would be visible.

use serde_json::json;
use usage_model::{ErrorCode, HarnessId};

use super::*;
use crate::testenv::{null_limits, settings_foreign, usage_limits};

/// One OAuth-enabled poll that must not touch the server nor mention the source.
async fn quiet(env: &Env, now: i64) -> HarnessView {
    let server = FakeServer::json(&usage_limits());
    let view = poll(env, &server, now).await;
    server.assert_untouched();
    assert!(!notice_codes(&view).contains(&"claude_oauth_partial"), "{view:?}");
    assert!(!view.meta.contains_key("extra_source") && view.plan.is_none());
    view
}

#[tokio::test]
async fn with_oauth_off_nothing_is_asked_and_the_login_is_not_even_opened() {
    let env = ready_env();
    // A login path that cannot be read would surface as a notice if anyone looked at it.
    std::fs::remove_file(env.credentials()).unwrap();
    std::fs::create_dir_all(env.credentials()).unwrap();
    let server = FakeServer::json(&usage_limits());
    let view = crate::collect_with(&env.settings(), &env.ctx(), &endpoint(&server)).await;
    server.assert_untouched();
    assert_eq!((view.state, view.windows.len(), view.notices.len()), (ViewState::Ready, 2, 0));
    assert!(!env.oauth_cache().exists(), "no throttle file either");
}

#[tokio::test]
async fn a_disabled_tab_is_never_polled() {
    let env = ready_env();
    let server = FakeServer::json(&usage_limits());
    let off = ClaudeSettings { enabled: false, ..oauth_on(&env) };
    let view = crate::collect_with(&off, &env.ctx(), &endpoint(&server)).await;
    server.assert_untouched();
    assert_eq!(view, HarnessView::disabled(HarnessId::Claude));
}

#[tokio::test]
async fn without_consent_nothing_is_read_or_asked() {
    let env = ready_env();
    let server = FakeServer::json(&usage_limits());
    let no_consent = ClaudeSettings { shim_consent: false, ..oauth_on(&env) };
    let view = crate::collect_with(&no_consent, &env.ctx(), &endpoint(&server)).await;
    server.assert_untouched();
    assert_eq!(view, HarnessView::needs_consent(HarnessId::Claude));
}

#[tokio::test]
async fn a_missing_shim_needs_setup_and_asks_nothing() {
    let env = Env::new();
    env.write_credentials((NOW + 3600) * 1000);
    let view = quiet(&env, NOW).await;
    assert_eq!(view.state, ViewState::NeedsSetup);
}

#[tokio::test]
async fn somebody_elses_status_line_needs_setup_and_asks_nothing() {
    let env = Env::new();
    env.write_settings(settings_foreign());
    env.write_credentials((NOW + 3600) * 1000);
    assert_eq!(quiet(&env, NOW).await.state, ViewState::NeedsSetup);
}

#[tokio::test]
async fn an_outdated_shim_needs_setup_and_asks_nothing() {
    let env = ready_env();
    std::fs::remove_file(env.shim_copy()).unwrap();
    let view = quiet(&env, NOW).await;
    assert_eq!(view.state, ViewState::NeedsSetup);
    assert_eq!(notice_codes(&view), ["claude_shim_outdated"]);
}

#[tokio::test]
async fn unreadable_settings_are_an_error_and_ask_nothing() {
    let env = ready_env();
    env.write_settings("{ not json");
    let view = quiet(&env, NOW).await;
    assert_eq!(view.error.map(|e| e.code), Some(ErrorCode::SettingsUnreadable));
}

#[tokio::test]
async fn waiting_for_the_first_session_asks_nothing() {
    let env = ready_env();
    std::fs::remove_file(env.rate_limits()).unwrap();
    let view = quiet(&env, NOW).await;
    assert_eq!(notice_codes(&view), ["claude_waiting_first_session"]);
}

#[tokio::test]
async fn no_rate_limits_ask_nothing() {
    let env = ready_env();
    env.write_rate_limits(&null_limits());
    let view = quiet(&env, NOW).await;
    assert_eq!(
        (view.state, notice_codes(&view)),
        (ViewState::NoData, vec!["claude_no_rate_limits"])
    );
}

#[tokio::test]
async fn a_broken_recording_is_an_error_and_asks_nothing() {
    let env = ready_env();
    env.write_rate_limits("{ torn");
    let view = quiet(&env, NOW).await;
    assert_eq!(view.error.map(|e| e.code), Some(ErrorCode::BadResponse));
}

#[tokio::test]
async fn a_stale_recording_asks_nothing() {
    let env = ready_env();
    // Both windows of the fixture have reset by then.
    let view = quiet(&env, NOW + 400_001).await;
    assert!(view.stale && view.state == ViewState::Ready);

    let ancient = json!({"v": 1, "capturedAt": NOW - 9 * 86_400, "rateLimits": {"seven_day": {"used_percentage": 5}}});
    env.write_rate_limits(&ancient.to_string());
    assert!(quiet(&env, NOW).await.stale);
}
