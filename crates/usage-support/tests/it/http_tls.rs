//! `SafeClient`: certificate verification under each `TlsPolicy`, against throwaway TLS servers
//! on loopback. Every rejection is paired with a control that is accepted, so a pass cannot come
//! from a broken server.

use std::sync::Arc;
use std::time::{Duration, Instant};

use rustls::ServerConfig;
use serde_json::json;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use usage_model::{ErrorCode, HarnessError};
use usage_support::{Auth, ClientOptions, RequestSpec, SafeClient, Secret, TlsPolicy};

use crate::common::tls::{expired_by_new_ca, issued_by_new_ca, self_signed};
use crate::common::{Reply, TestServer, options};

fn with(tls: TlsPolicy) -> SafeClient {
    SafeClient::new(ClientOptions { tls, ..options() }).unwrap()
}

fn trusting(ca_pem: &str) -> SafeClient {
    with(TlsPolicy::CustomCa(ca_pem.as_bytes().to_vec()))
}

fn insecure(host: &str) -> SafeClient {
    with(TlsPolicy::Insecure { host: host.into() })
}

async fn https(config: Arc<ServerConfig>) -> TestServer {
    TestServer::start_tls(config, |_| Reply::json(200, r#"{"secure":true}"#)).await
}

async fn fetch(client: &SafeClient, url: String) -> Result<serde_json::Value, HarnessError> {
    // The credential must never leave the process unless the server was verified.
    let key = Secret::new("sk-live-fabricated-tls");
    client.get_json(&RequestSpec::new(url).auth(Auth::new(&key))).await
}

fn assert_tls_refusal(error: &HarnessError, server: &TestServer) {
    assert_eq!(error.code, ErrorCode::TlsError, "{error:?}");
    assert!(!error.retryable);
    assert_eq!(error.params.get("host").map(String::as_str), Some("127.0.0.1"));
    assert!(server.requests().is_empty(), "a request reached an unverified server");
}

#[tokio::test]
async fn a_custom_ca_trusts_servers_it_issued_by_address_and_by_name() {
    let pki = issued_by_new_ca(&["127.0.0.1", "localhost"]);
    let server = https(pki.server).await;
    let client = trusting(&pki.ca_pem);
    assert_eq!(fetch(&client, server.url("/")).await.unwrap(), json!({"secure": true}));
    let by_name = format!("https://localhost:{}/", server.addr.port());
    assert_eq!(fetch(&client, by_name).await.unwrap(), json!({"secure": true}));
    assert_eq!(
        server.last_request().header("authorization"),
        Some("Bearer sk-live-fabricated-tls")
    );
}

#[tokio::test]
async fn a_custom_ca_bundle_may_hold_several_roots_and_leading_comments() {
    let (first, second) = (issued_by_new_ca(&["127.0.0.1"]), issued_by_new_ca(&["127.0.0.1"]));
    let bundle = format!("# corporate roots\n{}\n{}", first.ca_pem, second.ca_pem);
    let client = trusting(&bundle);
    for pki in [first, second] {
        let server = https(pki.server).await;
        assert!(fetch(&client, server.url("/")).await.is_ok());
    }
}

#[tokio::test]
async fn a_custom_ca_rejects_a_server_from_another_ca() {
    let (served, trusted) = (issued_by_new_ca(&["127.0.0.1"]), issued_by_new_ca(&["127.0.0.1"]));
    let server = https(served.server).await;
    let error = fetch(&trusting(&trusted.ca_pem), server.url("/")).await.unwrap_err();
    assert_tls_refusal(&error, &server);
    // Control: the same server is fine for a client that trusts its CA.
    assert!(fetch(&trusting(&served.ca_pem), server.url("/")).await.is_ok());
}

#[tokio::test]
async fn a_custom_ca_still_checks_the_host_name() {
    let pki = issued_by_new_ca(&["other.example"]);
    let server = https(pki.server).await;
    let error = fetch(&trusting(&pki.ca_pem), server.url("/")).await.unwrap_err();
    assert_tls_refusal(&error, &server);
}

#[tokio::test]
async fn an_expired_certificate_is_rejected_even_from_a_trusted_ca() {
    let pki = expired_by_new_ca(&["127.0.0.1"]);
    let server = https(pki.server).await;
    let error = fetch(&trusting(&pki.ca_pem), server.url("/")).await.unwrap_err();
    assert_tls_refusal(&error, &server);
    // Insecure mode knowingly skips validation, so the expired certificate passes there.
    assert!(fetch(&insecure("127.0.0.1"), server.url("/")).await.is_ok());
}

#[tokio::test]
async fn the_system_store_rejects_certificates_it_does_not_know() {
    let unknown = issued_by_new_ca(&["127.0.0.1"]);
    let server = https(unknown.server).await;
    let error = fetch(&with(TlsPolicy::System), server.url("/")).await.unwrap_err();
    assert_tls_refusal(&error, &server);

    let self_signed = https(self_signed(&["127.0.0.1"])).await;
    let error = fetch(&with(TlsPolicy::System), self_signed.url("/")).await.unwrap_err();
    assert_tls_refusal(&error, &self_signed);
}

#[tokio::test]
async fn insecure_mode_accepts_an_unverifiable_certificate_for_its_host_only() {
    let server = https(self_signed(&["other.example"])).await;
    let client = insecure("127.0.0.1");
    assert_eq!(fetch(&client, server.url("/")).await.unwrap(), json!({"secure": true}));

    // Same server, other name: refused locally, so the connection count does not move.
    let before = server.connections();
    let by_name = format!("https://localhost:{}/", server.addr.port());
    let error = fetch(&client, by_name).await.unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidUrl);
    assert_eq!(error.params.get("host").map(String::as_str), Some("localhost"));
    assert_eq!(server.connections(), before);
}

#[tokio::test]
async fn a_plain_http_server_behind_an_https_url_is_a_tls_error() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        while let Ok((mut tcp, _)) = listener.accept().await {
            let _ = tcp.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{}").await;
            let _ = tcp.flush().await;
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    });
    let error =
        fetch(&with(TlsPolicy::System), format!("https://127.0.0.1:{port}/")).await.unwrap_err();
    assert_eq!(error.code, ErrorCode::TlsError, "{error:?}");
    assert!(!error.retryable);
}

#[tokio::test]
async fn a_handshake_that_never_completes_hits_the_deadline() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        let mut held = Vec::new();
        while let Ok((tcp, _)) = listener.accept().await {
            held.push(tcp);
        }
    });
    let client =
        SafeClient::new(ClientOptions { timeout: Duration::from_millis(400), ..options() })
            .unwrap();
    let started = Instant::now();
    let error = fetch(&client, format!("https://127.0.0.1:{port}/")).await.unwrap_err();
    assert_eq!(error.code, ErrorCode::Timeout, "{error:?}");
    assert!(error.retryable);
    assert!(started.elapsed() < Duration::from_secs(3), "{:?}", started.elapsed());
}

#[tokio::test]
async fn a_verified_server_that_hangs_up_is_a_retryable_network_error() {
    let pki = issued_by_new_ca(&["127.0.0.1"]);
    let server = TestServer::start_tls(pki.server, |_| Reply::Hangup).await;
    let error = fetch(&trusting(&pki.ca_pem), server.url("/")).await.unwrap_err();
    assert_eq!(error.code, ErrorCode::Network, "{error:?}");
    assert!(error.retryable);
}
