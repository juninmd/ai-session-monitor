//! What the user sees, stores or logs never holds the token or the account details the server
//! returns, even when a hostile server quotes them back.

use usage_model::ViewState;

use super::*;
use crate::fake::Reply;

/// Values that must never leave the collector: the token, and the name, e-mail and tier id that
/// `status_ide.json` carries next to the quota.
const PRIVATE: [&str; 4] = [TOKEN, "fixture@example.invalid", "Fixture User", "fixture-pro-tier"];

/// The view as the UI receives it (serialized) and as a log line would print it (`Debug`).
fn assert_clean(view: &HarnessView, case: &str) {
    let shown = format!("{view:?} {}", serde_json::to_string(view).expect("views serialize"));
    for secret in PRIVATE {
        assert!(!shown.contains(secret), "{case}: a private value reached the view");
    }
}

/// A server that quotes the CSRF token it was sent back to the caller, in every way it can.
fn echoing(case: &str, token: &str) -> Reply {
    match case {
        "unauthorized" => Reply::json(401, &format!(r#"{{"message":"bad token {token}"}}"#)),
        "server_error" => Reply::json(500, &format!(r#"{{"message":"{token} broke it"}}"#)),
        "html" => Reply::Html(200, format!("<p>{token}</p>")),
        "garbage" => Reply::json(200, &format!("not json {token}")),
        "wrong_shape" => Reply::json(200, &format!(r#"{{"unexpected":"{token}"}}"#)),
        "envelope" => Reply::json(200, &format!(r#"{{"code":"{token}","message":"{token}"}}"#)),
        _ => Reply::Hangup,
    }
}

#[tokio::test]
async fn a_good_poll_shows_the_quota_and_none_of_the_account() {
    let server = serve("status_ide", Some("summary_full")).await;
    let view = poll(vec![server.target(TOKEN)]).await;
    assert_eq!(view.state, ViewState::Ready);
    assert_clean(&view, "success");
    // The tier name is the plan; nothing else of the account is displayed.
    assert!(view.meta.is_empty() && view.facts.is_empty());
}

#[tokio::test]
async fn a_server_that_quotes_the_token_back_is_not_quoted_in_turn() {
    for case in
        ["unauthorized", "server_error", "html", "garbage", "wrong_shape", "envelope", "hangup"]
    {
        let server = FakeServer::http(move |request| {
            echoing(case, request.header("x-codeium-csrf-token").unwrap_or_default())
        })
        .await;
        let view = poll(vec![server.target(TOKEN)]).await;
        assert_eq!(view.state, ViewState::Error, "{case}");
        assert_clean(&view, case);
    }
}

#[tokio::test]
async fn a_status_name_the_server_invents_is_not_repeated_in_the_error() {
    let server = FakeServer::http(|request| {
        echoing("envelope", request.header("x-codeium-csrf-token").unwrap_or_default())
    })
    .await;
    let view = poll(vec![server.target(TOKEN)]).await;
    let error = view.error.as_ref().expect("the envelope is an error");
    assert!(error.params.is_empty(), "{:?}", error.params.keys().collect::<Vec<_>>());
}

#[tokio::test]
async fn a_rejected_token_is_never_part_of_the_error() {
    let server = serve("status_ide", Some("summary_full")).await;
    let view = poll(vec![server.target("rejected-token-0002")]).await;
    let shown = format!("{view:?} {}", serde_json::to_string(&view).expect("views serialize"));
    assert!(!shown.contains("rejected-token-0002"));
    assert_eq!(view.state, ViewState::Error);
}

#[tokio::test]
async fn the_token_travels_in_a_header_and_never_in_the_url_or_the_body() {
    let server = serve("status_ide", Some("summary_full")).await;
    poll(vec![server.target(TOKEN)]).await;
    let requests = server.requests();
    assert_eq!(requests.len(), 2);
    for request in requests {
        assert!(!request.path.contains(TOKEN) && !request.path.contains('?'));
        assert!(!request.body.contains(TOKEN));
        assert_eq!(request.header("x-codeium-csrf-token"), Some(TOKEN));
    }
}
