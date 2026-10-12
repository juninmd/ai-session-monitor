//! A scripted HTTP/1.1 server on 127.0.0.1 (ephemeral port) that logs every request, so tests can
//! prove where a credential went and that gated calls never reached the network.

use std::io::{ErrorKind, Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Request {
    pub method: String,
    pub target: String,
    pub headers: Vec<(String, String)>,
}

impl Request {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.iter().find(|(k, _)| k.eq_ignore_ascii_case(name)).map(|(_, v)| v.as_str())
    }
}

pub enum Reply {
    /// A complete response; `Content-Length` is added.
    Full { status: u16, headers: Vec<(String, String)>, body: Vec<u8> },
    /// A 200 JSON response with no length that streams `bytes` spaces, then closes.
    Streamed { bytes: usize },
    /// Waits, then answers with the inner reply.
    Delayed(Duration, Box<Reply>),
    /// Closes the connection without a byte.
    Hangup,
}

impl Reply {
    pub fn json(status: u16, body: &str) -> Self {
        let headers = vec![("Content-Type".to_owned(), "application/json".to_owned())];
        Self::Full { status, headers, body: body.as_bytes().to_vec() }
    }

    pub fn with_headers(status: u16, headers: &[(&str, &str)], body: &str) -> Self {
        let headers = headers.iter().map(|(k, v)| ((*k).to_owned(), (*v).to_owned())).collect();
        Self::Full { status, headers, body: body.as_bytes().to_vec() }
    }
}

struct State {
    log: Mutex<Vec<Request>>,
    stop: AtomicBool,
    connections: AtomicUsize,
    responder: Box<dyn Fn(&Request) -> Reply + Send + Sync>,
}

pub struct FakeServer {
    addr: SocketAddr,
    state: Arc<State>,
    accept: Option<JoinHandle<()>>,
}

impl FakeServer {
    pub fn start(responder: impl Fn(&Request) -> Reply + Send + Sync + 'static) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let addr = listener.local_addr().unwrap();
        let state = Arc::new(State {
            log: Mutex::new(Vec::new()),
            stop: AtomicBool::new(false),
            connections: AtomicUsize::new(0),
            responder: Box::new(responder),
        });
        let shared = Arc::clone(&state);
        let accept = thread::spawn(move || {
            while !shared.stop.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        shared.connections.fetch_add(1, Ordering::SeqCst);
                        let conn_state = Arc::clone(&shared);
                        thread::spawn(move || serve(stream, &conn_state));
                    }
                    Err(e) if e.kind() == ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(_) => break,
                }
            }
        });
        Self { addr, state, accept: Some(accept) }
    }

    /// A server that always answers `reply()`.
    pub fn always(reply: impl Fn() -> Reply + Send + Sync + 'static) -> Self {
        Self::start(move |_| reply())
    }

    pub fn base_url(&self) -> String {
        format!("http://{}", self.addr)
    }

    pub fn requests(&self) -> Vec<Request> {
        self.state.log.lock().unwrap().clone()
    }

    /// Connections accepted, including ones that never sent a request.
    pub fn connections(&self) -> usize {
        self.state.connections.load(Ordering::SeqCst)
    }

    /// Waits long enough for a queued connection to be accepted, then checks the counters.
    pub fn assert_untouched(&self) {
        thread::sleep(Duration::from_millis(150));
        assert_eq!(self.connections(), 0, "no connection may be opened");
        assert!(self.requests().is_empty(), "no request may be sent");
    }
}

impl Drop for FakeServer {
    fn drop(&mut self) {
        self.state.stop.store(true, Ordering::SeqCst);
        if let Some(handle) = self.accept.take() {
            let _ = handle.join();
        }
    }
}

/// A URL on a port nothing listens on: connecting is refused immediately.
pub fn closed_port_url() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);
    url
}

fn serve(mut stream: TcpStream, state: &State) {
    let _ = stream.set_nonblocking(false);
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let Some(request) = read_request(&mut stream) else { return };
    state.log.lock().unwrap().push(request.clone());
    write_reply(&mut stream, (state.responder)(&request), state);
    let _ = stream.shutdown(Shutdown::Both);
}

fn read_request(stream: &mut TcpStream) -> Option<Request> {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 1024];
    while !buf.windows(4).any(|w| w == b"\r\n\r\n") {
        if buf.len() > 16 * 1024 {
            return None;
        }
        let n = stream.read(&mut chunk).ok()?;
        if n == 0 {
            return None;
        }
        buf.extend_from_slice(&chunk[..n]);
    }
    let text = String::from_utf8_lossy(&buf).into_owned();
    let mut lines = text.split("\r\n");
    let mut first = lines.next()?.split(' ');
    let (method, target) = (first.next()?.to_owned(), first.next()?.to_owned());
    let headers = lines
        .take_while(|line| !line.is_empty())
        .filter_map(|line| line.split_once(':'))
        .map(|(k, v)| (k.trim().to_owned(), v.trim().to_owned()))
        .collect();
    Some(Request { method, target, headers })
}

fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        302 => "Found",
        401 => "Unauthorized",
        402 => "Payment Required",
        403 => "Forbidden",
        404 => "Not Found",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        _ => "Status",
    }
}

fn write_reply(stream: &mut TcpStream, reply: Reply, state: &State) {
    match reply {
        Reply::Hangup => {}
        Reply::Delayed(delay, inner) => {
            let mut waited = Duration::ZERO;
            while waited < delay && !state.stop.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(10));
                waited += Duration::from_millis(10);
            }
            write_reply(stream, *inner, state);
        }
        Reply::Full { status, headers, body } => {
            let mut head = format!("HTTP/1.1 {status} {}\r\nConnection: close\r\n", reason(status));
            head.push_str(&format!("Content-Length: {}\r\n", body.len()));
            for (name, value) in headers {
                head.push_str(&format!("{name}: {value}\r\n"));
            }
            head.push_str("\r\n");
            let _ = stream.write_all(head.as_bytes());
            let _ = stream.write_all(&body);
        }
        Reply::Streamed { bytes } => {
            let head =
                "HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Type: application/json\r\n\r\n";
            let _ = stream.write_all(head.as_bytes());
            let block = [b' '; 4096];
            let mut sent = 0;
            while sent < bytes && stream.write_all(&block).is_ok() {
                sent += block.len();
            }
        }
    }
}
