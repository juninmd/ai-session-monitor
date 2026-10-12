//! A tiny HTTP/1.1 server on `127.0.0.1:0` that plays a LiteLLM proxy: routed or scripted replies,
//! a request log and a connection counter, so a test can prove that a gate never reached the
//! network and see exactly which requests a poll made.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio_rustls::TlsAcceptor;

use super::reply::{Reply, write_reply};
use super::seen::Seen;

const MAX_HEAD_BYTES: usize = 64 * 1024;

type Router = Box<dyn Fn(&Seen) -> Reply + Send + Sync>;

struct Shared {
    router: Router,
    accepted: AtomicUsize,
    log: Mutex<Vec<Seen>>,
}

pub struct Fake {
    addr: SocketAddr,
    scheme: &'static str,
    shared: Arc<Shared>,
    task: JoinHandle<()>,
}

impl Fake {
    pub async fn start(router: impl Fn(&Seen) -> Reply + Send + Sync + 'static) -> Self {
        Self::launch(None, router).await
    }

    pub async fn start_tls(
        config: Arc<rustls::ServerConfig>,
        router: impl Fn(&Seen) -> Reply + Send + Sync + 'static,
    ) -> Self {
        Self::launch(Some(TlsAcceptor::from(config)), router).await
    }

    /// Answers each known path with its reply and anything else with a LiteLLM-style 404.
    pub async fn routes(routes: impl IntoIterator<Item = (&'static str, Reply)>) -> Self {
        let table: HashMap<String, Reply> =
            routes.into_iter().map(|(path, reply)| (path.to_owned(), reply)).collect();
        Self::start(move |seen| {
            table
                .get(seen.path())
                .cloned()
                .unwrap_or_else(|| Reply::json_status(404, r#"{"detail":"Not Found"}"#))
        })
        .await
    }

    async fn launch(
        acceptor: Option<TlsAcceptor>,
        router: impl Fn(&Seen) -> Reply + Send + Sync + 'static,
    ) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let shared = Arc::new(Shared {
            router: Box::new(router),
            accepted: AtomicUsize::new(0),
            log: Mutex::new(Vec::new()),
        });
        let scheme = if acceptor.is_some() { "https" } else { "http" };
        let accepting = Arc::clone(&shared);
        let task = tokio::spawn(async move {
            while let Ok((tcp, _)) = listener.accept().await {
                accepting.accepted.fetch_add(1, Ordering::SeqCst);
                let (shared, acceptor) = (Arc::clone(&accepting), acceptor.clone());
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

    /// `scheme://127.0.0.1:port`, ready to be a base URL.
    pub fn base(&self) -> String {
        format!("{}://127.0.0.1:{}", self.scheme, self.addr.port())
    }

    pub fn requests(&self) -> Vec<Seen> {
        self.shared.log.lock().unwrap().clone()
    }

    pub fn targets(&self) -> Vec<String> {
        self.requests().into_iter().map(|seen| seen.target).collect()
    }

    pub fn connections(&self) -> usize {
        self.shared.accepted.load(Ordering::SeqCst)
    }

    /// Proves the collector never opened a connection: waits long enough for a queued connection
    /// to be accepted, then checks the counter.
    pub async fn assert_untouched(&self) {
        tokio::time::sleep(Duration::from_millis(200)).await;
        assert_eq!(self.connections(), 0, "the poll must stop before any I/O");
        assert!(self.requests().is_empty());
    }
}

impl Drop for Fake {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn serve<S: AsyncRead + AsyncWrite + Unpin>(mut stream: S, shared: &Shared) {
    let Some(request) = read_request(&mut stream).await else { return };
    shared.log.lock().unwrap().push(request.clone());
    let reply = (shared.router)(&request);
    write_reply(&mut stream, reply).await;
    let _ = stream.shutdown().await;
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|window| window == needle)
}

async fn read_request<S: AsyncRead + Unpin>(stream: &mut S) -> Option<Seen> {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 4096];
    let head_end = loop {
        if let Some(at) = find(&buf, b"\r\n\r\n") {
            break at;
        }
        if buf.len() > MAX_HEAD_BYTES {
            return None;
        }
        let read = stream.read(&mut chunk).await.ok()?;
        if read == 0 {
            return None;
        }
        buf.extend_from_slice(&chunk[..read]);
    };
    let head = String::from_utf8_lossy(&buf[..head_end]).into_owned();
    let mut lines = head.split("\r\n");
    let mut first = lines.next()?.split(' ');
    let (method, target) = (first.next()?.to_owned(), first.next()?.to_owned());
    let headers = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(k, v)| (k.trim().to_owned(), v.trim().to_owned()))
        .collect();
    Some(Seen { method, target, headers })
}
