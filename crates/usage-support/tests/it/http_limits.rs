//! `SafeClient`: body size caps, the hard deadline and transport failures.

use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::net::TcpListener;
use usage_model::{ErrorCode, HarnessError};
use usage_support::{ClientOptions, RequestSpec, SafeClient};

use crate::common::{Reply, TestServer, client, options};

fn capped(max_body_bytes: usize) -> SafeClient {
    SafeClient::new(ClientOptions { max_body_bytes, ..options() }).unwrap()
}

fn with_timeout(timeout: Duration) -> SafeClient {
    SafeClient::new(ClientOptions { timeout, ..options() }).unwrap()
}

async fn get(client: &SafeClient, server: &TestServer) -> Result<serde_json::Value, HarnessError> {
    client.get_json(&RequestSpec::new(server.url("/data"))).await
}

/// A JSON document of exactly `len` bytes.
fn json_of_len(len: usize) -> String {
    assert!(len >= 8);
    format!("{{\"a\":\"{}\"}}", "x".repeat(len - 8))
}

fn only_host(error: &HarnessError) -> bool {
    error.params.keys().map(String::as_str).collect::<Vec<_>>() == ["host"]
}

fn chunked(chunks: &[&[u8]], pause: Duration, endless: bool) -> Reply {
    let chunks = chunks.iter().map(|c| c.to_vec()).collect();
    Reply::Chunked { status: 200, chunks, pause, endless }
}

#[tokio::test]
async fn a_body_exactly_at_the_cap_is_accepted_and_one_byte_more_is_not() {
    for len in [8usize, 100, 1000] {
        let body = json_of_len(len);
        let server = TestServer::start(move |_| Reply::json(200, &body)).await;
        assert!(get(&capped(len), &server).await.is_ok(), "{len}");
        let error = get(&capped(len - 1), &server).await.unwrap_err();
        assert_eq!(error.code, ErrorCode::ResponseTooLarge, "{len}");
        assert!(!error.retryable);
        assert_eq!(error.params.get("host").map(String::as_str), Some("127.0.0.1"));
    }
}

#[tokio::test]
async fn the_cap_applies_to_chunked_bodies_that_declare_no_length() {
    let server = TestServer::start(|_| chunked(&[b"{\"a\":", b"1}"], Duration::ZERO, false)).await;
    assert_eq!(get(&capped(7), &server).await.unwrap(), serde_json::json!({"a": 1}));
    let error = get(&capped(6), &server).await.unwrap_err();
    assert_eq!(error.code, ErrorCode::ResponseTooLarge);
    assert!(only_host(&error));
}

#[tokio::test]
async fn an_endless_stream_is_cut_off_at_the_cap_and_the_connection_is_dropped() {
    let filler = vec![b'x'; 512];
    let server = TestServer::start(move |_| {
        chunked(&[b"{\"a\":\"", &filler], Duration::from_millis(2), true)
    })
    .await;
    let started = Instant::now();
    let error = get(&capped(4096), &server).await.unwrap_err();
    assert_eq!(error.code, ErrorCode::ResponseTooLarge);
    assert!(started.elapsed() < Duration::from_secs(4), "{:?}", started.elapsed());

    // The server must notice that the client left, after writing only a bounded amount.
    let mut stopped = false;
    for _ in 0..100 {
        stopped = server.streamed().1;
        if stopped {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    let (written, _) = server.streamed();
    assert!(stopped, "the client kept draining an endless body");
    assert!(written < 1024 * 1024, "{written} bytes were streamed");
}

#[tokio::test]
async fn a_huge_declared_length_is_refused_from_the_headers_alone() {
    let server =
        TestServer::start(|_| Reply::Truncated { declared: usize::MAX / 2, sent: b"{}".to_vec() })
            .await;
    let error = get(&client(), &server).await.unwrap_err();
    assert_eq!(error.code, ErrorCode::ResponseTooLarge);
    assert!(!error.retryable);
}

#[tokio::test]
async fn degenerate_caps_neither_underflow_nor_overflow() {
    let server = TestServer::start(|_| Reply::json(200, "{}")).await;
    let error = get(&capped(0), &server).await.unwrap_err();
    assert_eq!(error.code, ErrorCode::ResponseTooLarge);
    assert!(get(&capped(2), &server).await.is_ok());
    assert!(get(&capped(usize::MAX), &server).await.is_ok());

    let empty = TestServer::start(|_| Reply::json(200, "")).await;
    let error = get(&capped(0), &empty).await.unwrap_err();
    assert_eq!(error.code, ErrorCode::BadResponse);
}

#[tokio::test]
async fn a_cap_beyond_the_hard_ceiling_is_clamped_to_it() {
    const CEILING: usize = 16 << 20;
    let block = vec![b'x'; 1 << 20];
    let server =
        TestServer::start(move |_| chunked(&[b"{\"a\":\"", &block], Duration::ZERO, true)).await;
    let patient =
        ClientOptions { max_body_bytes: usize::MAX, timeout: Duration::from_secs(30), ..options() };
    let error = get(&SafeClient::new(patient).unwrap(), &server).await.unwrap_err();
    assert_eq!(error.code, ErrorCode::ResponseTooLarge);

    let mut stopped = false;
    for _ in 0..100 {
        stopped = server.streamed().1;
        if stopped {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    let (written, _) = server.streamed();
    assert!(stopped, "the client kept draining an endless body");
    assert!(written >= CEILING, "the ceiling is lower than documented: {written} bytes");
    assert!(written < 4 * CEILING, "{written} bytes were streamed past the ceiling");
}

#[tokio::test]
async fn post_responses_obey_the_same_cap() {
    let body = json_of_len(500);
    let server = TestServer::start(move |_| Reply::json(200, &body)).await;
    let request = RequestSpec::new(server.url("/rpc"));
    let error = capped(100).post_json(&request, &serde_json::json!({})).await.unwrap_err();
    assert_eq!(error.code, ErrorCode::ResponseTooLarge);
}

#[tokio::test]
async fn the_deadline_covers_the_whole_call_not_each_read() {
    let server =
        TestServer::start(|_| chunked(&[b"{\"a\":", b" "], Duration::from_millis(100), true)).await;
    let started = Instant::now();
    let error = get(&with_timeout(Duration::from_millis(700)), &server).await.unwrap_err();
    let took = started.elapsed();
    assert_eq!(error.code, ErrorCode::Timeout);
    assert!(error.retryable);
    assert!(only_host(&error));
    assert!(took >= Duration::from_millis(600) && took < Duration::from_secs(4), "{took:?}");
}

#[tokio::test]
async fn a_server_that_never_answers_or_answers_late_times_out_retryably() {
    let silent = TestServer::start(|_| Reply::Silent).await;
    let late = TestServer::start(|_| {
        Reply::After(Duration::from_secs(5), Box::new(Reply::json(200, "{}")))
    })
    .await;
    for server in [&silent, &late] {
        let started = Instant::now();
        let error = get(&with_timeout(Duration::from_millis(300)), server).await.unwrap_err();
        assert_eq!(error.code, ErrorCode::Timeout);
        assert!(error.retryable);
        assert!(started.elapsed() < Duration::from_secs(3), "{:?}", started.elapsed());
    }
}

#[tokio::test]
async fn extreme_timeouts_neither_panic_nor_hang() {
    let server = TestServer::start(|_| Reply::json(200, "{}")).await;
    assert!(get(&with_timeout(Duration::MAX), &server).await.is_ok());
    let error = get(&with_timeout(Duration::ZERO), &server).await.unwrap_err();
    assert_eq!(error.code, ErrorCode::Timeout);
}

#[tokio::test]
async fn transport_failures_are_retryable_network_errors_that_name_only_the_host() {
    let huge_header = format!("HTTP/1.1 200 OK\r\nX-Pad: {}\r\n\r\n{{}}", "a".repeat(2 << 20));
    for case in 0..4 {
        let huge_header = huge_header.clone().into_bytes();
        let server = TestServer::start(move |_| match case {
            0 => Reply::Hangup,
            1 => Reply::Truncated { declared: 100, sent: b"{\"a\":".to_vec() },
            2 => Reply::Raw(b"this is not http\r\n\r\n".to_vec()),
            _ => Reply::Raw(huge_header.clone()),
        })
        .await;
        let error = get(&client(), &server).await.unwrap_err();
        assert_eq!(error.code, ErrorCode::Network, "case {case}");
        assert!(error.retryable, "case {case}");
        assert!(only_host(&error), "case {case}: {:?}", error.params);
    }
}

#[tokio::test]
async fn a_closed_port_is_a_retryable_network_error() {
    let port = {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        listener.local_addr().unwrap().port()
    };
    let request = RequestSpec::new(format!("http://127.0.0.1:{port}/"));
    let error = client().get_json(&request).await.unwrap_err();
    assert_eq!(error.code, ErrorCode::Network);
    assert!(error.retryable);
    assert!(only_host(&error));
}

#[tokio::test]
async fn one_client_serves_concurrent_calls() {
    let server = TestServer::start(|_| Reply::json(200, r#"{"ok":true}"#)).await;
    let shared = Arc::new(client());
    let tasks: Vec<_> = (0..16)
        .map(|n| {
            let (shared, url) = (Arc::clone(&shared), server.url(&format!("/n/{n}")));
            tokio::spawn(async move { shared.get_json(&RequestSpec::new(url)).await })
        })
        .collect();
    for task in tasks {
        assert!(task.await.unwrap().is_ok());
    }
    assert_eq!(server.requests().len(), 16);
}
