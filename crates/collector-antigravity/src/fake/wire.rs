//! The two halves of HTTP/1.1 the stand-in needs: read one request, write one reply.

use std::time::Duration;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

const MAX_HEAD: usize = 64 * 1024;

#[derive(Clone, Debug)]
pub(crate) struct Request {
    /// Request target, `/exa.language_server_pb.LanguageServerService/GetUserStatus`.
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl Request {
    pub(crate) fn header(&self, name: &str) -> Option<&str> {
        let found = self.headers.iter().find(|(key, _)| key.eq_ignore_ascii_case(name));
        found.map(|(_, value)| value.as_str())
    }

    /// `GetUserStatus`, `RetrieveUserQuotaSummary`.
    pub(crate) fn method(&self) -> &str {
        self.path.rsplit('/').next().unwrap_or_default()
    }
}

#[derive(Clone, Debug)]
pub(crate) enum Reply {
    Json(u16, String),
    Html(u16, String),
    /// Written as the entire response, whatever it is.
    Raw(Vec<u8>),
    /// Accepts the request and never answers.
    Silent,
    /// Closes the connection without a byte.
    Hangup,
    /// Waits, then answers with the inner reply.
    After(Duration, Box<Reply>),
}

impl Reply {
    pub(crate) fn json(status: u16, body: &str) -> Self {
        Self::Json(status, body.to_owned())
    }

    pub(crate) fn status(status: u16) -> Self {
        Self::Json(status, "{}".to_owned())
    }
}

fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        401 => "Unauthorized",
        404 => "Not Found",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        _ => "Status",
    }
}

fn full(status: u16, content_type: &str, body: &str) -> Vec<u8> {
    let head = format!(
        "HTTP/1.1 {status} {}\r\nConnection: close\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\n\r\n",
        reason(status),
        body.len()
    );
    [head.as_bytes(), body.as_bytes()].concat()
}

pub(crate) async fn write_reply<S: AsyncWrite + Unpin>(stream: &mut S, mut reply: Reply) {
    while let Reply::After(delay, inner) = reply {
        tokio::time::sleep(delay).await;
        reply = *inner;
    }
    let bytes = match reply {
        Reply::Json(status, body) => full(status, "application/json", &body),
        Reply::Html(status, body) => full(status, "text/html", &body),
        Reply::Raw(bytes) => bytes,
        Reply::Silent => std::future::pending().await,
        Reply::Hangup | Reply::After(..) => return,
    };
    let _ = stream.write_all(&bytes).await;
    let _ = stream.flush().await;
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|window| window == needle)
}

pub(crate) async fn read_request<S: AsyncRead + Unpin>(stream: &mut S) -> Option<Request> {
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 4096];
    let head_end = loop {
        if let Some(at) = find(&buffer, b"\r\n\r\n") {
            break at + 4;
        }
        if buffer.len() > MAX_HEAD {
            return None;
        }
        match stream.read(&mut chunk).await {
            Ok(0) | Err(_) => return None,
            Ok(read) => buffer.extend_from_slice(&chunk[..read]),
        }
    };
    let head = String::from_utf8_lossy(&buffer[..head_end]).into_owned();
    let mut lines = head.split("\r\n");
    let path = lines.next()?.split(' ').nth(1)?.to_owned();
    let headers: Vec<(String, String)> = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(key, value)| (key.trim().to_owned(), value.trim().to_owned()))
        .collect();
    let length = headers
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case("content-length"))
        .and_then(|(_, value)| value.parse::<usize>().ok())
        .unwrap_or(0);
    let mut body = buffer[head_end..].to_vec();
    while body.len() < length {
        match stream.read(&mut chunk).await {
            Ok(0) | Err(_) => return None,
            Ok(read) => body.extend_from_slice(&chunk[..read]),
        }
    }
    Some(Request { path, headers, body: String::from_utf8_lossy(&body).into_owned() })
}
