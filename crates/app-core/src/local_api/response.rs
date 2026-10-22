//! Routing and the wire form of a response. Every response, errors included, carries the same
//! hardening headers and no CORS ones.

use super::request::Request;
use super::server::Provider;

#[derive(Debug)]
pub(super) struct Response {
    pub status: u16,
    body: Vec<u8>,
}

impl Response {
    pub(super) fn json(status: u16, body: Vec<u8>) -> Self {
        Self { status, body }
    }

    /// `{"error":"<slug>"}`: a fixed word per status, never anything from the request.
    pub(super) fn error(status: u16) -> Self {
        let slug = match status {
            400 => "bad_request",
            403 => "forbidden",
            404 => "not_found",
            405 => "method_not_allowed",
            408 => "request_timeout",
            431 => "headers_too_large",
            503 => "unavailable",
            _ => "error",
        };
        Self::json(status, format!("{{\"error\":\"{slug}\"}}").into_bytes())
    }

    pub(super) fn to_bytes(&self) -> Vec<u8> {
        let reason = match self.status {
            200 => "OK",
            400 => "Bad Request",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            408 => "Request Timeout",
            431 => "Request Header Fields Too Large",
            _ => "Service Unavailable",
        };
        let allow = if self.status == 405 { "Allow: GET\r\n" } else { "" };
        let mut out = format!(
            "HTTP/1.1 {} {reason}\r\n{allow}Content-Type: application/json; charset=utf-8\r\n\
             Content-Length: {}\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\n\
             Connection: close\r\n\r\n",
            self.status,
            self.body.len()
        )
        .into_bytes();
        out.extend_from_slice(&self.body);
        out
    }
}

/// Order matters: nothing about the API's shape is revealed to a request that fails the Host or
/// Origin check, and a wrong method on a known path is 405, on an unknown one 404.
pub(super) fn route(request: &Request, port: u16, provider: &Provider) -> Response {
    let Some(host) = request.host.as_deref() else { return Response::error(400) };
    if !host_allowed(host, port) || request.has_origin {
        return Response::error(403);
    }
    let known = matches!(request.path.as_str(), "/v1/limits" | "/v1/health");
    if request.method != "GET" {
        return Response::error(if known { 405 } else { 404 });
    }
    match request.path.as_str() {
        "/v1/health" => Response::json(200, br#"{"ok":true}"#.to_vec()),
        "/v1/limits" => match provider() {
            Some(reading) => {
                Response::json(200, super::body::limits(&reading.snapshot, reading.warn_at_pct))
            }
            None => Response::error(503),
        },
        _ => Response::error(404),
    }
}

/// DNS-rebinding defence: a page on `evil.example` that resolves to 127.0.0.1 still sends its own
/// name as `Host`.
pub(super) fn host_allowed(host: &str, port: u16) -> bool {
    [format!("127.0.0.1:{port}"), format!("localhost:{port}")]
        .iter()
        .any(|allowed| host.eq_ignore_ascii_case(allowed))
}
