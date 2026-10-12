//! A poll that fails below the protocol: size cap, deadlines, dropped or garbled connections.

use std::time::Instant;

use usage_model::ErrorCode;

use super::failures::{against, failed, kind, params};
use super::*;
use crate::fake::Reply;

#[tokio::test]
async fn a_response_over_the_size_cap_is_refused() {
    let limits = Limits { request_timeout: Duration::from_secs(3), max_body_bytes: 1_024 };
    let config = Config { limits, budget: Duration::from_secs(20) };
    let big = format!(r#"{{"padding":"{}"}}"#, "x".repeat(4_096));
    let server = FakeServer::http(move |_| Reply::json(200, &big)).await;
    let discovery = StaticDiscovery::new(Ok(vec![server.target(TOKEN)]));
    let view = poll_with(&on(), &discovery, &config).await;
    assert_eq!(kind(failed(&view)), (ErrorCode::ResponseTooLarge, false));
}

#[tokio::test]
async fn a_server_that_never_answers_is_a_timeout_worth_retrying() {
    let late = Reply::After(Duration::from_secs(30), Box::new(Reply::status(200)));
    for reply in [Reply::Silent, late] {
        let server = FakeServer::http(move |_| reply.clone()).await;
        let discovery = StaticDiscovery::new(Ok(vec![server.target(TOKEN)]));
        let config = timed(Duration::from_millis(300), Duration::from_secs(20));
        let started = Instant::now();
        let view = poll_with(&on(), &discovery, &config).await;
        let error = failed(&view);
        assert_eq!(kind(error), (ErrorCode::Timeout, true));
        assert_eq!(params(error), [("host", "127.0.0.1")]);
        assert!(started.elapsed() < Duration::from_secs(10), "the request deadline must hold");
    }
}

#[tokio::test]
async fn the_overall_budget_ends_a_poll_that_keeps_waiting() {
    let server = FakeServer::http(|_| Reply::Silent).await;
    let discovery = StaticDiscovery::new(Ok(vec![server.target(TOKEN)]));
    let config = timed(Duration::from_secs(30), Duration::from_millis(300));
    let started = Instant::now();
    let view = poll_with(&on(), &discovery, &config).await;
    let error = failed(&view);
    assert_eq!(kind(error), (ErrorCode::Timeout, true));
    assert!(error.params.is_empty());
    assert!(started.elapsed() < Duration::from_secs(10), "the budget must hold");
}

#[tokio::test]
async fn a_server_that_hangs_up_is_a_network_error_worth_retrying() {
    let view = against(Reply::Hangup).await;
    let error = failed(&view);
    assert_eq!(kind(error), (ErrorCode::Network, true));
    assert_eq!(params(error), [("host", "127.0.0.1")]);
}

#[tokio::test]
async fn bytes_that_are_not_http_are_a_network_error_too() {
    let view = against(Reply::Raw(b"\x00\x01 not http at all \r\n\r\n".to_vec())).await;
    assert_eq!(kind(failed(&view)), (ErrorCode::Network, true));
}
