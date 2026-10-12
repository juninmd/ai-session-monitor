//! `SafeClient`: how HTTP statuses, content types and bodies map to results and error params.

use serde_json::json;
use usage_model::{ErrorCode, HarnessError};
use usage_support::{RequestSpec, Secret};

use crate::common::{Reply, TestServer, client};

async fn get(server: &TestServer) -> Result<serde_json::Value, HarnessError> {
    client().get_json(&RequestSpec::new(server.url("/data"))).await
}

async fn error_for(reply: impl Fn() -> Reply + Send + Sync + 'static) -> HarnessError {
    let server = TestServer::start(move |_| reply()).await;
    get(&server).await.unwrap_err()
}

fn param<'a>(error: &'a HarnessError, key: &str) -> Option<&'a str> {
    error.params.get(key).map(String::as_str)
}

#[tokio::test]
async fn a_json_object_is_returned_as_is_including_unicode() {
    let server =
        TestServer::start(|_| Reply::json(200, r#"{"nome":"João 🔑","n":[1,2.5,null]}"#)).await;
    let value = get(&server).await.unwrap();
    assert_eq!(value, json!({"nome": "João 🔑", "n": [1, 2.5, null]}));
}

#[tokio::test]
async fn any_json_root_is_accepted() {
    for body in ["[]", "42", "\"text\"", "null", "true", "  {\"a\":1}\n"] {
        let server = TestServer::start(move |_| Reply::json(200, body)).await;
        assert!(get(&server).await.is_ok(), "{body:?}");
    }
}

#[tokio::test]
async fn a_utf8_byte_order_mark_is_tolerated() {
    let server =
        TestServer::start(|_| Reply::with_type(200, "application/json", b"\xEF\xBB\xBF{\"a\":1}"))
            .await;
    assert_eq!(get(&server).await.unwrap(), json!({"a": 1}));
}

#[tokio::test]
async fn unauthorized_and_forbidden_are_not_retryable_and_carry_the_status() {
    for status in [401u16, 403] {
        let error =
            error_for(move || Reply::json(status, r#"{"error":"bad key sk-live-abc123"}"#)).await;
        assert_eq!(error.code, ErrorCode::Unauthorized, "{status}");
        assert!(!error.retryable);
        assert_eq!(param(&error, "httpStatus"), Some(status.to_string().as_str()));
        assert_eq!(param(&error, "host"), Some("127.0.0.1"));
        assert!(!format!("{error:?}").contains("sk-live"), "the body must never be echoed");
    }
}

#[tokio::test]
async fn not_found_is_not_retryable() {
    let error = error_for(|| Reply::json(404, "{}")).await;
    assert_eq!(error.code, ErrorCode::NotFound);
    assert!(!error.retryable);
    assert_eq!(param(&error, "httpStatus"), Some("404"));
}

#[tokio::test]
async fn rate_limit_is_retryable_and_reports_retry_after_in_integer_seconds_capped_at_an_hour() {
    for (header, expected) in [
        ("120", Some("120")),
        ("0", Some("0")),
        ("3600", Some("3600")),
        ("86400", Some("3600")),
        ("99999999999999999999999", Some("3600")),
        ("Wed, 21 Oct 2026 07:28:00 GMT", None),
        ("-1", None),
        ("soon", None),
        ("1.5", None),
    ] {
        let error = error_for(move || Reply::status(429, &[("Retry-After", header)])).await;
        assert_eq!(error.code, ErrorCode::RateLimited, "{header}");
        assert!(error.retryable);
        assert_eq!(param(&error, "retryAfterSecs"), expected, "{header}");
        assert_eq!(param(&error, "httpStatus"), Some("429"));
    }
    let error = error_for(|| Reply::status(429, &[])).await;
    assert_eq!(param(&error, "retryAfterSecs"), None);
}

#[tokio::test]
async fn server_errors_are_retryable() {
    for status in [500u16, 502, 503, 504, 599] {
        let error = error_for(move || Reply::status(status, &[])).await;
        assert_eq!(error.code, ErrorCode::ServerError, "{status}");
        assert!(error.retryable);
        assert_eq!(param(&error, "httpStatus"), Some(status.to_string().as_str()));
    }
}

#[tokio::test]
async fn other_error_statuses_are_bad_responses_with_the_status() {
    for status in [400u16, 405, 409, 418, 422] {
        let error = error_for(move || Reply::status(status, &[])).await;
        assert_eq!(error.code, ErrorCode::BadResponse, "{status}");
        assert!(!error.retryable);
        assert_eq!(param(&error, "httpStatus"), Some(status.to_string().as_str()));
    }
}

#[tokio::test]
async fn redirects_are_never_followed_and_never_echoed() {
    let target = TestServer::start(|_| Reply::json(200, r#"{"reached":true}"#)).await;
    let location = target.url("/stolen?token=sk-live-leak");
    for status in [301u16, 302, 303, 307, 308, 304] {
        let location = location.clone();
        let error =
            error_for(move || Reply::status(status, &[("Location", location.as_str())])).await;
        assert_eq!(error.code, ErrorCode::BadResponse, "{status}");
        assert!(!error.retryable);
        assert_eq!(param(&error, "redirect"), Some("true"));
        let text = format!("{error:?}");
        assert!(!text.contains("stolen") && !text.contains("sk-live"), "{text}");
    }
    target.assert_untouched().await;
}

#[tokio::test]
async fn html_and_other_non_json_content_types_are_rejected() {
    for content_type in ["text/html", "text/html; charset=utf-8", "application/xml", "image/png"] {
        let server =
            TestServer::start(move |_| Reply::with_type(200, content_type, b"{\"a\":1}")).await;
        let error = get(&server).await.unwrap_err();
        assert_eq!(error.code, ErrorCode::BadResponse, "{content_type}");
        assert!(!error.retryable);
    }
}

#[tokio::test]
async fn json_flavoured_or_missing_content_types_are_tolerated() {
    for content_type in
        ["application/problem+json", "text/plain", "Application/JSON; charset=UTF-8"]
    {
        let server =
            TestServer::start(move |_| Reply::with_type(200, content_type, b"{\"a\":1}")).await;
        assert!(get(&server).await.is_ok(), "{content_type}");
    }
    let no_type =
        b"HTTP/1.1 200 OK\r\nContent-Length: 7\r\nConnection: close\r\n\r\n{\"a\":1}".to_vec();
    let server = TestServer::start(move |_| Reply::Raw(no_type.clone())).await;
    assert_eq!(get(&server).await.unwrap(), json!({"a": 1}));
}

#[tokio::test]
async fn empty_and_whitespace_bodies_are_bad_responses_even_on_204() {
    for (status, body) in [(200u16, ""), (200, "   \n"), (204, "")] {
        let server = TestServer::start(move |_| Reply::json(status, body)).await;
        let error = get(&server).await.unwrap_err();
        assert_eq!(error.code, ErrorCode::BadResponse, "{status} {body:?}");
        assert!(!error.retryable);
    }
}

#[tokio::test]
async fn invalid_json_is_a_bad_response_that_never_echoes_the_body() {
    let hostile_depth = "[".repeat(100_000);
    for body in [
        r#"{"a":"#.to_owned(),
        r#"{"a":1} trailing sk-live-leak"#.to_owned(),
        "<html>sk-live-leak</html>".to_owned(),
        "{'a':1}".to_owned(),
        "NaN".to_owned(),
        hostile_depth,
    ] {
        let server = TestServer::start(move |_| Reply::json(200, &body)).await;
        let error = get(&server).await.unwrap_err();
        assert_eq!(error.code, ErrorCode::BadResponse);
        assert!(!format!("{error:?}").contains("sk-live"));
    }
}

#[tokio::test]
async fn errors_never_contain_the_credential_that_was_sent() {
    let secret = Secret::new("sk-live-4f9a2c71-fabricated");
    for reply in [
        || Reply::json(401, r#"{"detail":"invalid key"}"#),
        || Reply::json(500, "{}"),
        || Reply::json(200, "not json"),
        || Reply::Hangup,
    ] {
        let server = TestServer::start(move |_| reply()).await;
        let request = RequestSpec::new(server.url("/k?api_key=query-secret"))
            .auth(usage_support::Auth::new(&secret))
            .secret_header("X-Csrf-Token", &Secret::new("csrf-fabricated-9d8c"));
        let error = client().get_json(&request).await.unwrap_err();
        let text = format!("{error:?} {error:#?} {request:?}");
        for needle in ["sk-live-4f9a", "csrf-fabricated", "query-secret", "api_key=query"] {
            assert!(!text.contains(needle), "{needle} leaked: {text}");
        }
    }
}
