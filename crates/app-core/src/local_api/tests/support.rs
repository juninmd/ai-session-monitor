//! Shared rig for the loopback tests.

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::time::Duration;

use usage_model::{HarnessId, HarnessView, Snapshot, UsageWindow, ViewState, WindowKind};

use super::super::{LocalApi, Provider, Reading};

pub(super) const NOW: i64 = 1_791_633_600;

pub(super) struct Reply {
    pub status: u16,
    pub headers: BTreeMap<String, String>,
    pub body: String,
}

pub(super) fn provider() -> Provider {
    Arc::new(|| {
        let mut window = UsageWindow::new("five_hour", WindowKind::FiveHour);
        window.used_pct = Some(30.0);
        window.resets_at = Some(NOW + 3600);
        let mut view = HarnessView::with_state(HarnessId::Claude, ViewState::Ready);
        view.windows = vec![window];
        Some(Reading {
            snapshot: Snapshot { generated_at: NOW, views: vec![view] },
            warn_at_pct: 25,
        })
    })
}

pub(super) fn start() -> LocalApi {
    LocalApi::start_on(0, provider()).expect("start")
}

pub(super) fn connect(api: &LocalApi) -> TcpStream {
    let stream = TcpStream::connect(api.addr()).expect("connect");
    stream.set_read_timeout(Some(Duration::from_secs(5))).expect("timeout");
    stream
}

pub(super) fn read_reply(stream: &mut TcpStream) -> Option<Reply> {
    let mut raw = Vec::new();
    let _ = stream.read_to_end(&mut raw);
    let text = String::from_utf8(raw).expect("utf-8");
    let (head, body) = text.split_once("\r\n\r\n")?;
    let mut lines = head.split("\r\n");
    let status = lines.next()?.split(' ').nth(1)?.parse().ok()?;
    let headers = lines
        .filter_map(|line| line.split_once(": "))
        .map(|(name, value)| (name.to_ascii_lowercase(), value.to_owned()))
        .collect();
    Some(Reply { status, headers, body: body.to_owned() })
}

pub(super) fn send(api: &LocalApi, raw: &str) -> Reply {
    let mut stream = connect(api);
    stream.write_all(raw.as_bytes()).expect("write");
    read_reply(&mut stream).expect("a response")
}

pub(super) fn get(api: &LocalApi, path: &str, host: &str) -> Reply {
    send(api, &format!("GET {path} HTTP/1.1\r\nHost: {host}\r\n\r\n"))
}

pub(super) fn own_host(api: &LocalApi) -> String {
    format!("127.0.0.1:{}", api.port())
}
