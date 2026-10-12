//! A throwaway HTTP/1.1 server on 127.0.0.1 (ephemeral port) for the OAuth tests. It answers one
//! request per connection from a scripted handler, logs every request head and counts the
//! connections, so a test can prove that a request never left the process. Only fabricated data
//! ever passes through it.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const MAX_HEAD_BYTES: usize = 64 * 1024;
const READ_TIMEOUT: Duration = Duration::from_secs(5);
/// How long a silent connection sleeps between looks at the stop flag.
const SILENT_POLL: Duration = Duration::from_millis(20);

#[derive(Clone, Debug)]
pub(crate) struct Request {
    pub method: String,
    pub target: String,
    pub headers: Vec<(String, String)>,
}

impl Request {
    pub(crate) fn header(&self, name: &str) -> Option<&str> {
        let found = self.headers.iter().find(|(key, _)| key.eq_ignore_ascii_case(name));
        found.map(|(_, value)| value.as_str())
    }
}

pub(crate) enum Reply {
    /// A complete response with a `Content-Length`.
    Full { status: u16, headers: Vec<(String, String)>, body: Vec<u8> },
    /// These bytes are the whole response; the connection then closes.
    Raw(Vec<u8>),
    /// Accepts the request and never answers.
    Silent,
}

impl Reply {
    pub(crate) fn json(body: &str) -> Self {
        Self::typed(200, "application/json", body.as_bytes())
    }

    pub(crate) fn typed(status: u16, content_type: &str, body: &[u8]) -> Self {
        let headers = vec![("Content-Type".to_owned(), content_type.to_owned())];
        Self::Full { status, headers, body: body.to_vec() }
    }

    /// An empty JSON object with the given status and extra headers.
    pub(crate) fn status(status: u16, headers: &[(&str, &str)]) -> Self {
        let mut all = vec![("Content-Type".to_owned(), "application/json".to_owned())];
        all.extend(headers.iter().map(|(key, value)| ((*key).to_owned(), (*value).to_owned())));
        Self::Full { status, headers: all, body: b"{}".to_vec() }
    }
}

type Handler = Box<dyn Fn(&Request) -> Reply + Send + Sync>;

struct Shared {
    handler: Handler,
    log: Mutex<Vec<Request>>,
    connections: AtomicUsize,
    stop: AtomicBool,
}

pub(crate) struct FakeServer {
    addr: SocketAddr,
    shared: Arc<Shared>,
}

impl FakeServer {
    pub(crate) fn start(handler: impl Fn(&Request) -> Reply + Send + Sync + 'static) -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let addr = listener.local_addr().unwrap();
        let shared = Arc::new(Shared {
            handler: Box::new(handler),
            log: Mutex::new(Vec::new()),
            connections: AtomicUsize::new(0),
            stop: AtomicBool::new(false),
        });
        let accepting = Arc::clone(&shared);
        thread::spawn(move || {
            for stream in listener.incoming() {
                if accepting.stop.load(Ordering::SeqCst) {
                    break;
                }
                let Ok(stream) = stream else { continue };
                accepting.connections.fetch_add(1, Ordering::SeqCst);
                let shared = Arc::clone(&accepting);
                thread::spawn(move || serve(stream, &shared));
            }
        });
        Self { addr, shared }
    }

    pub(crate) fn json(body: &str) -> Self {
        let body = body.to_owned();
        Self::start(move |_| Reply::json(&body))
    }

    /// What the injected endpoint points at.
    pub(crate) fn base(&self) -> String {
        format!("http://127.0.0.1:{}", self.addr.port())
    }

    pub(crate) fn requests(&self) -> Vec<Request> {
        self.shared.log.lock().unwrap().clone()
    }

    /// Connections accepted so far; any request implies one.
    pub(crate) fn connections(&self) -> usize {
        self.shared.connections.load(Ordering::SeqCst)
    }

    /// Gives a stray connection time to arrive, then checks that none did.
    pub(crate) fn assert_untouched(&self) {
        thread::sleep(Duration::from_millis(100));
        assert_eq!(self.connections(), 0, "no request may reach the server");
    }
}

impl Drop for FakeServer {
    fn drop(&mut self) {
        self.shared.stop.store(true, Ordering::SeqCst);
        // A throwaway connection wakes the blocked accept so the thread can see the flag. The
        // thread is not joined: if the wake-up failed, a hung join would hang the test run.
        let _ = TcpStream::connect(self.addr);
    }
}

fn serve(mut stream: TcpStream, shared: &Shared) {
    let _ = stream.set_read_timeout(Some(READ_TIMEOUT));
    let Some(request) = read_head(&mut stream) else { return };
    shared.log.lock().unwrap().push(request.clone());
    match (shared.handler)(&request) {
        Reply::Full { status, headers, body } => {
            let mut head = format!("HTTP/1.1 {status} Status\r\nConnection: close\r\n");
            head.push_str(&format!("Content-Length: {}\r\n", body.len()));
            for (name, value) in headers {
                head.push_str(&format!("{name}: {value}\r\n"));
            }
            head.push_str("\r\n");
            let _ = stream.write_all(head.as_bytes());
            let _ = stream.write_all(&body);
        }
        Reply::Raw(bytes) => {
            let _ = stream.write_all(&bytes);
        }
        Reply::Silent => {
            while !shared.stop.load(Ordering::SeqCst) {
                thread::sleep(SILENT_POLL);
            }
        }
    }
}

/// The request line and headers; the tests only send bodiless GETs.
fn read_head(stream: &mut TcpStream) -> Option<Request> {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 2048];
    while !buf.windows(4).any(|w| w == b"\r\n\r\n") {
        if buf.len() > MAX_HEAD_BYTES {
            return None;
        }
        let read = stream.read(&mut chunk).ok().filter(|n| *n > 0)?;
        buf.extend_from_slice(&chunk[..read]);
    }
    let text = String::from_utf8_lossy(&buf);
    let mut lines = text.split("\r\n");
    let mut first = lines.next()?.split(' ');
    let (method, target) = (first.next()?.to_owned(), first.next()?.to_owned());
    let headers = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(key, value)| (key.trim().to_owned(), value.trim().to_owned()))
        .collect();
    Some(Request { method, target, headers })
}
