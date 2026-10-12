//! Every way the request can fail, as the notice the user would see. The statusLine data always
//! survives; only the `reason` differs.

use std::net::TcpListener;

use usage_model::Severity;

use super::*;
use crate::testenv::usage_limits;

const BODY_CAP: usize = 1024 * 1024;

async fn view_for(reply: impl Fn() -> Reply + Send + Sync + 'static) -> HarnessView {
    let env = ready_env();
    let server = FakeServer::start(move |_| reply());
    poll(&env, &server, NOW).await
}

#[tokio::test]
async fn http_statuses_map_to_their_error_codes() {
    let cases = [
        (401, "unauthorized"),
        (403, "unauthorized"),
        (404, "not_found"),
        (429, "rate_limited"),
        (500, "server_error"),
        (503, "server_error"),
        (302, "bad_response"),
        (400, "bad_response"),
        (418, "bad_response"),
    ];
    for (status, reason) in cases {
        let view =
            view_for(move || Reply::status(status, &[("Location", "http://127.0.0.1:9/")])).await;
        assert_partial(&view, reason);
    }
}

#[tokio::test]
async fn a_notice_names_only_the_reason_at_info_severity() {
    let view = view_for(|| Reply::status(429, &[("Retry-After", "30")])).await;
    assert_partial(&view, "rate_limited");
    assert_eq!(view.notices[0].severity, Severity::Info);
    assert_eq!(view.notices[0].params.len(), 1, "no status, host or path in the notice");
}

#[tokio::test]
async fn a_body_that_is_not_json_is_a_bad_response() {
    for reply in [
        (|| Reply::json("not json")) as fn() -> Reply,
        || Reply::json(""),
        || Reply::json("{\"five_hour\":"),
        || Reply::typed(200, "text/html", b"<html>maintenance</html>"),
    ] {
        assert_partial(&view_for(reply).await, "bad_response");
    }
}

#[tokio::test]
async fn json_of_another_shape_means_the_endpoint_changed() {
    for body in ["{}", "[1,2]", "{\"hello\":\"world\"}", "null", "\"x\"", "7"] {
        let view = view_for(move || Reply::json(body)).await;
        assert_partial(&view, "endpoint_changed");
    }
}

#[tokio::test]
async fn a_body_over_the_cap_is_refused_whether_or_not_its_size_is_declared() {
    let declared =
        view_for(|| Reply::typed(200, "application/json", &vec![b' '; BODY_CAP + 1])).await;
    assert_partial(&declared, "response_too_large");

    let undeclared = view_for(|| {
        let mut raw =
            b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n"
                .to_vec();
        raw.extend(std::iter::repeat_n(b' ', BODY_CAP + 4096));
        Reply::Raw(raw)
    })
    .await;
    assert_partial(&undeclared, "response_too_large");
}

#[tokio::test]
async fn a_server_that_never_answers_times_out() {
    let env = ready_env();
    let server = FakeServer::start(|_| Reply::Silent);
    let short = Endpoint { timeout: Duration::from_millis(300), ..endpoint(&server) };
    assert_partial(&poll_with(&env, &short, NOW).await, "timeout");
}

#[tokio::test]
async fn a_closed_port_is_a_network_error() {
    let env = ready_env();
    let port = TcpListener::bind(("127.0.0.1", 0)).unwrap().local_addr().unwrap().port();
    let closed = Endpoint {
        base: format!("http://127.0.0.1:{port}"),
        allow_loopback_http: true,
        timeout: TEST_TIMEOUT,
    };
    assert_partial(&poll_with(&env, &closed, NOW).await, "network");
}

#[tokio::test]
async fn garbage_instead_of_http_is_a_network_error() {
    let view = view_for(|| Reply::Raw(b"\x00\x01 this is not http\r\n\r\n".to_vec())).await;
    assert_partial(&view, "network");
}

#[tokio::test]
async fn plain_http_is_refused_before_any_connection_unless_loopback_is_allowed() {
    let env = ready_env();
    let server = FakeServer::json(&usage_limits());
    let strict = Endpoint { allow_loopback_http: false, ..endpoint(&server) };
    assert_partial(&poll_with(&env, &strict, NOW).await, "invalid_url");
    server.assert_untouched();
}
