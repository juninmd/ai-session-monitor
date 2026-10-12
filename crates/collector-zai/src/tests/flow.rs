//! Whole polls: the gates that run before any I/O, the view a good answer becomes, and the key
//! staying out of every view.

use usage_model::settings::ZaiSettings;
use usage_model::{ErrorCode, HarnessId, HarnessView, ViewState};
use usage_support::Secret;

use super::fake_server::{FakeServer, Reply};
use super::{KEY, NOW, enabled, fixture, poll, secret};
use crate::{Plan, plan};

const BLANK_KEYS: [&str; 5] = ["", " ", "\t\r\n", "\u{a0}\u{2003}", "   \n   "];

fn serving(name: &'static str) -> FakeServer {
    FakeServer::always(move || Reply::json(200, &fixture(name)))
}

#[test]
fn a_disabled_harness_is_never_read_whatever_else_is_set() {
    let key = secret();
    for key in [None, Some(&key)] {
        let off = ZaiSettings { enabled: false };
        match plan(&off, key) {
            Plan::Done(view) => assert_eq!(*view, HarnessView::disabled(HarnessId::Zai)),
            Plan::Read(_) => panic!("a disabled harness must not read"),
        }
    }
}

#[test]
fn no_key_or_a_blank_one_needs_setup() {
    let expected = HarnessView::needs_setup(HarnessId::Zai);
    let blanks: Vec<Secret> = BLANK_KEYS.iter().map(|blank| Secret::new(*blank)).collect();
    for key in std::iter::once(None).chain(blanks.iter().map(Some)) {
        match plan(&enabled(), key) {
            Plan::Done(view) => assert_eq!(*view, expected, "{key:?}"),
            Plan::Read(_) => panic!("{key:?} must not read"),
        }
    }
}

#[tokio::test]
async fn gated_polls_open_no_socket() {
    let server = serving("ok_pro.json");
    let off = ZaiSettings { enabled: false };
    assert_eq!(poll(&off, Some(&secret()), &server).await.state, ViewState::Disabled);
    assert_eq!(poll(&enabled(), None, &server).await.state, ViewState::NeedsSetup);
    assert_eq!(
        poll(&enabled(), Some(&Secret::new("  ")), &server).await.state,
        ViewState::NeedsSetup
    );
    server.assert_untouched();
}

#[tokio::test]
async fn a_good_answer_is_a_ready_unofficial_view_stamped_with_the_poll_time() {
    let server = serving("ok_pro.json");
    let view = poll(&enabled(), Some(&secret()), &server).await;
    assert_eq!(view.state, ViewState::Ready);
    assert_eq!(view.id, HarnessId::Zai);
    let source = view.source.as_ref().unwrap();
    assert_eq!((source.id.as_str(), source.official), ("quota_limit", false));
    assert_eq!(view.read_at, Some(NOW));
    assert_eq!(view.plan.as_deref(), Some("Pro"));
    assert!(!view.stale);
    assert!(view.error.is_none() && view.notices.is_empty() && view.meta.is_empty());
    assert_eq!(view.windows.len(), 3);
}

#[tokio::test]
async fn a_view_whose_windows_all_reset_is_stale_and_one_without_a_reset_is_not() {
    let past = r#"{"code":200,"success":true,"data":{"limits":[
        {"type":"TOKENS_LIMIT","unit":3,"number":5,"percentage":100,"nextResetTime":1799996400000}]}}"#;
    let expired = FakeServer::always(move || Reply::json(200, past));
    assert!(poll(&enabled(), Some(&secret()), &expired).await.stale);
    let open_ended = serving("ok_lite.json");
    assert!(!poll(&enabled(), Some(&secret()), &open_ended).await.stale);
}

#[tokio::test]
async fn failures_become_error_views_without_the_key_or_the_response_text() {
    for (status, code) in [
        (401, ErrorCode::Unauthorized),
        (403, ErrorCode::Unauthorized),
        (404, ErrorCode::EndpointChanged),
        (429, ErrorCode::RateLimited),
        (500, ErrorCode::ServerError),
    ] {
        let server = FakeServer::always(move || Reply::json(status, &fixture("error_auth.json")));
        let view = poll(&enabled(), Some(&secret()), &server).await;
        assert_eq!(view.state, ViewState::Error, "{status}");
        assert_eq!(view.error.as_ref().map(|e| e.code), Some(code), "{status}");
        assert!(view.windows.is_empty());
        assert!(!serde_json::to_string(&view).unwrap().contains("fabricated"));
    }
}

#[tokio::test]
async fn failures_reported_inside_a_200_map_like_the_same_http_status() {
    for (name, code, retryable) in [
        ("auth_in_band.json", ErrorCode::Unauthorized, false),
        ("rate_in_band.json", ErrorCode::RateLimited, true),
        ("failure_in_band.json", ErrorCode::ServerError, true),
        ("unknown_failure.json", ErrorCode::EndpointChanged, false),
        ("empty_limits.json", ErrorCode::UnsupportedPlan, false),
    ] {
        let view = poll(&enabled(), Some(&secret()), &serving(name)).await;
        assert_eq!(view.state, ViewState::Error, "{name}");
        let error = view.error.as_ref().unwrap();
        assert_eq!((error.code, error.retryable), (code, retryable), "{name}");
        assert!(view.windows.is_empty() && view.plan.is_none(), "{name}");
        assert!(!serde_json::to_string(&view).unwrap().contains("fabricated"), "{name}");
    }
    let auth = poll(&enabled(), Some(&secret()), &serving("auth_in_band.json")).await;
    assert_eq!(auth.error.unwrap().params.get("ambiguous").map(String::as_str), Some("true"));
}

#[tokio::test]
async fn the_key_reaches_only_the_authorization_header_of_the_one_request() {
    let server = serving("ok_pro.json");
    let view = poll(&enabled(), Some(&secret()), &server).await;
    assert_eq!(view.state, ViewState::Ready);
    let requests = server.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].header("authorization"), Some(KEY));
}
