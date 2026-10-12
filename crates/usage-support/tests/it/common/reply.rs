//! What the test server answers with, and how each answer is written to the wire.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

use tokio::io::{AsyncWrite, AsyncWriteExt};

pub enum Reply {
    /// A complete response with a `Content-Length`.
    Full { status: u16, headers: Vec<(String, String)>, body: Vec<u8> },
    /// Chunked transfer coding, `pause` between chunks; `endless` repeats the last chunk until
    /// the client goes away.
    Chunked { status: u16, chunks: Vec<Vec<u8>>, pause: Duration, endless: bool },
    /// Declares `declared` body bytes, sends `sent` and closes the connection.
    Truncated { declared: usize, sent: Vec<u8> },
    /// Waits, then answers with the inner reply.
    After(Duration, Box<Reply>),
    /// Accepts the request and never answers.
    Silent,
    /// Closes the connection without a byte.
    Hangup,
    /// Writes these bytes as the entire response.
    Raw(Vec<u8>),
}

impl Reply {
    pub fn json(status: u16, body: &str) -> Self {
        Self::with_type(status, "application/json", body.as_bytes())
    }

    pub fn with_type(status: u16, content_type: &str, body: &[u8]) -> Self {
        Self::Full {
            status,
            headers: vec![("Content-Type".into(), content_type.into())],
            body: body.to_vec(),
        }
    }

    pub fn status(status: u16, headers: &[(&str, &str)]) -> Self {
        let headers = headers.iter().map(|(k, v)| ((*k).into(), (*v).into())).collect();
        Self::Full { status, headers, body: b"{}".to_vec() }
    }
}

/// What an endless chunked stream did: bytes written, and whether it stopped because the peer left.
#[derive(Default)]
pub struct StreamStats {
    pub bytes: AtomicUsize,
    pub peer_left: AtomicBool,
}

fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
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

fn json_type() -> [(String, String); 1] {
    [("Content-Type".to_owned(), "application/json".to_owned())]
}

pub async fn write_reply<S: AsyncWrite + Unpin>(
    stream: &mut S,
    mut reply: Reply,
    stats: &StreamStats,
) {
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
        Reply::Chunked { status, chunks, pause, endless } => {
            let start = head(status, &json_type(), "Transfer-Encoding: chunked\r\n");
            let _ = stream.write_all(&start).await;
            stream_chunks(stream, chunks, pause, endless, stats).await;
        }
        Reply::Truncated { declared, sent } => {
            let extra = format!("Content-Length: {declared}\r\n");
            let _ = stream.write_all(&head(200, &json_type(), &extra)).await;
            let _ = stream.write_all(&sent).await;
            let _ = stream.flush().await;
        }
        Reply::Silent => std::future::pending::<()>().await,
        Reply::Hangup => {}
        Reply::Raw(bytes) => {
            let _ = stream.write_all(&bytes).await;
        }
        Reply::After(..) => unreachable!("unwrapped above"),
    }
}

async fn stream_chunks<S: AsyncWrite + Unpin>(
    stream: &mut S,
    chunks: Vec<Vec<u8>>,
    pause: Duration,
    endless: bool,
    stats: &StreamStats,
) {
    let mut next = chunks.into_iter();
    let mut last = Vec::new();
    loop {
        let chunk = match next.next() {
            Some(chunk) => chunk,
            None if endless && !last.is_empty() => last.clone(),
            None => break,
        };
        let mut frame = format!("{:x}\r\n", chunk.len()).into_bytes();
        frame.extend_from_slice(&chunk);
        frame.extend_from_slice(b"\r\n");
        if stream.write_all(&frame).await.is_err() || stream.flush().await.is_err() {
            stats.peer_left.store(true, Ordering::SeqCst);
            return;
        }
        stats.bytes.fetch_add(frame.len(), Ordering::SeqCst);
        last = chunk;
        if !pause.is_zero() {
            tokio::time::sleep(pause).await;
        }
    }
    let _ = stream.write_all(b"0\r\n\r\n").await;
}
