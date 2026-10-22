//! The bounds: deadline, connection cap, a failing provider.

use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use super::super::server::Limits;
use super::super::{LocalApi, Provider};
use super::support::*;

#[test]
fn a_silent_client_is_cut_off_at_the_deadline() {
    let limits = Limits { read_deadline: Duration::from_millis(300), ..Limits::default() };
    let api = LocalApi::start_with(0, provider(), limits).expect("start");
    let began = Instant::now();
    let mut stream = connect(&api);
    let reply = read_reply(&mut stream).expect("a 408");
    assert_eq!(reply.status, 408);
    assert!(began.elapsed() < Duration::from_secs(3));
}

#[test]
fn a_trickling_client_cannot_extend_the_deadline() {
    let limits = Limits { read_deadline: Duration::from_millis(400), ..Limits::default() };
    let api = LocalApi::start_with(0, provider(), limits).expect("start");
    let began = Instant::now();
    let mut stream = connect(&api);
    // One byte every 100 ms never completes a head; each read alone would beat a per-read timeout.
    for byte in b"GET /v1/health HTTP/1.1\r\nHost: x".iter() {
        if stream.write_all(&[*byte]).is_err() {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
        if began.elapsed() > Duration::from_secs(2) {
            break;
        }
    }
    let reply = read_reply(&mut stream).expect("a 408");
    assert_eq!(reply.status, 408);
    assert!(began.elapsed() < Duration::from_secs(3), "{:?}", began.elapsed());
}

#[test]
fn concurrent_connections_are_capped_and_slots_come_back() {
    let limits = Limits {
        read_deadline: Duration::from_millis(800),
        max_connections: 2,
        ..Limits::default()
    };
    let api = LocalApi::start_with(0, provider(), limits).expect("start");
    let idle = [connect(&api), connect(&api)];
    std::thread::sleep(Duration::from_millis(200));
    // The third is closed without an answer while two are being waited on.
    let mut third = connect(&api);
    assert!(read_reply(&mut third).is_none());
    drop(idle);
    std::thread::sleep(Duration::from_millis(1200));
    assert_eq!(get(&api, "/v1/health", &own_host(&api)).status, 200);
}

#[test]
fn a_provider_that_panics_costs_one_connection_not_the_server() {
    let calls = Arc::new(AtomicUsize::new(0));
    let seen = Arc::clone(&calls);
    let panicking: Provider = Arc::new(move || {
        assert!(seen.fetch_add(1, Ordering::SeqCst) != 0, "first call fails");
        None
    });
    let api = LocalApi::start_on(0, panicking).expect("start");
    let mut stream = connect(&api);
    stream
        .write_all(
            format!("GET /v1/limits HTTP/1.1\r\nHost: {}\r\n\r\n", own_host(&api)).as_bytes(),
        )
        .expect("write");
    assert!(read_reply(&mut stream).is_none());
    assert_eq!(get(&api, "/v1/limits", &own_host(&api)).status, 503);
    assert_eq!(get(&api, "/v1/health", &own_host(&api)).status, 200);
}
