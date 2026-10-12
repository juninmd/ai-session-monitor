//! An `EventSink` that records everything it is told, with the virtual instant of each event.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

use app_core::{EventSink, Notification};
use serde::Serialize;
use tokio::time::Instant;
use usage_model::{Settings, Snapshot};

use crate::collector::lock;

pub enum Event {
    Snapshot(Box<Snapshot>),
    Settings(Box<Settings>),
    Notice(Box<Notification>),
}

/// Longest a parked call waits to be released, so a broken test fails instead of hanging.
const HOLD_LIMIT: Duration = Duration::from_secs(5);

/// Parks the first `settings_changed` call inside the sink until the test lets it go.
struct Gate {
    entered: mpsc::Sender<()>,
    release: Mutex<mpsc::Receiver<()>>,
    used: AtomicBool,
}

impl Gate {
    fn hold_first_call(&self) {
        if self.used.swap(true, Ordering::SeqCst) {
            return;
        }
        // Either side being gone only means the test is over; the assertions report the rest.
        let _ = self.entered.send(());
        let _ = lock(&self.release).recv_timeout(HOLD_LIMIT);
    }
}

/// The test's end of a `Gate`.
pub struct GateHandle {
    entered: mpsc::Receiver<()>,
    release: mpsc::Sender<()>,
}

impl GateHandle {
    /// Blocks until a call is parked inside the sink.
    pub fn wait_until_parked(&self) {
        let parked = self.entered.recv_timeout(HOLD_LIMIT);
        assert!(parked.is_ok(), "no delivery reached the sink");
    }

    pub fn release(&self) {
        let _ = self.release.send(());
    }
}

#[derive(Default)]
pub struct RecordingSink {
    events: Mutex<Vec<(Instant, Event)>>,
    /// A real pause inside `settings_changed`, to widen the window for overlapping deliveries.
    settings_pause: Duration,
    gate: Option<Gate>,
    inside: AtomicUsize,
    max_inside: AtomicUsize,
    /// How many of the next `snapshot_changed` / `notify` calls panic before recording anything.
    panic_snapshots: AtomicUsize,
    panic_notices: AtomicUsize,
}

/// True once per unit left in `armed`, counting it down.
fn fires(armed: &AtomicUsize) -> bool {
    armed.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |left| left.checked_sub(1)).is_ok()
}

impl RecordingSink {
    pub fn new() -> Arc<Self> {
        Arc::default()
    }

    pub fn pausing_in_settings(pause: Duration) -> Arc<Self> {
        Arc::new(Self { settings_pause: pause, ..Self::default() })
    }

    /// A sink with a bug: its first `count` snapshot deliveries panic, in the scheduler task.
    pub fn panicking_in_snapshots(count: usize) -> Arc<Self> {
        Arc::new(Self { panic_snapshots: AtomicUsize::new(count), ..Self::default() })
    }

    /// Same for the first `count` notifications.
    pub fn panicking_in_notices(count: usize) -> Arc<Self> {
        Arc::new(Self { panic_notices: AtomicUsize::new(count), ..Self::default() })
    }

    /// The first `settings_changed` call stays inside the sink until the handle releases it.
    pub fn gated_first_settings() -> (Arc<Self>, GateHandle) {
        let (entered, entered_rx) = mpsc::channel();
        let (release_tx, release) = mpsc::channel();
        let gate = Gate { entered, release: Mutex::new(release), used: AtomicBool::new(false) };
        let sink = Arc::new(Self { gate: Some(gate), ..Self::default() });
        (sink, GateHandle { entered: entered_rx, release: release_tx })
    }

    fn push(&self, event: Event) {
        lock(&self.events).push((Instant::now(), event));
    }

    pub fn snapshots(&self) -> Vec<(Instant, Snapshot)> {
        let events = lock(&self.events);
        let pick = |(at, event): &(Instant, Event)| match event {
            Event::Snapshot(snapshot) => Some((*at, snapshot.as_ref().clone())),
            _ => None,
        };
        events.iter().filter_map(pick).collect()
    }

    pub fn last_snapshot(&self) -> Snapshot {
        let last = self.snapshots().pop();
        last.map(|(_, snapshot)| snapshot).unwrap_or_else(|| panic!("no snapshot was emitted"))
    }

    pub fn settings(&self) -> Vec<Settings> {
        let events = lock(&self.events);
        let pick = |(_, event): &(Instant, Event)| match event {
            Event::Settings(settings) => Some(settings.as_ref().clone()),
            _ => None,
        };
        events.iter().filter_map(pick).collect()
    }

    pub fn notices(&self) -> Vec<Notification> {
        let events = lock(&self.events);
        let pick = |(_, event): &(Instant, Event)| match event {
            Event::Notice(notice) => Some(notice.as_ref().clone()),
            _ => None,
        };
        events.iter().filter_map(pick).collect()
    }

    /// The order of events by kind.
    pub fn kinds(&self) -> Vec<&'static str> {
        let kind = |(_, event): &(Instant, Event)| match event {
            Event::Snapshot(_) => "snapshot",
            Event::Settings(_) => "settings",
            Event::Notice(_) => "notice",
        };
        lock(&self.events).iter().map(kind).collect()
    }

    /// The most `settings_changed` calls that ever ran at the same time.
    pub fn max_overlap(&self) -> usize {
        self.max_inside.load(Ordering::SeqCst)
    }

    /// Everything emitted, in the forms a log line or the webview would carry (Debug and JSON).
    pub fn dump(&self) -> String {
        let render = |(_, event): &(Instant, Event)| match event {
            Event::Snapshot(snapshot) => format!("{snapshot:?} {}", json(snapshot.as_ref())),
            Event::Settings(settings) => format!("{settings:?} {}", json(settings.as_ref())),
            Event::Notice(notice) => format!("{notice:?}"),
        };
        lock(&self.events).iter().map(render).collect::<Vec<_>>().join("\n")
    }
}

fn json<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_default()
}

impl EventSink for RecordingSink {
    fn snapshot_changed(&self, snapshot: &Snapshot) {
        assert!(!fires(&self.panic_snapshots), "the sink was told to fail on a snapshot");
        self.push(Event::Snapshot(Box::new(snapshot.clone())));
    }

    fn settings_changed(&self, settings: &Settings) {
        let now_inside = self.inside.fetch_add(1, Ordering::SeqCst) + 1;
        self.max_inside.fetch_max(now_inside, Ordering::SeqCst);
        if let Some(gate) = &self.gate {
            gate.hold_first_call();
        }
        if !self.settings_pause.is_zero() {
            std::thread::sleep(self.settings_pause);
        }
        self.push(Event::Settings(Box::new(settings.clone())));
        self.inside.fetch_sub(1, Ordering::SeqCst);
    }

    fn notify(&self, notification: &Notification) {
        assert!(!fires(&self.panic_notices), "the sink was told to fail on a notification");
        self.push(Event::Notice(Box::new(notification.clone())));
    }
}
