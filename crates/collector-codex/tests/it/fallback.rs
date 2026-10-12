//! When the app-server cannot be used, the newest rollout stands in, labelled as last seen. Which
//! failures allow that, and how the stand-in is described.

use usage_model::{ErrorCode, HarnessView, Severity, Source, ViewState};

use crate::common::{FIVE_HOUR_RESET, NOW, Rig, WEEKLY_RESET, assert_failure, rollout};

const EVENT_TIME: &str = "2026-10-10T19:59:00.000Z";
/// `EVENT_TIME` in epoch seconds: a minute before `NOW`.
const EVENT_SECS: i64 = NOW - 60;

fn plant(rig: &Rig) {
    let line = rollout::event(EVENT_TIME, Some((31.0, FIVE_HOUR_RESET)), Some((9.0, WEEKLY_RESET)));
    rollout::write_session(
        rig.home.path(),
        "2026/10/10",
        "rollout-2026-10-10T19-00-00-a.jsonl",
        &[line],
    );
}

fn assert_rollout_view(view: &HarnessView, reason: &str) {
    assert_eq!(view.state, ViewState::Ready, "{reason}: {view:?}");
    assert_eq!(view.source, Some(Source { id: "rollout".into(), official: false }), "{reason}");
    assert_eq!(view.read_at, Some(EVENT_SECS), "{reason}: dated by the event, not by the poll");
    assert!(!view.stale && view.error.is_none(), "{reason}");
    assert_eq!(view.plan.as_deref(), Some("Plus"), "{reason}");
    let used: Vec<_> = view.windows.iter().map(|w| (w.id.as_str(), w.used_pct)).collect();
    assert_eq!(used, [("five_hour", Some(31.0)), ("weekly", Some(9.0))], "{reason}");
    let [notice] = view.notices.as_slice() else { panic!("{reason}: {:?}", view.notices) };
    assert_eq!((notice.code.as_str(), notice.severity), ("codex_rollout_fallback", Severity::Info));
    assert_eq!(notice.params.get("reason").map(String::as_str), Some(reason), "{reason}");
    assert_eq!(notice.params.len(), 1, "{reason}: {:?}", notice.params);
}

#[tokio::test]
async fn a_missing_binary_falls_back_to_the_newest_rollout() {
    let rig = Rig::without_binary();
    plant(&rig);
    assert_rollout_view(&rig.collect().await, "binary_not_found");
}

#[tokio::test]
async fn each_failure_that_is_not_the_users_account_falls_back_and_says_why() {
    let reasons = [
        ("exits_early", "handshake_failed"),
        ("exits_midway", "handshake_failed"),
        ("bad_handshake", "handshake_failed"),
        ("garbage_line", "protocol_error"),
        ("huge_line", "protocol_error"),
        ("bad_shape", "endpoint_changed"),
        ("backend_429", "rate_limited"),
        ("backend_500", "server_error"),
        ("backend_418", "bad_response"),
        ("offline", "network"),
    ];
    for (mode, reason) in reasons {
        let rig = Rig::new(mode);
        plant(&rig);
        assert_rollout_view(&rig.collect().await, reason);
    }
}

#[tokio::test]
async fn a_timeout_falls_back_too() {
    let rig = Rig::new("hang");
    plant(&rig);
    let view = rig.collect_within(std::time::Duration::from_secs(1)).await;
    assert_rollout_view(&view, "timeout");
}

#[tokio::test]
async fn problems_with_the_account_never_fall_back_to_old_numbers() {
    let cases = [
        ("not_signed_in", ErrorCode::NotSignedIn),
        ("not_signed_in_rpc", ErrorCode::NotSignedIn),
        ("backend_401", ErrorCode::NotSignedIn),
        ("backend_403", ErrorCode::Unauthorized),
    ];
    for (mode, code) in cases {
        let rig = Rig::new(mode);
        plant(&rig);
        let view = rig.collect().await;
        assert_failure(&view, code, false);
    }
}

#[tokio::test]
async fn an_api_key_login_needs_no_fallback_either() {
    let rig = Rig::new("api_key");
    plant(&rig);
    let view = rig.collect().await;
    assert_eq!(view.source.map(|s| s.id).as_deref(), Some("app_server"));
    assert!(view.windows.is_empty(), "old numbers of another login must not appear");
}

#[tokio::test]
async fn a_working_app_server_is_preferred_over_a_rollout() {
    let rig = Rig::new("ok");
    plant(&rig);
    let view = rig.collect().await;
    assert_eq!(view.source.map(|s| s.id).as_deref(), Some("app_server"));
    assert!(view.notices.is_empty());
}

#[tokio::test]
async fn without_any_rollout_the_original_error_is_what_shows() {
    let rig = Rig::new("bad_shape");
    let view = rig.collect().await;
    assert_failure(&view, ErrorCode::EndpointChanged, false);
}

#[tokio::test]
async fn a_stale_rollout_is_still_shown_but_marked_stale() {
    let rig = Rig::without_binary();
    let line = rollout::event(EVENT_TIME, Some((31.0, NOW - 10)), Some((9.0, NOW - 5)));
    rollout::write_session(rig.home.path(), "2026/10/10", "rollout-a.jsonl", &[line]);
    let view = rig.collect().await;
    assert_eq!(view.state, ViewState::Ready);
    assert!(view.stale, "both windows reset before now");
    assert_eq!(view.windows.len(), 2);

    let half = Rig::without_binary();
    let line = rollout::event(EVENT_TIME, Some((31.0, NOW - 10)), Some((9.0, WEEKLY_RESET)));
    rollout::write_session(half.home.path(), "2026/10/10", "rollout-a.jsonl", &[line]);
    assert!(!half.collect().await.stale, "the weekly window is still running");
}
