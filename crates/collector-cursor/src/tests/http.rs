//! The usage-summary request against a loopback fake: where the cookie goes, and every way it
//! can fail.

use std::time::{Duration, Instant};

use usage_model::{ErrorCode, HarnessError};
use usage_support::Secret;

use super::fake_server::{FakeServer, Reply, closed_port_url};
use super::{NOW, fixture, token};
use crate::jwt;
use crate::parse::{Parsed, parse_summary};
use crate::remote::{Endpoint, fetch};

fn cookie() -> Secret {
    jwt::cookie(&token("valid"), NOW).unwrap()
}

async fn fetch_with(url: String, timeout: Duration) -> Result<Parsed, HarnessError> {
    fetch(&Endpoint::loopback(url, timeout), &cookie(), NOW).await
}

async fn fetch_from(server: &FakeServer) -> Result<Parsed, HarnessError> {
    fetch_with(server.base_url(), Duration::from_secs(5)).await
}

async fn failure_of(server: &FakeServer) -> HarnessError {
    fetch_from(server).await.unwrap_err()
}

fn serving(status: u16, body: &'static str) -> FakeServer {
    FakeServer::always(move || Reply::json(status, body))
}

#[tokio::test]
async fn one_cookie_get_reaches_the_injected_url_and_nowhere_else() {
    let server = FakeServer::always(|| Reply::json(200, &fixture("ok_pro.json")));
    let decoy = FakeServer::always(|| Reply::json(200, &fixture("ok_pro.json")));
    assert_eq!(fetch_from(&server).await.unwrap().windows.len(), 1);

    let requests = server.requests();
    assert_eq!(requests.len(), 1, "exactly one request per read");
    let request = &requests[0];
    assert_eq!(request.method, "GET");
    assert_eq!(request.target, "/api/usage-summary");
    assert_eq!(request.header("cookie"), Some(cookie().expose()));
    assert_eq!(request.header("accept"), Some("application/json"));
    assert!(request.header("authorization").is_none());
    assert!(request.header("user-agent").is_some_and(|ua| ua.starts_with("ai-session-monitor/")));
    decoy.assert_untouched();
}

#[tokio::test]
async fn the_cookie_travels_only_in_the_cookie_header() {
    let server = FakeServer::always(|| Reply::json(200, &fixture("ok_pro.json")));
    fetch_from(&server).await.unwrap();
    let request = server.requests().remove(0);
    let jwt = token("valid");
    assert!(!request.target.contains(&jwt) && !request.target.contains("user_01"));
    for (name, value) in &request.headers {
        let carries = value.contains(&jwt) || value.contains("user_01FABCURSOR");
        assert_eq!(carries, name.eq_ignore_ascii_case("cookie"), "{name}");
    }
}

#[tokio::test]
async fn unauthorized_and_forbidden_are_ambiguous_never_a_certain_bad_session() {
    for status in [401, 403] {
        let server = FakeServer::always(move || Reply::json(status, &fixture("error_auth.json")));
        let error = failure_of(&server).await;
        assert_eq!(error.code, ErrorCode::Unauthorized, "{status}");
        assert!(!error.retryable);
        assert_eq!(error.params["ambiguous"], "true");
        assert_eq!(error.params["httpStatus"], status.to_string());
        let debug = format!("{error:?}");
        assert!(!debug.contains(&token("valid")) && !debug.contains("user_01FABCURSOR"));
        assert!(!debug.contains("fabricated"), "no response text in the error");
    }
}

#[tokio::test]
async fn rate_limit_carries_the_retry_after_it_was_given() {
    let cases = [
        (Some("120"), Some("120")),
        (Some("99999999999"), Some("3600")),
        (Some("Wed, 21 Oct 2026 07:28:00 GMT"), None),
        (None, None),
    ];
    for (header, want) in cases {
        let headers: Vec<(&str, &str)> = header.map(|v| ("Retry-After", v)).into_iter().collect();
        let server = FakeServer::always(move || Reply::with_headers(429, &headers, "{}"));
        let error = failure_of(&server).await;
        assert_eq!(error.code, ErrorCode::RateLimited);
        assert!(error.retryable);
        assert_eq!(error.params.get("retryAfterSecs").map(String::as_str), want, "{header:?}");
    }
}

#[tokio::test]
async fn server_errors_are_retryable() {
    for status in [500, 502, 503] {
        let error = failure_of(&serving(status, "{}")).await;
        assert_eq!(error.code, ErrorCode::ServerError, "{status}");
        assert!(error.retryable);
    }
}

#[tokio::test]
async fn a_missing_route_means_the_endpoint_changed() {
    let error = failure_of(&serving(404, r#"{"error":"not found"}"#)).await;
    assert_eq!(error.code, ErrorCode::EndpointChanged);
    assert!(!error.retryable);
    assert_eq!(error.params["httpStatus"], "404");
}

#[tokio::test]
async fn html_and_non_json_bodies_are_a_bad_response() {
    let html = FakeServer::always(|| {
        Reply::with_headers(200, &[("Content-Type", "text/html")], "<html>sign in</html>")
    });
    assert_eq!(failure_of(&html).await.code, ErrorCode::BadResponse);
    assert_eq!(failure_of(&serving(200, "this is not json")).await.code, ErrorCode::BadResponse);
    assert_eq!(failure_of(&serving(200, "")).await.code, ErrorCode::BadResponse);
    assert_eq!(
        failure_of(&serving(200, r#"{"billingCycleEnd": "#)).await.code,
        ErrorCode::BadResponse
    );
}

#[tokio::test]
async fn json_that_is_not_the_summary_is_endpoint_changed() {
    for body in [r#"{"hello":1}"#, "[]", "null", r#"{"individualUsage":{}}"#] {
        let error = failure_of(&serving(200, body)).await;
        assert_eq!((error.code, error.retryable), (ErrorCode::EndpointChanged, false), "{body}");
    }
}

#[tokio::test]
async fn an_oversized_body_is_refused_declared_or_streamed() {
    let padded = format!(r#"{{"membershipType":"pro","pad":"{}"}}"#, "x".repeat(70 * 1024));
    let declared = FakeServer::always(move || Reply::json(200, &padded));
    let error = failure_of(&declared).await;
    assert_eq!((error.code, error.retryable), (ErrorCode::ResponseTooLarge, false));

    let streamed = FakeServer::always(|| Reply::Streamed { bytes: 4 * 1024 * 1024 });
    assert_eq!(failure_of(&streamed).await.code, ErrorCode::ResponseTooLarge);
}

#[tokio::test]
async fn redirects_are_not_followed_so_the_cookie_stays_with_the_first_host() {
    let target = FakeServer::always(|| Reply::json(200, &fixture("ok_pro.json")));
    let location = format!("{}/api/usage-summary", target.base_url());
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
    assert!(started.elapsed() < Duration::from_secs(3), "{:?}", started.elapsed());
}

// Windows retries SYNs to a closed loopback port for ~2 s before refusing; a shorter budget reads as a timeout.
#[tokio::test]
async fn an_unreachable_or_dropped_connection_is_a_retryable_network_error() {
    let refused = fetch_with(closed_port_url(), Duration::from_secs(10)).await.unwrap_err();
    assert_eq!((refused.code, refused.retryable), (ErrorCode::Network, true));

    let hangup = FakeServer::always(|| Reply::Hangup);
    let dropped = failure_of(&hangup).await;
    assert_eq!((dropped.code, dropped.retryable), (ErrorCode::Network, true));
}

#[tokio::test]
async fn a_cookie_that_would_break_the_header_is_refused_before_any_connection() {
    let server = serving(200, "{}");
    let endpoint = Endpoint::loopback(server.base_url(), Duration::from_secs(2));
    for value in ["a=b\r\nX-Injected: 1", "a=b\ncd", "a=b\0cd"] {
        let error = fetch(&endpoint, &Secret::new(value), NOW).await.unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidInput, "{value:?}");
    }
    server.assert_untouched();
}

#[tokio::test]
async fn plain_http_is_refused_before_any_connection_without_the_test_escape() {
    let server = serving(200, "{}");
    let endpoint = Endpoint::strict(server.base_url());
    let error = fetch(&endpoint, &cookie(), NOW).await.unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidUrl);
    server.assert_untouched();
}

#[tokio::test]
async fn every_fixture_served_over_http_gives_what_the_parser_gives() {
    for name in [
        "ok_pro.json",
        "ok_on_demand.json",
        "unlimited.json",
        "hostile.json",
        "wrong_types.json",
        "expired_cycle.json",
        "not_an_object.json",
    ] {
        let server = FakeServer::always(move || Reply::json(200, &fixture(name)));
        let body: serde_json::Value = serde_json::from_str(&fixture(name)).unwrap();
        match (parse_summary(&body, NOW), fetch_from(&server).await) {
            (Ok(direct), Ok(served)) => assert_eq!(direct.windows, served.windows, "{name}"),
            (Err(direct), Err(served)) => assert_eq!(direct, served, "{name}"),
            (direct, served) => panic!("{name}: parsed {direct:?} but served {served:?}"),
        }
    }
}
