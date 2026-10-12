//! Test doubles: a language server stand-in on `127.0.0.1:0` (plain or self-signed HTTPS) with
//! scripted replies, a request log and a connection counter, and a discovery that returns what it
//! is told. No test ever talks to a real Antigravity.

mod tls;
mod wire;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use serde_json::Value;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio_rustls::TlsAcceptor;
use usage_model::Ctx;
use usage_support::Secret;

use crate::cli::{Cli, CliError};
use crate::discovery::{Discovery, DiscoveryError, Scheme, Target};
pub(crate) use wire::{Reply, Request};

type Handler = Box<dyn Fn(&Request) -> Reply + Send + Sync>;

struct Shared {
    handler: Handler,
    accepted: AtomicUsize,
    log: Mutex<Vec<Request>>,
}

pub(crate) struct FakeServer {
    port: u16,
    scheme: Scheme,
    shared: Arc<Shared>,
    task: JoinHandle<()>,
}

impl FakeServer {
    pub(crate) async fn http(handler: impl Fn(&Request) -> Reply + Send + Sync + 'static) -> Self {
        Self::launch(None, handler).await
    }

    /// Serves a certificate no client would trust by default.
    pub(crate) async fn https(handler: impl Fn(&Request) -> Reply + Send + Sync + 'static) -> Self {
        Self::launch(Some(tls::acceptor()), handler).await
    }

    async fn launch(
        acceptor: Option<TlsAcceptor>,
        handler: impl Fn(&Request) -> Reply + Send + Sync + 'static,
    ) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let shared = Arc::new(Shared {
            handler: Box::new(handler),
            accepted: AtomicUsize::new(0),
            log: Mutex::new(Vec::new()),
        });
        let scheme = if acceptor.is_some() { Scheme::Https } else { Scheme::Http };
        let served = Arc::clone(&shared);
        let task = tokio::spawn(async move {
            while let Ok((tcp, _)) = listener.accept().await {
                served.accepted.fetch_add(1, Ordering::SeqCst);
                let (shared, acceptor) = (Arc::clone(&served), acceptor.clone());
                tokio::spawn(async move {
                    match acceptor {
                        Some(acceptor) => {
                            if let Ok(stream) = acceptor.accept(tcp).await {
                                serve(stream, &shared).await;
                            }
                        }
                        None => serve(tcp, &shared).await,
                    }
                });
            }
        });
        Self { port, scheme, shared, task }
    }

    pub(crate) fn target(&self, token: &str) -> Target {
        Target { scheme: self.scheme, port: self.port, token: Secret::new(token) }
    }

    pub(crate) fn requests(&self) -> Vec<Request> {
        self.shared.log.lock().unwrap().clone()
    }

    pub(crate) fn connections(&self) -> usize {
        self.shared.accepted.load(Ordering::SeqCst)
    }
}

impl Drop for FakeServer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn serve<S: AsyncRead + AsyncWrite + Unpin>(mut stream: S, shared: &Shared) {
    let Some(request) = wire::read_request(&mut stream).await else { return };
    shared.log.lock().unwrap().push(request.clone());
    wire::write_reply(&mut stream, (shared.handler)(&request)).await;
    let _ = stream.shutdown().await;
}

/// A port nothing listens on: bound to learn a free number, then released.
pub(crate) async fn closed_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    listener.local_addr().unwrap().port()
}

/// Answers like the real server: 401 unless the CSRF header carries the token, then one reply per
/// RPC, and 404 for anything else.
#[derive(Clone, Debug)]
pub(crate) struct Script {
    pub token: String,
    pub status: Reply,
    pub summary: Reply,
}

impl Script {
    /// `summary: None` is the IDE's server, which has no such method.
    pub(crate) fn new(token: &str, status: &str, summary: Option<&str>) -> Self {
        Self {
            token: token.to_owned(),
            status: Reply::json(200, status),
            summary: summary.map_or_else(|| Reply::status(404), |body| Reply::json(200, body)),
        }
    }

    pub(crate) fn reply(&self, request: &Request) -> Reply {
        if request.header("x-codeium-csrf-token") != Some(self.token.as_str()) {
            return Reply::status(401);
        }
        match request.method() {
            "GetUserStatus" => self.status.clone(),
            "RetrieveUserQuotaSummary" => self.summary.clone(),
            _ => Reply::status(404),
        }
    }

    pub(crate) async fn serve_http(self) -> FakeServer {
        FakeServer::http(move |request| self.reply(request)).await
    }

    pub(crate) async fn serve_https(self) -> FakeServer {
        FakeServer::https(move |request| self.reply(request)).await
    }
}

/// Discovery that returns what it was given and counts how often it was asked.
pub(crate) struct StaticDiscovery {
    result: Result<Vec<Target>, DiscoveryError>,
    calls: AtomicUsize,
}

impl StaticDiscovery {
    pub(crate) fn new(result: Result<Vec<Target>, DiscoveryError>) -> Self {
        Self { result, calls: AtomicUsize::new(0) }
    }

    pub(crate) fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl Discovery for StaticDiscovery {
    async fn find(&self) -> Result<Vec<Target>, DiscoveryError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.result.clone()
    }
}

/// No CLI installed: the IDE-only machine.
pub(crate) struct AbsentCli;

impl Cli for AbsentCli {
    async fn usage(&self, _ctx: &Ctx) -> Result<Value, CliError> {
        Err(CliError::Absent)
    }
}

/// A CLI that answers what it was given and counts how often it was asked.
pub(crate) struct ScriptedCli {
    result: Result<Value, CliError>,
    calls: AtomicUsize,
}

impl ScriptedCli {
    pub(crate) fn new(result: Result<Value, CliError>) -> Self {
        Self { result, calls: AtomicUsize::new(0) }
    }

    pub(crate) fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl Cli for ScriptedCli {
    async fn usage(&self, _ctx: &Ctx) -> Result<Value, CliError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.result.clone()
    }
}
