//! Whole polls: database, token, request and view together, and the token staying out of every
//! view.

use usage_model::{ErrorCode, HarnessId, ViewState};

use super::fake_server::{FakeServer, Reply};
use super::{COOKIE, FakeDb, NOW, fixture, on, poll, token};

fn serving(name: &'static str) -> FakeServer {
    FakeServer::always(move || Reply::json(200, &fixture(name)))
}

fn signed_in() -> FakeDb {
    FakeDb::with_token(&token("valid"))
}

#[tokio::test]
async fn a_good_answer_is_a_ready_unofficial_view_stamped_with_the_poll_time() {
    let view = poll(&on(), &signed_in(), &serving("ok_pro.json")).await;
    assert_eq!(view.state, ViewState::Ready);
    assert_eq!(view.id, HarnessId::Cursor);
    let source = view.source.as_ref().unwrap();
    assert_eq!((source.id.as_str(), source.official), ("usage_summary", false));
    assert_eq!(view.read_at, Some(NOW));
    assert_eq!(view.plan.as_deref(), Some("Pro"));
    assert!(!view.stale);
    assert!(view.error.is_none() && view.notices.is_empty() && view.meta.is_empty());
    assert_eq!(view.windows.len(), 1);
}

#[tokio::test]
async fn the_cookie_sent_is_built_from_the_row_in_the_database() {
    let server = serving("ok_pro.json");
    poll(&on(), &signed_in(), &server).await;
    let requests = server.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].header("cookie"), Some(format!("{COOKIE}{}", token("valid")).as_str()));
}

#[tokio::test]
async fn the_unlimited_and_on_demand_shapes_reach_the_view() {
    let unlimited = poll(&on(), &signed_in(), &serving("unlimited.json")).await;
    assert_eq!(unlimited.state, ViewState::Ready);
    assert!(unlimited.windows.is_empty());
    assert_eq!(unlimited.facts.len(), 2);
    assert!(!unlimited.stale, "no windows, nothing to have expired");

    let spend = poll(&on(), &signed_in(), &serving("ok_on_demand.json")).await;
    assert_eq!(spend.windows.len(), 2);
}

#[tokio::test]
async fn a_cycle_that_already_ended_makes_the_view_stale() {
    let view = poll(&on(), &signed_in(), &serving("expired_cycle.json")).await;
    assert_eq!(view.state, ViewState::Ready);
    assert!(view.stale);
    let open_ended = FakeServer::always(|| {
        Reply::json(200, r#"{"individualUsage":{"plan":{"totalPercentUsed":3}}}"#)
    });
    assert!(!poll(&on(), &signed_in(), &open_ended).await.stale);
}

#[tokio::test]
async fn failures_become_error_views_without_the_token_or_the_response_text() {
    for (status, code) in [
        (401, ErrorCode::Unauthorized),
        (403, ErrorCode::Unauthorized),
        (404, ErrorCode::EndpointChanged),
        (429, ErrorCode::RateLimited),
        (500, ErrorCode::ServerError),
    ] {
        let server = FakeServer::always(move || Reply::json(status, &fixture("error_auth.json")));
        let view = poll(&on(), &signed_in(), &server).await;
        assert_eq!(view.state, ViewState::Error, "{status}");
        assert_eq!(view.error.as_ref().map(|e| e.code), Some(code), "{status}");
        assert!(view.windows.is_empty());
        assert!(!serde_json::to_string(&view).unwrap().contains("fabricated"));
    }
}

#[tokio::test]
async fn an_unreadable_summary_is_endpoint_changed_not_zeros() {
    for name in ["wrong_types.json", "not_an_object.json"] {
        let view = poll(&on(), &signed_in(), &serving(name)).await;
        assert_eq!(view.state, ViewState::Error, "{name}");
        assert_eq!(view.error.unwrap().code, ErrorCode::EndpointChanged, "{name}");
    }
}

#[tokio::test]
async fn a_locked_store_is_a_retryable_error_with_a_fixed_reason_and_no_request() {
    let db = signed_in();
    let writer = db.writer();
    writer.execute_batch("BEGIN EXCLUSIVE").unwrap();
    let server = serving("ok_pro.json");
    let view = poll(&on(), &db, &server).await;
    let error = view.error.unwrap();
    assert_eq!((error.code, error.retryable), (ErrorCode::Timeout, true));
    assert_eq!(error.params["reason"], "locked");
    writer.execute_batch("ROLLBACK").unwrap();
    server.assert_untouched();
}

#[tokio::test]
async fn nothing_from_the_neighbouring_rows_or_the_token_reaches_any_view() {
    // `poll` already scans every view for the token, the user id and the canary; this runs it
    // over the whole range of outcomes at once.
    let server = serving("ok_on_demand.json");
    for db in [signed_in(), FakeDb::with_token(&token("expired")), FakeDb::empty()] {
        poll(&on(), &db, &server).await;
    }
}
