//! The listener: one accept thread on loopback, one short-lived thread per connection, a cap on
//! how many at once. Dropping the handle stops accepting and joins the accept thread.

use std::fmt;
use std::io;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use usage_model::Snapshot;
use usage_model::settings::LOCAL_API_PORT;

use super::connection::serve;

/// Connections served at once; the rest are closed unanswered until one finishes.
const MAX_CONNECTIONS: usize = 8;
/// How often the accept loop looks at the stop flag while nobody connects.
const ACCEPT_POLL: Duration = Duration::from_millis(50);

/// What a request is answered from. `None` while the app has nothing to show yet (503).
pub struct Reading {
    pub snapshot: Snapshot,
    /// `general.warn_at_pct`, the status threshold.
    pub warn_at_pct: u8,
}

/// Called on a connection thread for every `GET /v1/limits`; must be quick and never block on I/O.
pub type Provider = Arc<dyn Fn() -> Option<Reading> + Send + Sync>;

#[derive(Clone, Copy)]
pub(super) struct Limits {
    pub read_deadline: Duration,
    pub write_timeout: Duration,
    pub max_connections: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            read_deadline: Duration::from_secs(5),
            write_timeout: Duration::from_secs(5),
            max_connections: MAX_CONNECTIONS,
        }
    }
}

/// Why the server did not start. The app keeps running without it; a later toggle retries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StartError {
    /// Another program (or another instance) holds the port.
    PortInUse,
    /// Anything else the OS refused; the kind only, never OS text.
    Io(io::ErrorKind),
}

impl fmt::Display for StartError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PortInUse => f.write_str("the port is in use"),
            Self::Io(kind) => write!(f, "the listener failed ({kind})"),
        }
    }
}

impl std::error::Error for StartError {}

impl From<io::Error> for StartError {
    fn from(err: io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::AddrInUse => Self::PortInUse,
            kind => Self::Io(kind),
        }
    }
}

/// A running server. Stops when dropped.
pub struct LocalApi {
    addr: SocketAddr,
    stop: Arc<AtomicBool>,
    accept: Option<JoinHandle<()>>,
}

impl fmt::Debug for LocalApi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalApi").field("port", &self.addr.port()).finish_non_exhaustive()
    }
}

impl LocalApi {
    /// Listens on `127.0.0.1:LOCAL_API_PORT`.
    pub fn start(provider: Provider) -> Result<Self, StartError> {
        Self::start_on(LOCAL_API_PORT, provider)
    }

    /// The same on another port (`0` picks a free one; see [`LocalApi::port`]). For tests.
    pub fn start_on(port: u16, provider: Provider) -> Result<Self, StartError> {
        Self::start_with(port, provider, Limits::default())
    }

    pub(super) fn start_with(
        port: u16,
        provider: Provider,
        limits: Limits,
    ) -> Result<Self, StartError> {
        // Loopback IPv4 and nothing else. Rust sets no address reuse on Windows, so a second
        // listener cannot share the port there.
        let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port))?;
        listener.set_nonblocking(true)?;
        let addr = listener.local_addr()?;
        let port = addr.port();
        let stop = Arc::new(AtomicBool::new(false));
        let accept = thread::Builder::new().name("local-api".to_owned()).spawn({
            let stop = Arc::clone(&stop);
            move || accept_loop(&listener, port, &provider, limits, &stop)
        })?;
        Ok(Self { addr, stop, accept: Some(accept) })
    }

    /// The port actually bound.
    pub fn port(&self) -> u16 {
        self.addr.port()
    }

    /// The address actually bound: always `127.0.0.1`.
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }
}

impl Drop for LocalApi {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(accept) = self.accept.take() {
            let _ = accept.join();
        }
    }
}

/// Releases a connection slot when its thread ends, however it ends.
struct Slot(Arc<AtomicUsize>);

impl Drop for Slot {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

fn accept_loop(
    listener: &TcpListener,
    port: u16,
    provider: &Provider,
    limits: Limits,
    stop: &AtomicBool,
) {
    let active = Arc::new(AtomicUsize::new(0));
    while !stop.load(Ordering::SeqCst) {
        let (stream, peer) = match listener.accept() {
            Ok(accepted) => accepted,
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(ACCEPT_POLL);
                continue;
            }
            Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
            // A connection that died in the backlog, or a transient resource error: look again.
            Err(_) => {
                thread::sleep(ACCEPT_POLL);
                continue;
            }
        };
        if !matches!(peer, SocketAddr::V4(addr) if addr.ip().is_loopback()) {
            continue;
        }
        if active.fetch_add(1, Ordering::SeqCst) >= limits.max_connections {
            active.fetch_sub(1, Ordering::SeqCst);
            continue;
        }
        let slot = Slot(Arc::clone(&active));
        let provider = Arc::clone(provider);
        // A failed spawn drops the closure, and with it the slot and the stream.
        let _ = thread::Builder::new().name("local-api-conn".to_owned()).spawn(move || {
            let _slot = slot;
            // Accepted sockets may inherit the listener's non-blocking mode on some platforms.
            if stream.set_nonblocking(false).is_ok() {
                serve(stream, port, &provider, limits);
            }
        });
    }
}
