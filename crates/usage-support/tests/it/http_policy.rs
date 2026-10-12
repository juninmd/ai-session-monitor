//! `SafeClient`: which requests are allowed to leave, and the proof that a refused one sends nothing.

use serde_json::json;
use usage_model::{ErrorCode, HarnessError};
use usage_support::{ClientOptions, RequestSpec, SafeClient, TlsPolicy};

use crate::common::{Reply, TestServer, client, options};

async fn ok_server() -> TestServer {
    TestServer::start(|_| Reply::json(200, r#"{"ok":true}"#)).await
}

fn insecure(host: &str) -> SafeClient {
    let tls = TlsPolicy::Insecure { host: host.into() };
    SafeClient::new(ClientOptions { tls, ..options() }).unwrap()
}

async fn refused(server: &TestServer, client: &SafeClient, request: RequestSpec) -> HarnessError {
    let error = client.get_json(&request).await.unwrap_err();
    server.assert_untouched().await;
    error
}

#[tokio::test]
async fn query_and_segments_reach_the_server_encoded_under_the_path_prefix() {
    let server = ok_server().await;
    let request = RequestSpec::new(server.url("/prefix/"))
        .segment("team")
        .segment("a b/c")
        .segment("é")
        .query("user id", "ana & bob")
        .query("q", "日本");
    client().get_json(&request).await.unwrap();
    assert_eq!(
        server.last_request().target,
        "/prefix/team/a%20b%2Fc/%C3%A9?user%20id=ana%20%26%20bob&q=%E6%97%A5%E6%9C%AC"
    );
}

#[tokio::test]
async fn dot_segments_cannot_redirect_the_request() {
    let server = ok_server().await;
    let request = RequestSpec::new(server.url("/api")).segment("..").segment("admin");
    let error = refused(&server, &client(), request).await;
    assert_eq!(error.code, ErrorCode::InvalidUrl);
}

#[tokio::test]
async fn plain_http_to_loopback_needs_the_flag() {
    let server = ok_server().await;
    let strict = SafeClient::new(ClientOptions { allow_loopback_http: false, ..options() });
    let error = refused(&server, &strict.unwrap(), RequestSpec::new(server.url("/"))).await;
    assert_eq!(error.code, ErrorCode::InvalidUrl);
    let allowed = client().get_json(&RequestSpec::new(server.url("/"))).await;
    assert_eq!(allowed.unwrap(), json!({"ok": true}));
}

#[tokio::test]
async fn plain_http_to_other_hosts_is_refused_without_io_even_with_the_flag() {
    let server = ok_server().await;
    let port = server.addr.port();
    for url in [
        format!("http://localhost.example.test:{port}/"),
        format!("http://127.0.0.1.example.test:{port}/"),
        format!("http://0.0.0.0:{port}/"),
        format!("http://[::ffff:8.8.8.8]:{port}/"),
        format!("http://example.test:{port}/"),
    ] {
        let error = client().get_json(&RequestSpec::new(url.clone())).await.unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidUrl, "{url}");
    }
    server.assert_untouched().await;
}

#[tokio::test]
async fn localhost_by_name_is_forced_to_loopback() {
    let server = ok_server().await;
    let url = format!("http://localhost:{}/", server.addr.port());
    assert!(client().get_json(&RequestSpec::new(url)).await.is_ok());
    assert_eq!(server.requests().len(), 1);
}

#[tokio::test]
async fn userinfo_fragments_and_foreign_schemes_are_refused_before_io() {
    let server = ok_server().await;
    let port = server.addr.port();
    for url in [
        format!("http://user:pass@127.0.0.1:{port}/"),
        format!("http://127.0.0.1:{port}/#frag"),
        format!("ftp://127.0.0.1:{port}/"),
        format!("http://127.0.0.1:{port}/\r\nX-Injected: 1"),
    ] {
        let error = client().get_json(&RequestSpec::new(url.clone())).await.unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidUrl, "{url:?}");
    }
    server.assert_untouched().await;
}

#[tokio::test]
async fn insecure_mode_refuses_every_other_host_before_io() {
    let server = ok_server().await;
    // The target is a live loopback server; only the host rule can stop the request.
    let other = insecure("litellm.corp.example");
    let error = refused(&server, &other, RequestSpec::new(server.url("/"))).await;
    assert_eq!(error.code, ErrorCode::InvalidUrl);
    assert_eq!(error.params.get("host").map(String::as_str), Some("127.0.0.1"));

    let foreign = RequestSpec::new("https://evil.example/");
    let error = other.get_json(&foreign).await.unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidUrl);
    assert_eq!(error.params.get("host").map(String::as_str), Some("evil.example"));

    // A prefix, a suffix or a look-alike of the real host is not the host.
    for allowed in ["127.0.0.", "1.0.0.1", "127.0.0.1.example", "local", "127.0.0.10"] {
        let request = RequestSpec::new(server.url("/"));
        let error = insecure(allowed).get_json(&request).await.unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidUrl, "{allowed}");
    }
    server.assert_untouched().await;
}

#[tokio::test]
async fn insecure_mode_matches_the_host_case_insensitively_and_exactly() {
    let server = ok_server().await;
    for allowed in ["127.0.0.1", " 127.0.0.1 ", "127.0.0.1."] {
        let request = RequestSpec::new(server.url("/"));
        assert!(insecure(allowed).get_json(&request).await.is_ok(), "{allowed:?}");
    }
    let by_name = format!("http://localhost:{}/", server.addr.port());
    assert!(insecure("LocalHost").get_json(&RequestSpec::new(by_name)).await.is_ok());
}

#[test]
fn construction_rejects_unusable_tls_settings_and_user_agents() {
    let build = |tls| SafeClient::new(ClientOptions { tls, ..options() }).unwrap_err();
    for pem in [
        Vec::new(),
        b"not pem at all".to_vec(),
        b"-----BEGIN CERTIFICATE-----\n!!!not base64!!!\n-----END CERTIFICATE-----\n".to_vec(),
        b"-----BEGIN CERTIFICATE-----\nAAAA\n-----END CERTIFICATE-----\n".to_vec(),
        b"-----BEGIN PRIVATE KEY-----\nAAAA\n-----END PRIVATE KEY-----\n".to_vec(),
        vec![b'x'; 2 * 1024 * 1024],
    ] {
        let error = build(TlsPolicy::CustomCa(pem));
        assert_eq!(error.code, ErrorCode::TlsError);
        assert!(!error.retryable);
    }
    for host in ["", "  ", "."] {
        let error = build(TlsPolicy::Insecure { host: host.into() });
        assert_eq!(error.code, ErrorCode::TlsError, "{host:?}");
    }
    let bad_agent = SafeClient::new(ClientOptions { user_agent: "a\r\nb".into(), ..options() });
    assert_eq!(bad_agent.unwrap_err().code, ErrorCode::InvalidInput);
}
