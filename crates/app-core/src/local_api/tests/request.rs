use super::super::request::{MAX_HEAD_BYTES, MAX_HEADERS, Malformed, Request, parse};

fn ok(head: &str) -> Request {
    parse(head.as_bytes()).expect("valid head")
}

fn bad(head: &str) -> Malformed {
    parse(head.as_bytes()).expect_err("invalid head")
}

#[test]
fn a_plain_get_is_parsed() {
    let request = ok("GET /v1/limits HTTP/1.1\r\nHost: 127.0.0.1:6737\r\nAccept: */*\r\n\r\n");
    assert_eq!(request.method, "GET");
    assert_eq!(request.path, "/v1/limits");
    assert_eq!(request.host.as_deref(), Some("127.0.0.1:6737"));
    assert!(!request.has_origin);
}

#[test]
fn the_query_string_is_dropped_and_header_names_are_case_insensitive() {
    let request =
        ok("GET /v1/health?x=1#frag HTTP/1.0\r\nhOsT:\t localhost:6737 \r\nORIGIN: null\r\n\r\n");
    assert_eq!(request.path, "/v1/health");
    assert_eq!(request.host.as_deref(), Some("localhost:6737"));
    assert!(request.has_origin, "even `Origin: null` counts");
}

#[test]
fn a_missing_host_is_none_and_left_to_the_router() {
    assert_eq!(ok("GET / HTTP/1.1\r\n\r\n").host, None);
}

#[test]
fn malformed_request_lines_are_a_400() {
    for head in [
        "\r\n\r\n",
        "GET\r\n\r\n",
        "GET / \r\n\r\n",
        "GET /  HTTP/1.1\r\n\r\n",
        "GET / HTTP/1.1 extra\r\n\r\n",
        "GET / HTTP/2.0\r\n\r\n",
        "GET / HTTP/1.1x\r\n\r\n",
        "G E T / HTTP/1.1\r\n\r\n",
        "GET http://127.0.0.1:6737/v1/limits HTTP/1.1\r\n\r\n",
        "GET * HTTP/1.1\r\n\r\n",
        "GET //evil.example/ HTTP/1.1\r\n\r\n",
        "GET v1/limits HTTP/1.1\r\n\r\n",
        "get\u{e9} / HTTP/1.1\r\n\r\n",
    ] {
        assert_eq!(bad(head), Malformed::BadRequest, "{head:?}");
    }
}

#[test]
fn malformed_headers_are_a_400() {
    for header in [
        "NoColon",
        ": empty-name",
        " Folded: value",
        "Host : spaced-name",
        "Bad Name: value",
        "Host: a\rb",
        "X: \u{0}",
        "X: \u{7f}",
        "X: caf\u{e9}",
        "Content-Length: 5",
        "Content-Length: abc",
        "Transfer-Encoding: chunked",
    ] {
        let head = format!("GET / HTTP/1.1\r\nHost: localhost:6737\r\n{header}\r\n\r\n");
        assert_eq!(bad(&head), Malformed::BadRequest, "{header:?}");
    }
}

#[test]
fn a_zero_content_length_is_fine_and_a_duplicate_host_is_not() {
    assert!(ok("GET / HTTP/1.1\r\nHost: a\r\nContent-Length: 0\r\n\r\n").host.is_some());
    let twice = "GET / HTTP/1.1\r\nHost: localhost:6737\r\nHost: evil.example\r\n\r\n";
    assert_eq!(bad(twice), Malformed::BadRequest);
}

#[test]
fn a_bare_newline_or_a_missing_blank_line_is_a_400() {
    assert_eq!(bad("GET / HTTP/1.1\nHost: a\n\n"), Malformed::BadRequest);
    assert_eq!(bad("GET / HTTP/1.1\r\nHost: a\r\nX: b\nY: c\r\n\r\n"), Malformed::BadRequest);
    assert_eq!(bad("GET / HTTP/1.1\r\nHost: a\r\n"), Malformed::BadRequest);
    assert_eq!(parse(&[0xff, 0xfe, b'\r', b'\n', b'\r', b'\n']), Err(Malformed::BadRequest));
}

#[test]
fn an_oversized_head_or_too_many_headers_is_a_431() {
    let long = format!("GET / HTTP/1.1\r\nX: {}\r\n\r\n", "a".repeat(MAX_HEAD_BYTES));
    assert_eq!(bad(&long), Malformed::TooLarge);
    let many: String = (0..=MAX_HEADERS).map(|i| format!("X-{i}: v\r\n")).collect();
    assert_eq!(bad(&format!("GET / HTTP/1.1\r\n{many}\r\n")), Malformed::TooLarge);
    let few: String = (0..MAX_HEADERS).map(|i| format!("X-{i}: v\r\n")).collect();
    assert!(parse(format!("GET / HTTP/1.1\r\n{few}\r\n").as_bytes()).is_ok());
}
