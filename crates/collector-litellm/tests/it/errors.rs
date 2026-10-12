//! Every way the first request can fail, and the most specific error code for each.

use std::time::{Duration, Instant};

use tokio::net::TcpListener;
use usage_model::settings::LitellmSettings;
use usage_model::{ErrorCode, HarnessView};

use crate::common::{Fake, Reply, SECRET, error_of, param, run, settings};

const HTML: &str = fixture!("error_page.html");

async fn answered(reply: Reply) -> (Fake, HarnessView) {
    let fake = Fake::routes([("/key/info", reply)]).await;
    let view = run(&settings(&fake.base())).await;
    (fake, view)
}

#[tokio::test]
async fn http_statuses_map_to_the_most_specific_code() {
    let cases = [
        (400, ErrorCode::BadResponse, false),
        (401, ErrorCode::Unauthorized, false),
        (403, ErrorCode::Unauthorized, false),
        (404, ErrorCode::NotFound, false),
        (418, ErrorCode::BadResponse, false),
        (429, ErrorCode::RateLimited, true),
        (500, ErrorCode::ServerError, true),
        (502, ErrorCode::ServerError, true),
        (503, ErrorCode::ServerError, true),
    ];
    // Proxies echo the rejected key in their error body; none of it may reach the view.
    let body =
        format!(r#"{{"error":{{"message":"Authentication Error: {SECRET} decoy-detail"}}}}"#);
    for (status, code, retryable) in cases {
        let (_, view) = answered(Reply::json_status(status, &body)).await;
        let error = error_of(&view);
        assert_eq!((error.code, error.retryable), (code, retryable), "{status}");
        assert_eq!(param(error, "httpStatus"), Some(status.to_string().as_str()));
        assert_eq!(param(error, "host"), Some("127.0.0.1"));
        assert_eq!(error.params.len(), 2, "{status}: {error:?}");
    }
}

#[tokio::test]
async fn a_rate_limit_carries_a_bounded_retry_after() {
    let cases = [("120", Some("120")), ("0", Some("0")), ("999999", Some("3600")), ("soon", None)];
    for (header, expected) in cases {
        let (_, view) = answered(Reply::status(429, &[("Retry-After", header)])).await;
        let error = error_of(&view);
        assert_eq!((error.code, error.retryable), (ErrorCode::RateLimited, true));
        assert_eq!(param(error, "retryAfterSecs"), expected, "Retry-After: {header}");
    }
    let (_, view) = answered(Reply::status(429, &[])).await;
    assert_eq!(param(error_of(&view), "retryAfterSecs"), None);
}

#[tokio::test]
async fn redirects_are_errors_and_are_never_followed() {
    let elsewhere =
        Fake::routes([("/key/info", Reply::json(fixture!("key_with_budget.json")))]).await;
    let target = format!("{}/key/info", elsewhere.base());
    for status in [301, 302, 307, 308] {
        let (_, view) = answered(Reply::status(status, &[("Location", target.as_str())])).await;
        let error = error_of(&view);
        assert_eq!((error.code, error.retryable), (ErrorCode::BadResponse, false), "{status}");
        assert_eq!(param(error, "redirect"), Some("true"));
        assert_eq!(param(error, "httpStatus"), Some(status.to_string().as_str()));
    }
    elsewhere.assert_untouched().await;
}

#[tokio::test]
async fn an_html_error_page_is_a_bad_response_or_a_server_error_by_status() {
    let (_, view) = answered(Reply::typed(200, "text/html; charset=utf-8", HTML)).await;
    assert_eq!(error_of(&view).code, ErrorCode::BadResponse);
    let (_, view) = answered(Reply::typed(502, "text/html", HTML)).await;
    let error = error_of(&view);
    assert_eq!((error.code, error.retryable), (ErrorCode::ServerError, true));
    assert_eq!(param(error, "httpStatus"), Some("502"));
}

#[tokio::test]
async fn bodies_that_are_not_json_are_a_bad_response() {
    let cases = [
        ("html labelled as json", Reply::json(HTML)),
        ("plain text", Reply::typed(200, "text/plain", "ok")),
        ("empty body", Reply::json("")),
        ("truncated json", Reply::json(r#"{"info": {"spend": 1"#)),
        ("xml", Reply::typed(200, "application/xml", "<info/>")),
        ("a json prefix with trailing text", Reply::json(r#"{"info": {}} extra"#)),
    ];
    for (name, reply) in cases {
        let (_, view) = answered(reply).await;
        let error = error_of(&view);
        assert_eq!((error.code, error.retryable), (ErrorCode::BadResponse, false), "{name}");
    }
}

#[tokio::test]
async fn a_body_without_the_documented_record_is_a_changed_endpoint() {
    let cases = [
        ("[]", "not_an_object"),
        (r#""text""#, "not_an_object"),
        ("7", "not_an_object"),
        ("null", "not_an_object"),
        ("{}", "no_record"),
        (r#"{"info": null}"#, "no_record"),
        (r#"{"info": []}"#, "no_record"),
        (r#"{"info": "x"}"#, "no_record"),
        (r#"{"detail": {"error": "proxy is starting"}}"#, "no_record"),
        (r#"{"user_info": {"max_budget": 5}}"#, "no_record"),
    ];
    for (body, reason) in cases {
        let (_, view) = answered(Reply::json(body)).await;
        let error = error_of(&view);
        assert_eq!((error.code, error.retryable), (ErrorCode::EndpointChanged, false), "{body}");
        assert_eq!(param(error, "reason"), Some(reason), "{body}");
        assert_eq!(param(error, "host"), Some("127.0.0.1"));
    }
}

#[tokio::test]
async fn a_body_over_the_cap_is_refused_declared_or_streamed() {
    for chunked in [false, true] {
        let (_, view) = answered(Reply::Big { bytes: 5 << 20, chunked }).await;
        let error = error_of(&view);
        assert_eq!(
            (error.code, error.retryable),
            (ErrorCode::ResponseTooLarge, false),
            "{chunked}"
        );
    }
    let padded = format!(r#"{{"info": {{"padding": "{}"}}}}"#, "p".repeat((1 << 20) + 1));
    let (_, view) = answered(Reply::json(&padded)).await;
    assert_eq!(error_of(&view).code, ErrorCode::ResponseTooLarge, "valid JSON is capped too");
}

#[tokio::test]
async fn a_slow_proxy_times_out_within_the_configured_time() {
    let fake = Fake::routes([(
        "/key/info",
        Reply::json(fixture!("key_with_budget.json")).after(Duration::from_secs(30)),
    )])
    .await;
    let settings = LitellmSettings { timeout_secs: 1, ..settings(&fake.base()) };
    let started = Instant::now();
    let view = run(&settings).await;
    let error = error_of(&view);
    assert_eq!((error.code, error.retryable), (ErrorCode::Timeout, true));
    assert_eq!(param(error, "host"), Some("127.0.0.1"));
    assert!(started.elapsed() < Duration::from_secs(5), "{:?}", started.elapsed());
}

#[tokio::test]
async fn a_proxy_that_hangs_up_is_a_retryable_network_error() {
    let (_, view) = answered(Reply::Hangup).await;
    let error = error_of(&view);
    assert_eq!((error.code, error.retryable), (ErrorCode::Network, true));
    assert_eq!(param(error, "host"), Some("127.0.0.1"));
}

#[tokio::test]
async fn nothing_listening_is_a_retryable_network_error() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);
    let view = run(&settings(&base)).await;
    let error = error_of(&view);
    assert_eq!((error.code, error.retryable), (ErrorCode::Network, true));
}

#[tokio::test]
async fn a_wrong_base_path_is_not_found_rather_than_a_changed_endpoint() {
    let fake =
        Fake::routes([("/litellm/key/info", Reply::json(fixture!("key_with_budget.json")))]).await;
    let view = run(&settings(&fake.base())).await;
    let error = error_of(&view);
    assert_eq!((error.code, error.retryable), (ErrorCode::NotFound, false));
    assert_eq!(fake.targets(), ["/key/info"]);
}
