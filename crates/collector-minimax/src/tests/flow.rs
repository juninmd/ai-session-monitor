//! Whole polls: the gates that run before any I/O, the view a good answer becomes, and the key
//! staying out of every view.

use usage_model::settings::MinimaxSettings;
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
        match plan(&MinimaxSettings { enabled: false }, key) {
            Plan::Done(view) => assert_eq!(*view, HarnessView::disabled(HarnessId::Minimax)),
            Plan::Read(_) => panic!("a disabled harness must not read"),
        }
    }
}

#[test]
fn no_key_or_a_blank_one_needs_setup() {
    let expected = HarnessView::needs_setup(HarnessId::Minimax);
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
    let server = serving("ok.json");
    let off = MinimaxSettings { enabled: false };
    assert_eq!(poll(&off, Some(&secret()), &server).await.state, ViewState::Disabled);
    assert_eq!(poll(&enabled(), None, &server).await.state, ViewState::NeedsSetup);
    assert_eq!(
        poll(&enabled(), Some(&Secret::new("  ")), &server).await.state,
        ViewState::NeedsSetup
    );
    server.assert_untouched();
}
#[tokio::test]
async fn a_good_answer_is_a_ready_unofficial_view_of_windows_and_no_facts() {
    let view = poll(&enabled(), Some(&secret()), &serving("ok.json")).await;
    assert_eq!((view.state, view.id), (ViewState::Ready, HarnessId::Minimax));
    let source = view.source.as_ref().unwrap();
    assert_eq!((source.id.as_str(), source.official), ("token_plan_remains", false));
    assert_eq!(view.read_at, Some(NOW));
    assert_eq!(view.windows.len(), 3);
    assert!(view.facts.is_empty());
    assert!(!view.stale && view.error.is_none() && view.notices.is_empty() && view.meta.is_empty());
}

#[tokio::test]
async fn a_failure_inside_a_200_is_an_error_view_without_the_response_text() {
    let view = poll(&enabled(), Some(&secret()), &serving("error_auth.json")).await;
    assert_eq!(view.state, ViewState::Error);
    assert_eq!(view.error.as_ref().map(|e| e.code), Some(ErrorCode::BadResponse));
    assert!(!serde_json::to_string(&view).unwrap().contains("fabricated"));
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
        assert!(view.facts.is_empty() && view.windows.is_empty());
        assert!(!serde_json::to_string(&view).unwrap().contains("fabricated"));
    }
}

#[tokio::test]
async fn the_key_reaches_only_the_authorization_header_of_the_one_request() {
    let server = serving("ok.json");
    poll(&enabled(), Some(&secret()), &server).await;
    let requests = server.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].header("authorization"), Some(format!("Bearer {KEY}").as_str()));
}
