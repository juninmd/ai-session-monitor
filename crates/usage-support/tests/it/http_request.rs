//! `RequestSpec`: what a request carries, what it refuses to carry, and what Debug may show.

use serde_json::json;
use usage_model::ErrorCode;
use usage_support::{Auth, RequestSpec, Secret, TlsPolicy};

use crate::common::{Reply, TestServer, client};

async fn ok_server() -> TestServer {
    TestServer::start(|_| Reply::json(200, r#"{"ok":true}"#)).await
}

#[tokio::test]
async fn the_bearer_credential_and_default_headers_are_sent() {
    let server = ok_server().await;
    let key = Secret::new("sk-live-fabricated-0001");
    let request = RequestSpec::new(server.url("/a")).auth(Auth::new(&key));
    client().get_json(&request).await.unwrap();

    let seen = server.last_request();
    assert_eq!(seen.method, "GET");
    assert_eq!(seen.header("authorization"), Some("Bearer sk-live-fabricated-0001"));
    assert_eq!(seen.header("accept"), Some("application/json"));
    let agent = format!("ai-session-monitor/{}", env!("CARGO_PKG_VERSION"));
    assert_eq!(seen.header("user-agent"), Some(agent.as_str()));
    assert_eq!(seen.header("cookie"), None);
    assert_eq!(seen.header("referer"), None);
    // No compression is negotiated, so the size cap bounds what the caller decodes.
    assert_eq!(seen.header("accept-encoding"), None);
}

#[tokio::test]
async fn a_credential_can_travel_in_any_header_with_or_without_a_scheme() {
    let server = ok_server().await;
    let key = Secret::new("sk-live-fabricated-0002");
    let csrf = Secret::new("csrf-fabricated-0003");
    let request = RequestSpec::new(server.url("/b"))
        .auth(Auth { header: "X-Api-Key".into(), scheme: None, ..Auth::new(&key) })
        .secret_header("X-Codeium-Csrf-Token", &csrf)
        .header("X-Trace", "abc 123");
    client().get_json(&request).await.unwrap();

    let seen = server.last_request();
    assert_eq!(seen.header("x-api-key"), Some("sk-live-fabricated-0002"));
    assert_eq!(seen.header("authorization"), None);
    assert_eq!(seen.header("x-codeium-csrf-token"), Some("csrf-fabricated-0003"));
    assert_eq!(seen.header("x-trace"), Some("abc 123"));

    let token = Auth { scheme: Some("Token".into()), ..Auth::new(&key) };
    client().get_json(&RequestSpec::new(server.url("/c")).auth(token)).await.unwrap();
    assert_eq!(
        server.last_request().header("authorization"),
        Some("Token sk-live-fabricated-0002")
    );
}

#[tokio::test]
async fn a_custom_user_agent_replaces_the_default() {
    let server = ok_server().await;
    let options = usage_support::ClientOptions {
        user_agent: "my-agent/9".into(),
        ..crate::common::options()
    };
    let client = usage_support::SafeClient::new(options).unwrap();
    client.get_json(&RequestSpec::new(server.url("/"))).await.unwrap();
    assert_eq!(server.last_request().header("user-agent"), Some("my-agent/9"));
}

#[tokio::test]
async fn post_json_sends_the_document_with_its_content_type() {
    let server = ok_server().await;
    let body = json!({"nome": "João 🔑", "n": [1, 2], "nested": {"a": null}});
    let key = Secret::new("sk-fabricated-77");
    let request = RequestSpec::new(server.url("/rpc")).auth(Auth::new(&key));
    assert_eq!(client().post_json(&request, &body).await.unwrap(), json!({"ok": true}));

    let seen = server.last_request();
    assert_eq!(seen.method, "POST");
    assert_eq!(seen.header("content-type"), Some("application/json"));
    assert_eq!(seen.header("authorization"), Some("Bearer sk-fabricated-77"));
    assert_eq!(serde_json::from_slice::<serde_json::Value>(&seen.body).unwrap(), body);
}

#[tokio::test]
async fn bad_headers_are_refused_before_io_without_echoing_their_values() {
    let server = ok_server().await;
    let url = server.url("/");
    let empty = Secret::new("");
    let leaky = Secret::new("sk-leak\nmore");
    let one = Secret::new("k");
    let cases = [
        RequestSpec::new(&url).header("Bad Name", "v"),
        RequestSpec::new(&url).header("", "v"),
        RequestSpec::new(&url).header("X-A", "line\r\nX-Injected: 1"),
        RequestSpec::new(&url).header("X-A", "\u{7f}"),
        RequestSpec::new(&url).secret_header("X-Key", &leaky),
        RequestSpec::new(&url).header("Host", "evil.example"),
        RequestSpec::new(&url).header("content-length", "5"),
        RequestSpec::new(&url).header("Transfer-Encoding", "chunked"),
        RequestSpec::new(&url).header("Connection", "upgrade"),
        RequestSpec::new(&url).header("Content-Type", "text/plain"),
        RequestSpec::new(&url).auth(Auth::new(&empty)),
        RequestSpec::new(&url).auth(Auth { header: "Host".into(), ..Auth::new(&one) }),
    ];
    for request in cases {
        let error = client().get_json(&request).await.unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidInput, "{request:?}");
        assert!(error.params.is_empty());
        assert!(!format!("{error:?} {request:?}").contains("sk-leak"));
    }
    server.assert_untouched().await;
}

#[test]
fn request_and_client_debug_output_hides_values() {
    let key = Secret::new("sk-live-fabricated-0004");
    let csrf = Secret::new("csrf-secret-2");
    let request = RequestSpec::new("https://h.example/p?api_key=query-secret-1")
        .auth(Auth::new(&key))
        .secret_header("X-Csrf", &csrf)
        .header("X-Trace", "trace-visible-3")
        .query("token", "query-secret-4")
        .segment("seg");
    let text = format!("{request:?} {request:#?} {:?}", client());
    for hidden in ["sk-live-fabricated", "csrf-secret", "query-secret", "trace-visible"] {
        assert!(!text.contains(hidden), "{hidden} leaked: {text}");
    }
    assert!(text.contains("authorization") && text.contains("x-csrf"), "{text}");
    assert!(format!("{:?}", TlsPolicy::CustomCa(vec![1, 2, 3])).contains("3 bytes"));
}
