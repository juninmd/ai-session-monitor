//! The requests against a loopback fake: where the key goes, and every way they can fail.

use std::time::{Duration, Instant};

use usage_model::{ErrorCode, HarnessError};
use usage_support::Secret;

use super::fake_server::{FakeServer, Reply, closed_port_url};
use super::{KEY, fixture, routed, secret};
use crate::remote::{Endpoint, Fetched, fetch};

async fn fetch_with(url: String, timeout: Duration) -> Result<Fetched, HarnessError> {
    fetch(&Endpoint::loopback(url, timeout), &secret()).await
}

async fn failure_of(server: &FakeServer) -> HarnessError {
    fetch_with(server.base_url(), Duration::from_secs(5)).await.unwrap_err()
}

fn serving(status: u16, body: &'static str) -> FakeServer {
    FakeServer::always(move || Reply::json(status, body))
}

#[tokio::test]
async fn two_gets_with_the_bearer_key_reach_the_injected_url_and_nowhere_else() {
    let server = routed((200, "key_limited.json"), (200, "credits_ok.json"));
    let decoy = routed((200, "key_limited.json"), (200, "credits_ok.json"));
    let fetched = fetch_with(server.base_url(), Duration::from_secs(5)).await.unwrap();
    assert!(fetched.credits.is_some());

    let requests = server.requests();
    let mut targets: Vec<&str> = requests.iter().map(|r| r.target.as_str()).collect();
    targets.sort_unstable();
    assert_eq!(targets, ["/api/v1/credits", "/api/v1/key"]);
    for request in &requests {
        assert_eq!(request.method, "GET");
        assert_eq!(request.header("authorization"), Some(format!("Bearer {KEY}").as_str()));
        assert!(request.header("cookie").is_none());
        assert!(!request.target.contains(KEY) && !request.target.contains('?'));
        for (name, value) in &request.headers {
            assert_eq!(value.contains(KEY), name.eq_ignore_ascii_case("authorization"), "{name}");
        }
    }
    decoy.assert_untouched();
}

#[tokio::test]
async fn an_ordinary_key_is_refused_by_credits_and_the_poll_still_succeeds() {
    let server = routed((200, "key_limited.json"), (403, "credits_forbidden.json"));
    let fetched = fetch_with(server.base_url(), Duration::from_secs(5)).await.unwrap();
    assert!(fetched.credits.is_none());
    assert_eq!(fetched.key.limit, Some(10.0));
}

#[tokio::test]
async fn a_credits_failure_of_any_kind_only_leaves_the_balance_unknown() {
    for status in [401, 402, 429, 500] {
        let server = routed((200, "key_limited.json"), (status, "credits_forbidden.json"));
        let fetched = fetch_with(server.base_url(), Duration::from_secs(5)).await.unwrap();
        assert!(fetched.credits.is_none(), "{status}");
    }
}

#[tokio::test]
async fn unauthorized_on_the_key_document_carries_no_response_text_and_no_key() {
    for status in [401, 403] {
        let server = FakeServer::always(move || Reply::json(status, &fixture("error_auth.json")));
        let error = failure_of(&server).await;
        assert_eq!((error.code, error.retryable), (ErrorCode::Unauthorized, false), "{status}");
        assert_eq!(error.params["httpStatus"], status.to_string());
        assert!(!format!("{error:?}").contains(KEY));
        assert!(!format!("{error:?}").contains("fabricated"));
        assert_eq!(server.requests().len(), 1, "a refused key is not retried on credits");
    }
}

#[tokio::test]
async fn payment_required_is_a_bad_response_that_names_the_status() {
    let error = failure_of(&serving(402, r#"{"error":{"message":"fabricated: key limit"}}"#)).await;
    assert_eq!((error.code, error.retryable), (ErrorCode::BadResponse, false));
    assert_eq!(error.params["httpStatus"], "402");
    assert!(!format!("{error:?}").contains("fabricated"));
}

#[tokio::test]
async fn rate_limit_carries_the_retry_after_it_was_given() {
    let server = FakeServer::always(|| Reply::with_headers(429, &[("Retry-After", "120")], "{}"));
    let error = failure_of(&server).await;
    assert_eq!((error.code, error.retryable), (ErrorCode::RateLimited, true));
    assert_eq!(error.params["retryAfterSecs"], "120");
}

#[tokio::test]
async fn server_errors_are_retryable_and_a_missing_route_means_the_endpoint_changed() {
    for status in [500, 503] {
        let error = failure_of(&serving(status, "{}")).await;
        assert_eq!((error.code, error.retryable), (ErrorCode::ServerError, true), "{status}");
    }
    let gone = failure_of(&serving(404, "{}")).await;
    assert_eq!((gone.code, gone.retryable), (ErrorCode::EndpointChanged, false));
}

#[tokio::test]
async fn non_json_and_foreign_documents_are_refused() {
    assert_eq!(failure_of(&serving(200, "not json")).await.code, ErrorCode::BadResponse);
    for body in [r#"{"hello":1}"#, "[]", "null"] {
        let code = failure_of(&serving(200, body)).await.code;
        assert_eq!(code, ErrorCode::EndpointChanged, "{body}");
    }
}

#[tokio::test]
async fn an_oversized_body_is_refused_declared_or_streamed() {
    let padded = format!(r#"{{"data":{{"usage":1}},"pad":"{}"}}"#, "x".repeat(70 * 1024));
    let declared = FakeServer::always(move || Reply::json(200, &padded));
    assert_eq!(failure_of(&declared).await.code, ErrorCode::ResponseTooLarge);
    let streamed = FakeServer::always(|| Reply::Streamed { bytes: 4 * 1024 * 1024 });
    assert_eq!(failure_of(&streamed).await.code, ErrorCode::ResponseTooLarge);
}

#[tokio::test]
async fn redirects_are_not_followed_so_the_key_stays_with_the_first_host() {
    let target = routed((200, "key_limited.json"), (200, "credits_ok.json"));
    let location = format!("{}/api/v1/key", target.base_url());
    let origin =
        FakeServer::always(move || Reply::with_headers(302, &[("Location", &location)], ""));
    let error = failure_of(&origin).await;
    assert_eq!(error.code, ErrorCode::BadResponse);
    assert_eq!(error.params["redirect"], "true");
    assert_eq!(origin.requests().len(), 1);
    target.assert_untouched();
}

#[tokio::test]
async fn a_slow_server_times_out_within_the_budget() {
    let server = FakeServer::always(|| {
        Reply::Delayed(Duration::from_secs(5), Box::new(Reply::json(200, "{}")))
    });
    let started = Instant::now();
    let error = fetch_with(server.base_url(), Duration::from_millis(300)).await.unwrap_err();
    assert_eq!((error.code, error.retryable), (ErrorCode::Timeout, true));
    assert!(started.elapsed() < Duration::from_secs(3));
}

// Windows retries SYNs to a closed loopback port for ~2 s before refusing; a shorter budget reads as a timeout.
#[tokio::test]
async fn an_unreachable_or_dropped_connection_is_a_retryable_network_error() {
    let refused = fetch_with(closed_port_url(), Duration::from_secs(10)).await.unwrap_err();
    assert_eq!((refused.code, refused.retryable), (ErrorCode::Network, true));
    let dropped = failure_of(&FakeServer::always(|| Reply::Hangup)).await;
    assert_eq!((dropped.code, dropped.retryable), (ErrorCode::Network, true));
}

#[tokio::test]
async fn a_key_that_would_break_the_header_is_refused_before_any_connection() {
    let server = serving(200, "{}");
    let endpoint = Endpoint::loopback(server.base_url(), Duration::from_secs(2));
    for key in ["", "abc\r\nX-Injected: 1", "abc\ndef", "abc\0def"] {
        let error = fetch(&endpoint, &Secret::new(key)).await.unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidInput, "{key:?}");
    }
    server.assert_untouched();
}

#[tokio::test]
async fn plain_http_is_refused_before_any_connection_without_the_test_escape() {
    let server = serving(200, "{}");
    let error = fetch(&Endpoint::strict(server.base_url()), &secret()).await.unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidUrl);
    server.assert_untouched();
}
