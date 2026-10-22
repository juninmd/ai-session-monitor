//! Loopback integration: a real listener on an ephemeral port and real sockets.

use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::time::Duration;

use usage_model::settings::LOCAL_API_PORT;

use super::super::{LocalApi, StartError};
use super::support::*;

#[test]
fn production_listens_on_the_documented_port() {
    assert_eq!(LOCAL_API_PORT, 6737);
}

#[test]
fn it_binds_loopback_ipv4_only() {
    let api = start();
    assert_eq!(api.addr().ip(), Ipv4Addr::LOCALHOST);
    let v6 = SocketAddr::from((std::net::Ipv6Addr::LOCALHOST, api.port()));
    assert!(TcpStream::connect_timeout(&v6, Duration::from_millis(500)).is_err());
}

#[test]
fn health_and_limits_answer_with_hardened_headers() {
    let api = start();
    for host in
        [own_host(&api), format!("localhost:{}", api.port()), format!("LOCALHOST:{}", api.port())]
    {
        let health = get(&api, "/v1/health", &host);
        assert_eq!(health.status, 200);
        assert_eq!(health.body, r#"{"ok":true}"#);
        let limits = get(&api, "/v1/limits", &host);
        assert_eq!(limits.status, 200, "{host}");
        let json: serde_json::Value = serde_json::from_str(&limits.body).expect("json");
        assert_eq!(json["harnesses"][0]["windows"][0]["availablePct"], 70.0);
        for reply in [&health, &limits] {
            assert_eq!(reply.headers["cache-control"], "no-store");
            assert_eq!(reply.headers["x-content-type-options"], "nosniff");
            assert_eq!(reply.headers["content-type"], "application/json; charset=utf-8");
            assert_eq!(reply.headers["content-length"], reply.body.len().to_string());
            assert!(
                reply.headers.keys().all(|name| !name.starts_with("access-control-")),
                "no CORS header, ever"
            );
        }
    }
}

#[test]
fn a_query_string_does_not_change_the_route() {
    let api = start();
    assert_eq!(get(&api, "/v1/health?x=1", &own_host(&api)).status, 200);
}

#[test]
fn nothing_to_show_yet_is_a_503() {
    let api = LocalApi::start_on(0, Arc::new(|| None)).expect("start");
    assert_eq!(get(&api, "/v1/limits", &own_host(&api)).status, 503);
    assert_eq!(get(&api, "/v1/health", &own_host(&api)).status, 200);
}

#[test]
fn a_foreign_host_is_refused_even_on_unknown_paths() {
    let api = start();
    let port = api.port();
    for host in [
        "evil.example".to_owned(),
        format!("evil.example:{port}"),
        "127.0.0.1".to_owned(),
        "localhost".to_owned(),
        format!("127.0.0.1:{}", port.wrapping_add(1)),
        format!("[::1]:{port}"),
        format!("0.0.0.0:{port}"),
        format!("127.0.0.1.evil.example:{port}"),
        String::new(),
    ] {
        for path in ["/v1/limits", "/v1/health", "/nope"] {
            let reply = get(&api, path, &host);
            assert_eq!(reply.status, 403, "{host:?} {path}");
            assert_eq!(reply.body, r#"{"error":"forbidden"}"#);
        }
    }
}

#[test]
fn a_missing_or_duplicate_host_is_a_400() {
    let api = start();
    assert_eq!(send(&api, "GET /v1/health HTTP/1.1\r\n\r\n").status, 400);
    let twice = format!(
        "GET /v1/health HTTP/1.1\r\nHost: {0}\r\nHost: evil.example\r\n\r\n",
        own_host(&api)
    );
    assert_eq!(send(&api, &twice).status, 400);
}

#[test]
fn any_origin_header_is_refused() {
    let api = start();
    for origin in ["https://evil.example", "null", "http://127.0.0.1:6737", ""] {
        let raw = format!(
            "GET /v1/limits HTTP/1.1\r\nHost: {}\r\nOrigin: {origin}\r\n\r\n",
            own_host(&api)
        );
        let reply = send(&api, &raw);
        assert_eq!(reply.status, 403, "{origin:?}");
        assert!(!reply.body.contains("harnesses"));
    }
}

#[test]
fn only_get_is_served() {
    let api = start();
    let host = own_host(&api);
    for method in ["POST", "PUT", "DELETE", "PATCH", "OPTIONS", "HEAD", "TRACE", "CONNECT"] {
        let reply = send(&api, &format!("{method} /v1/limits HTTP/1.1\r\nHost: {host}\r\n\r\n"));
        assert_eq!(reply.status, 405, "{method}");
        assert_eq!(reply.headers["allow"], "GET");
        assert!(!reply.headers.contains_key("access-control-allow-methods"));
    }
    let unknown = send(&api, &format!("POST /nope HTTP/1.1\r\nHost: {host}\r\n\r\n"));
    assert_eq!(unknown.status, 404, "an unknown path is a 404 whatever the method");
}

#[test]
fn unknown_paths_are_404() {
    let api = start();
    for path in
        ["/", "/v1", "/v1/limits/", "/v1/Limits", "/v2/limits", "/v1/limits/x", "/%76%31/limits"]
    {
        assert_eq!(get(&api, path, &own_host(&api)).status, 404, "{path}");
    }
}

#[test]
fn malformed_and_oversized_requests_get_400_and_431() {
    let api = start();
    let host = own_host(&api);
    assert_eq!(send(&api, "garbage\r\n\r\n").status, 400);
    assert_eq!(
        send(
            &api,
            &format!("GET /v1/health HTTP/1.1\r\nHost: {host}\r\nContent-Length: 4\r\n\r\nbody")
        )
        .status,
        400
    );
    let huge =
        format!("GET /v1/health HTTP/1.1\r\nHost: {host}\r\nX: {}\r\n\r\n", "a".repeat(20_000));
    assert_eq!(send(&api, &huge).status, 431);
    // Never a terminator: the cap, not the deadline, ends it.
    let endless = format!("GET /v1/health HTTP/1.1\r\nHost: {host}\r\nX: {}", "a".repeat(20_000));
    assert_eq!(send(&api, &endless).status, 431);
}

#[test]
fn a_taken_port_fails_softly_and_a_later_start_works() {
    let first = start();
    let taken = LocalApi::start_on(first.port(), provider());
    assert_eq!(taken.expect_err("port in use"), StartError::PortInUse);
    let port = first.port();
    drop(first);
    let again = LocalApi::start_on(port, provider()).expect("the port is free again");
    assert_eq!(again.port(), port);
}

#[test]
fn a_port_held_by_a_foreign_listener_is_reported_as_in_use() {
    let foreign = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("bind");
    let port = foreign.local_addr().expect("addr").port();
    assert_eq!(LocalApi::start_on(port, provider()).expect_err("in use"), StartError::PortInUse);
    assert_eq!(StartError::PortInUse.to_string(), "the port is in use");
}

#[test]
fn dropping_the_handle_stops_the_listener() {
    let api = start();
    let addr = api.addr();
    drop(api);
    assert!(TcpStream::connect_timeout(&addr, Duration::from_millis(500)).is_err());
}
