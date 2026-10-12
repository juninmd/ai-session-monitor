//! A poll that cannot deliver: the most specific error, retryable only when waiting can help, and
//! no window a user could mistake for data.

use usage_model::{ErrorCode, HarnessError, HarnessId, ViewState};

use super::*;
use crate::discovery::DiscoveryError;
use crate::fake::Reply;

/// The error of a view that failed, after the checks every failure shares.
pub(super) fn failed(view: &HarnessView) -> &HarnessError {
    assert_eq!(view.id, HarnessId::Antigravity);
    assert_eq!(view.state, ViewState::Error);
    assert!(view.windows.is_empty() && view.notices.is_empty() && view.facts.is_empty());
    assert!(view.source.is_none() && view.plan.is_none());
    view.error.as_ref().expect("an error view carries its error")
}

pub(super) fn params(error: &HarnessError) -> Vec<(&str, &str)> {
    error.params.iter().map(|(key, value)| (key.as_str(), value.as_str())).collect()
}

pub(super) fn kind(error: &HarnessError) -> (ErrorCode, bool) {
    (error.code, error.retryable)
}

/// One server that answers every request with `reply`.
pub(super) async fn against(reply: Reply) -> HarnessView {
    let server = FakeServer::http(move |_| reply.clone()).await;
    poll(vec![server.target(TOKEN)]).await
}

async fn discovery_says(error: DiscoveryError) -> HarnessView {
    poll_with(&on(), &StaticDiscovery::new(Err(error)), &config()).await
}

#[tokio::test]
async fn no_language_server_is_a_normal_state_that_heals_by_itself() {
    let view = discovery_says(DiscoveryError::NotRunning).await;
    let error = failed(&view);
    assert_eq!(kind(error), (ErrorCode::NotRunning, true));
    assert!(error.params.is_empty());
    assert!(view.read_at.is_none());

    let view = poll(Vec::new()).await;
    assert_eq!(kind(failed(&view)), (ErrorCode::NotRunning, true));
}

#[tokio::test]
async fn a_server_still_starting_may_be_asked_again_later() {
    let view = discovery_says(DiscoveryError::NoPort(None)).await;
    let error = failed(&view);
    assert_eq!(kind(error), (ErrorCode::NotRunning, true));
    assert_eq!(params(error), [("reason", "no_port")]);
}

#[tokio::test]
async fn a_server_without_a_usable_token_needs_the_user_not_a_retry() {
    let view = discovery_says(DiscoveryError::NoToken).await;
    let error = failed(&view);
    assert_eq!(kind(error), (ErrorCode::Unauthorized, false));
    assert_eq!(params(error), [("reason", "no_token")]);
}

#[tokio::test]
async fn a_failure_of_the_process_scan_is_passed_through_as_it_is() {
    let cause = HarnessError::new(ErrorCode::SpawnFailed, false).with_param("reason", "scan");
    let lists =
        [DiscoveryError::Failed(cause.clone()), DiscoveryError::NoPort(Some(cause.clone()))];
    for error in lists {
        let view = discovery_says(error).await;
        assert_eq!(failed(&view), &cause);
    }
}

#[tokio::test]
async fn a_rejected_token_is_unauthorized_and_the_poll_stops_asking() {
    let server = serve("status_ide", Some("summary_full")).await;
    let view = poll(vec![server.target("not-the-token")]).await;
    let error = failed(&view);
    assert_eq!(kind(error), (ErrorCode::Unauthorized, false));
    assert_eq!(params(error), [("host", "127.0.0.1"), ("httpStatus", "401")]);
    assert_eq!(server.requests().len(), 1, "a refused status ends the attempt");
}

#[tokio::test]
async fn http_statuses_map_to_the_error_that_tells_the_user_what_to_do() {
    for (status, code, retryable) in [
        (401, ErrorCode::Unauthorized, false),
        (403, ErrorCode::Unauthorized, false),
        (404, ErrorCode::EndpointChanged, false),
        (429, ErrorCode::RateLimited, true),
        (500, ErrorCode::ServerError, true),
        (503, ErrorCode::ServerError, true),
        (302, ErrorCode::BadResponse, false),
    ] {
        let view = against(Reply::status(status)).await;
        let error = failed(&view);
        assert_eq!(kind(error), (code, retryable), "status {status}");
        let shown = error.params.get("httpStatus").map(String::as_str);
        assert_eq!(shown, Some(status.to_string().as_str()), "status {status}");
    }
}

#[tokio::test]
async fn a_retry_hint_is_passed_on_in_seconds() {
    let raw = "HTTP/1.1 429 Too Many Requests\r\nRetry-After: 120\r\nContent-Length: 0\r\n\r\n";
    let view = against(Reply::Raw(raw.as_bytes().to_vec())).await;
    let error = failed(&view);
    assert_eq!(kind(error), (ErrorCode::RateLimited, true));
    let expected = [("host", "127.0.0.1"), ("httpStatus", "429"), ("retryAfterSecs", "120")];
    assert_eq!(params(error), expected);
}

#[tokio::test]
async fn text_where_json_is_expected_is_a_bad_response() {
    let plain = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 2\r\n\r\nOK";
    for reply in [
        Reply::Html(200, "<html><body>Sign in</body></html>".to_owned()),
        Reply::json(200, "this is not json"),
        Reply::json(200, ""),
        Reply::Raw(plain.as_bytes().to_vec()),
    ] {
        let label = format!("{reply:?}");
        let view = against(reply).await;
        assert_eq!(kind(failed(&view)), (ErrorCode::BadResponse, false), "{label}");
    }
}

#[tokio::test]
async fn json_of_the_wrong_shape_is_an_endpoint_that_changed() {
    for body in [
        "{}",
        "[]",
        "null",
        "42",
        r#""text""#,
        r#"{"hello":"world"}"#,
        r#"{"userStatus":{}}"#,
        r#"{"userStatus":"x"}"#,
        r#"{"userStatus":{"unknown":{"a":1}}}"#,
    ] {
        let view = against(Reply::json(200, body)).await;
        let error = failed(&view);
        assert_eq!(kind(error), (ErrorCode::EndpointChanged, false), "{body}");
        assert!(error.params.is_empty(), "{body}");
    }
}

#[tokio::test]
async fn an_error_envelope_with_http_200_is_read_as_the_error_it_is() {
    for (body, code, retryable) in [
        (r#"{"code":16,"message":"missing token"}"#, ErrorCode::Unauthorized, false),
        (r#"{"code":"unauthenticated"}"#, ErrorCode::Unauthorized, false),
        (r#"{"code":14}"#, ErrorCode::ServerError, true),
        (r#"{"code":4}"#, ErrorCode::Timeout, true),
        (r#"{"code":5}"#, ErrorCode::EndpointChanged, false),
    ] {
        let view = against(Reply::json(200, body)).await;
        assert_eq!(kind(failed(&view)), (code, retryable), "{body}");
    }
    let view = against(Reply::json(200, r#"{"code":5}"#)).await;
    assert_eq!(params(failed(&view)), [("code", "5")]);
}

#[tokio::test]
async fn a_summary_that_fails_keeps_the_windows_the_status_already_gave() {
    for summary in [
        Reply::json(200, r#"{"unexpected":true}"#),
        Reply::Html(200, "<p>maintenance</p>".to_owned()),
        Reply::status(500),
        Reply::status(401),
    ] {
        let label = format!("{summary:?}");
        let script = Script { summary, ..Script::new(TOKEN, text("status_ide"), None) };
        let server = script.serve_http().await;
        let view = poll(vec![server.target(TOKEN)]).await;
        assert_eq!(view.state, ViewState::Ready, "{label}");
        assert_eq!(
            rows(&view),
            [
                ("gemini:five_hour", Some(30.0), Some(1_791_641_520)),
                ("claude_gpt:five_hour", Some(50.0), Some(1_791_649_800)),
            ],
            "{label}"
        );
        assert!(view.error.is_none(), "{label}");
    }
}

#[tokio::test]
async fn a_summary_that_fails_without_any_window_to_fall_back_on_is_the_error_of_the_summary() {
    for (summary, code, retryable) in [
        (Reply::json(200, r#"{"unexpected":true}"#), ErrorCode::EndpointChanged, false),
        (Reply::Html(200, "<p>maintenance</p>".to_owned()), ErrorCode::BadResponse, false),
        (Reply::status(500), ErrorCode::ServerError, true),
        (Reply::status(429), ErrorCode::RateLimited, true),
        (Reply::status(401), ErrorCode::Unauthorized, false),
    ] {
        let label = format!("{summary:?}");
        let script = Script { summary, ..Script::new(TOKEN, text("status_hub"), None) };
        let server = script.serve_http().await;
        let view = poll(vec![server.target(TOKEN)]).await;
        assert_eq!(kind(failed(&view)), (code, retryable), "{label}");
    }
}

#[tokio::test]
async fn a_status_in_an_unknown_shape_still_leaves_the_summary_a_chance() {
    for status in [
        Reply::status(404),
        Reply::json(200, r#"{"hello":"world"}"#),
        Reply::json(200, r#"{"code":5}"#),
    ] {
        let label = format!("{status:?}");
        let base = Script::new(TOKEN, text("status_hub"), Some(text("summary_full")));
        let server = Script { status, ..base }.serve_http().await;
        let view = poll(vec![server.target(TOKEN)]).await;
        assert_eq!(view.state, ViewState::Ready, "{label}");
        assert_eq!(view.windows.len(), 4, "{label}");
        assert!(view.plan.is_none() && view.error.is_none(), "{label}");
        assert_eq!(server.requests().len(), 2, "{label}");
    }
}

#[tokio::test]
async fn a_status_that_says_this_is_not_the_server_ends_the_attempt_with_one_question() {
    for (status, code) in [
        (Reply::Html(200, "<p>no</p>".to_owned()), ErrorCode::BadResponse),
        (Reply::json(200, "not json"), ErrorCode::BadResponse),
        (Reply::status(500), ErrorCode::ServerError),
        (Reply::status(429), ErrorCode::RateLimited),
        (Reply::status(401), ErrorCode::Unauthorized),
        (Reply::json(200, r#"{"code":16}"#), ErrorCode::Unauthorized),
        (Reply::Hangup, ErrorCode::Network),
    ] {
        let label = format!("{status:?}");
        let base = Script::new(TOKEN, text("status_hub"), Some(text("summary_full")));
        let server = Script { status, ..base }.serve_http().await;
        let view = poll(vec![server.target(TOKEN)]).await;
        assert_eq!(failed(&view).code, code, "{label}");
        assert_eq!(server.requests().len(), 1, "{label}");
    }
}

#[tokio::test]
async fn when_both_calls_fail_the_more_telling_failure_is_reported() {
    let cases = [
        (Reply::json(200, "{}"), Reply::status(500), ErrorCode::ServerError, true),
        (Reply::status(404), Reply::json(200, "{}"), ErrorCode::EndpointChanged, false),
        (Reply::status(404), Reply::status(401), ErrorCode::Unauthorized, false),
        (Reply::status(404), Reply::status(404), ErrorCode::EndpointChanged, false),
    ];
    for (status, summary, code, retryable) in cases {
        let label = format!("{status:?} then {summary:?}");
        let script = Script { status, summary, ..Script::new(TOKEN, "{}", None) };
        let server = script.serve_http().await;
        let view = poll(vec![server.target(TOKEN)]).await;
        assert_eq!(kind(failed(&view)), (code, retryable), "{label}");
    }
}
