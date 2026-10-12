//! A tiny HTTP/1.1 server on `127.0.0.1:0` for client tests: scripted replies, a request log and
//! a connection counter, so tests can prove that a refused request never reached the network.

use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio_rustls::TlsAcceptor;

use super::reply::{Reply, StreamStats, write_reply};

#[derive(Clone, Debug)]
pub struct Captured {
    pub method: String,
    pub target: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Captured {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.iter().find(|(k, _)| k.eq_ignore_ascii_case(name)).map(|(_, v)| v.as_str())
    }
}

type Handler = Box<dyn Fn(&Captured) -> Reply + Send + Sync>;

struct Shared {
    handler: Handler,
    accepted: AtomicUsize,
    log: Mutex<Vec<Captured>>,
    stream: StreamStats,
}

pub struct TestServer {
    pub addr: SocketAddr,
    scheme: &'static str,
    shared: Arc<Shared>,
    task: JoinHandle<()>,
}

impl TestServer {
    pub async fn start(handler: impl Fn(&Captured) -> Reply + Send + Sync + 'static) -> Self {
        Self::launch(None, handler).await
    }

    pub async fn start_tls(
        config: Arc<rustls::ServerConfig>,
        handler: impl Fn(&Captured) -> Reply + Send + Sync + 'static,
    ) -> Self {
        Self::launch(Some(TlsAcceptor::from(config)), handler).await
    }

    async fn launch(
        acceptor: Option<TlsAcceptor>,
        handler: impl Fn(&Captured) -> Reply + Send + Sync + 'static,
    ) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let shared = Arc::new(Shared {
            handler: Box::new(handler),
            accepted: AtomicUsize::new(0),
            log: Mutex::new(Vec::new()),
            stream: StreamStats::default(),
        });
        let scheme = if acceptor.is_some() { "https" } else { "http" };
        let loop_shared = Arc::clone(&shared);
        let task = tokio::spawn(async move {
            while let Ok((tcp, _)) = listener.accept().await {
                loop_shared.accepted.fetch_add(1, Ordering::SeqCst);
                let (shared, acceptor) = (Arc::clone(&loop_shared), acceptor.clone());
                tokio::spawn(async move {
                    match acceptor {
                        Some(acceptor) => {
                            if let Ok(tls) = acceptor.accept(tcp).await {
                                serve(tls, &shared).await;
                            }
                        }
                        None => serve(tcp, &shared).await,
                    }
                });
            }
        });
        Self { addr, scheme, shared, task }
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}://127.0.0.1:{}{path}", self.scheme, self.addr.port())
    }

    pub fn requests(&self) -> Vec<Captured> {
        self.shared.log.lock().unwrap().clone()
    }

    pub fn last_request(&self) -> Captured {
        self.requests().pop().expect("the server saw no request")
    }

    pub fn connections(&self) -> usize {
        self.shared.accepted.load(Ordering::SeqCst)
    }

    /// Bytes written by an endless chunked stream, and whether it ended because the peer left.
    pub fn streamed(&self) -> (usize, bool) {
        let stats = &self.shared.stream;
        (stats.bytes.load(Ordering::SeqCst), stats.peer_left.load(Ordering::SeqCst))
    }

    /// Proves the client never opened a connection: waits long enough for a queued connection to
    /// be accepted, then checks the counter.
    pub async fn assert_untouched(&self) {
        tokio::time::sleep(Duration::from_millis(200)).await;
        assert_eq!(self.connections(), 0, "the request must be refused before any I/O");
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn serve<S: AsyncRead + AsyncWrite + Unpin>(mut stream: S, shared: &Shared) {
    let Some(request) = read_request(&mut stream).await else { return };
    shared.log.lock().unwrap().push(request.clone());
    let reply = (shared.handler)(&request);
    write_reply(&mut stream, reply, &shared.stream).await;
    let _ = stream.shutdown().await;
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

async fn read_request<S: AsyncRead + Unpin>(stream: &mut S) -> Option<Captured> {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 4096];
    let head_end = loop {
        if let Some(pos) = find(&buf, b"\r\n\r\n") {
            break pos + 4;
        }
        if buf.len() > 64 * 1024 {
            return None;
        }
        let n = stream.read(&mut chunk).await.ok()?;
        if n == 0 {
            return None;
        }
        buf.extend_from_slice(&chunk[..n]);
    };
    let head = String::from_utf8_lossy(&buf[..head_end]).into_owned();
    let mut lines = head.split("\r\n");
    let mut first = lines.next()?.split(' ');
    let (method, target) = (first.next()?.to_owned(), first.next()?.to_owned());
    let headers: Vec<(String, String)> = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(k, v)| (k.trim().to_owned(), v.trim().to_owned()))
        .collect();
    let length = headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("content-length"))
        .and_then(|(_, v)| v.parse::<usize>().ok())
        .unwrap_or(0);
    let mut body = buf[head_end..].to_vec();
    while body.len() < length {
        let n = stream.read(&mut chunk).await.ok()?;
        if n == 0 {
            return None;
        }
        body.extend_from_slice(&chunk[..n]);
    }
    Some(Captured { method, target, headers, body })
}
