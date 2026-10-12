//! What the fake proxy answers with, and how each answer is written to the wire.

use std::time::Duration;

use tokio::io::{AsyncWrite, AsyncWriteExt};

const FILLER_BLOCK: usize = 64 * 1024;

#[derive(Clone, Debug)]
pub enum Reply {
    /// A complete response with a `Content-Length`.
    Full { status: u16, headers: Vec<(String, String)>, body: Vec<u8> },
    /// `bytes` of filler, declared up front or chunked. It is never valid JSON, so only a size cap
    /// can be what rejects it.
    Big { bytes: usize, chunked: bool },
    /// Waits, then answers with the inner reply.
    After(Duration, Box<Reply>),
    /// Closes the connection without a byte.
    Hangup,
}

impl Reply {
    pub fn json(body: &str) -> Self {
        Self::typed(200, "application/json", body)
    }

    pub fn json_status(status: u16, body: &str) -> Self {
        Self::typed(status, "application/json", body)
    }

    pub fn typed(status: u16, content_type: &str, body: &str) -> Self {
        Self::Full {
            status,
            headers: vec![("Content-Type".to_owned(), content_type.to_owned())],
            body: body.as_bytes().to_vec(),
        }
    }

    /// An empty JSON object with the given status and extra headers.
    pub fn status(status: u16, headers: &[(&str, &str)]) -> Self {
        let mut all = vec![("Content-Type".to_owned(), "application/json".to_owned())];
        all.extend(headers.iter().map(|(k, v)| ((*k).to_owned(), (*v).to_owned())));
        Self::Full { status, headers: all, body: b"{}".to_vec() }
    }

    pub fn after(self, delay: Duration) -> Self {
        Self::After(delay, Box::new(self))
    }
}

fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        301 => "Moved Permanently",
        302 => "Found",
        307 => "Temporary Redirect",
        308 => "Permanent Redirect",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        _ => "Status",
    }
}

fn head(status: u16, headers: &[(String, String)], extra: &str) -> Vec<u8> {
    let mut text = format!("HTTP/1.1 {status} {}\r\nConnection: close\r\n", reason(status));
    for (name, value) in headers {
        text.push_str(&format!("{name}: {value}\r\n"));
    }
    text.push_str(extra);
    text.push_str("\r\n");
    text.into_bytes()
}

pub async fn write_reply<S: AsyncWrite + Unpin>(stream: &mut S, mut reply: Reply) {
    while let Reply::After(delay, inner) = reply {
        tokio::time::sleep(delay).await;
        reply = *inner;
    }
    match reply {
        Reply::Full { status, headers, body } => {
            let extra = format!("Content-Length: {}\r\n", body.len());
            let _ = stream.write_all(&head(status, &headers, &extra)).await;
            let _ = stream.write_all(&body).await;
        }
        Reply::Big { bytes, chunked } => write_filler(stream, bytes, chunked).await,
        Reply::Hangup => {}
        Reply::After(..) => unreachable!("unwrapped above"),
    }
}

async fn write_filler<S: AsyncWrite + Unpin>(stream: &mut S, bytes: usize, chunked: bool) {
    let json = [("Content-Type".to_owned(), "application/json".to_owned())];
    let extra = if chunked {
        "Transfer-Encoding: chunked\r\n".to_owned()
    } else {
        format!("Content-Length: {bytes}\r\n")
    };
    if stream.write_all(&head(200, &json, &extra)).await.is_err() {
        return;
    }
    let block = vec![b'a'; FILLER_BLOCK];
    let mut left = bytes;
    while left > 0 {
        let size = left.min(FILLER_BLOCK);
        let mut frame = Vec::with_capacity(size + 16);
        if chunked {
            frame.extend_from_slice(format!("{size:x}\r\n").as_bytes());
        }
        frame.extend_from_slice(&block[..size]);
        if chunked {
            frame.extend_from_slice(b"\r\n");
        }
        // The client leaves as soon as it has seen enough; that is not a test failure.
        if stream.write_all(&frame).await.is_err() {
            return;
        }
        left -= size;
    }
    if chunked {
        let _ = stream.write_all(b"0\r\n\r\n").await;
    }
}
