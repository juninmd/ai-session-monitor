//! The requests against a loopback fake: where the key goes, and every way they can fail.

use std::time::{Duration, Instant};

use usage_model::{ErrorCode, HarnessError};
use usage_support::Secret;

use super::fake_server::{FakeServer, Reply, closed_port_url};
use super::{KEY, fixture, secret};
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
async fn the_key_reaches_the_injected_url_and_nowhere_else() {
    let server = FakeServer::always(|| Reply::json(200, &fixture("checklist_limited.json")));
    let decoy = FakeServer::always(|| Reply::json(200, &fixture("checklist_limited.json")));
    fetch_with(server.base_url(), Duration::from_secs(5)).await.unwrap();
    assert!(!server.requests().is_empty());
    for request in server.requests() {
        assert_eq!(request.header("authorization"), Some(format!("Bearer {KEY}").as_str()));
        assert!(request.header("cookie").is_none());
    }
    decoy.assert_untouched();
}

#[tokio::test]
async fn unauthorized_carries_no_response_text_and_no_key() {
    for status in [401, 403] {
        let server = FakeServer::always(move || Reply::json(status, &fixture("error_auth.json")));
        let error = failure_of(&server).await;
        assert_eq!((error.code, error.retryable), (ErrorCode::Unauthorized, false), "{status}");
        assert_eq!(error.params["httpStatus"], status.to_string());
        assert!(!format!("{error:?}").contains(KEY));
        assert!(!format!("{error:?}").contains("fabricated"));
    }
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
        assert_eq!(
            failure_of(&serving(200, body)).await.code,
            ErrorCode::EndpointChanged,
            "{body}"
        );
    }
}

#[tokio::test]
async fn an_oversized_body_is_refused_declared_or_streamed() {
    let padded = format!(r#"{{"stripe_balance":0,"pad":"{}"}}"#, "x".repeat(70 * 1024));
    let declared = FakeServer::always(move || Reply::json(200, &padded));
    assert_eq!(failure_of(&declared).await.code, ErrorCode::ResponseTooLarge);
    let streamed = FakeServer::always(|| Reply::Streamed { bytes: 4 * 1024 * 1024 });
    assert_eq!(failure_of(&streamed).await.code, ErrorCode::ResponseTooLarge);
}

#[tokio::test]
async fn redirects_are_not_followed_so_the_key_stays_with_the_first_host() {
    let target = FakeServer::always(|| Reply::json(200, &fixture("checklist_limited.json")));
    let location = format!("{}/payment/checklist", target.base_url());
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
