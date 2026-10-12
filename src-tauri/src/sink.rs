//! The `EventSink` the service reports to. It keeps the latest snapshot and settings, tells the
//! webview, and keeps the tray icon, its tooltip, the toasts and the autostart entry in step.
//! Every outward action goes through `Effects`, so the logic runs without a window.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{SystemTime, UNIX_EPOCH};

use app_core::{EventSink, Notification, NotificationKind};
use usage_model::status::Status;
use usage_model::{HarnessId, Settings, Snapshot};

use crate::autostart::Origin;
use crate::icon::Gauge;
use crate::{summary, tooltip};

#[cfg(test)]
mod service_tests;
#[cfg(test)]
mod tests;

/// A failed update of the startup entry is tried again on the tick this many times per streak.
const AUTOSTART_RETRIES: u8 = 3;

pub trait Effects: Send + Sync + 'static {
    fn emit_snapshot(&self, snapshot: &Snapshot);
    fn emit_settings(&self, settings: &Settings);
    /// Pixel size of the tray icon for the current display scale.
    fn icon_size(&self) -> u32;
    fn set_tray_icon(&self, size: u32, gauge: Gauge);
    fn set_tray_tooltip(&self, text: &str);
    fn toast(&self, notification: &Notification);
    /// Must not block: the caller may be the scheduler.
    fn sync_autostart(&self, enabled: bool, origin: Origin);
    /// Whether the last startup-entry update failed.
    fn autostart_failed(&self) -> bool;
}

/// Epoch seconds "now", injected so the tray countdown is testable.
pub type Clock = Arc<dyn Fn() -> i64 + Send + Sync>;

/// The wall clock, in epoch seconds.
pub fn system_clock() -> Clock {
    Arc::new(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| i64::try_from(elapsed.as_secs()).unwrap_or(i64::MAX))
    })
}

#[derive(Default)]
struct Latest {
    snapshot: Option<Snapshot>,
    settings: Settings,
}

/// What the tray and the OS already show, to skip redundant updates.
#[derive(Default)]
struct Applied {
    icon: Option<(u32, Option<u8>, Option<Status>)>,
    tooltip: Option<String>,
    autostart: AutostartSync,
}

/// The startup-entry update last handed to the OS and how its failures were handled.
#[derive(Default)]
struct AutostartSync {
    sent: Option<(bool, Origin)>,
    /// Retries spent on the current failure streak.
    retries: u8,
    /// The user was told about the current failure streak.
    told: bool,
}

/// What a tick owes a startup entry whose last update failed.
struct Resync {
    wanted: bool,
    origin: Origin,
    tell: bool,
    again: bool,
}

impl AutostartSync {
    fn on_tick(&mut self, failed: bool) -> Option<Resync> {
        let (wanted, origin) = self.sent.filter(|_| failed)?;
        let tell = !std::mem::replace(&mut self.told, true);
        let again = self.retries < AUTOSTART_RETRIES;
        self.retries += u8::from(again);
        Some(Resync { wanted, origin, tell, again })
    }
}

/// The toast for a failed startup-entry update. app-core has no kind for a failure of the shell
/// itself, and the toast shows only the title and the body.
fn autostart_failure(wanted: bool) -> Notification {
    let verb = if wanted { "ligar" } else { "desligar" };
    Notification {
        kind: NotificationKind::SettingsRecovered,
        harness: HarnessId::Claude,
        window_id: String::new(),
        title: "AI Session Monitor".to_owned(),
        body: format!("Não foi possível {verb} 'Iniciar com o Windows'."),
    }
}

pub struct ShellSink<E: Effects> {
    effects: E,
    clock: Clock,
    latest: Mutex<Latest>,
    applied: Mutex<Applied>,
}

/// State is plain data, so a panic elsewhere must not take the tray down with it.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

impl<E: Effects> ShellSink<E> {
    pub fn new(effects: E, clock: Clock) -> Self {
        Self {
            effects,
            clock,
            latest: Mutex::new(Latest::default()),
            applied: Mutex::new(Applied::default()),
        }
    }

    /// Startup: adopt the loaded settings without telling the webview, which fetches its own.
    pub fn prime(&self, settings: &Settings) {
        self.adopt(settings, Origin::Startup);
    }

    /// Re-renders the tray: the countdown moves and windows expire without any new snapshot. It
    /// always re-applies, which also repairs a tray left stale by two updates that raced.
    pub fn tick(&self) {
        self.retry_autostart();
        self.render(true);
    }

    fn adopt(&self, settings: &Settings, origin: Origin) {
        lock(&self.latest).settings = settings.clone();
        let wanted = settings.general.autostart;
        let changed = {
            let mut applied = lock(&self.applied);
            let changed = applied.autostart.sent.map(|(value, _)| value) != Some(wanted);
            if changed {
                let sent = Some((wanted, origin));
                applied.autostart = AutostartSync { sent, ..AutostartSync::default() };
            }
            changed
        };
        if changed {
            self.effects.sync_autostart(wanted, origin);
        }
        self.render(false);
    }

    /// A failed update is told to the user once and tried again on the next ticks, a few times,
    /// as the same kind of request: a retry must not turn a start-up pass into a user's change.
    fn retry_autostart(&self) {
        let failed = self.effects.autostart_failed();
        let Some(resync) = lock(&self.applied).autostart.on_tick(failed) else { return };
        if resync.tell {
            self.effects.toast(&autostart_failure(resync.wanted));
        }
        if resync.again {
            self.effects.sync_autostart(resync.wanted, resync.origin);
        }
    }

    fn render(&self, force: bool) {
        let now = (self.clock)();
        let (pct, status, text) = {
            let latest = lock(&self.latest);
            let general = &latest.settings.general;
            let reading =
                latest.snapshot.as_ref().and_then(|snapshot| summary::pick(snapshot, general, now));
            let text = tooltip::text(reading.as_ref(), general.tray_source, general, now);
            let pct = reading.as_ref().map(|r| r.available_pct.round().clamp(0.0, 100.0) as u8);
            (pct, reading.map(|r| r.status), text)
        };
        let size = self.effects.icon_size();
        let (icon_changed, tooltip_changed) = {
            let mut applied = lock(&self.applied);
            let icon = Some((size, pct, status));
            let tooltip = Some(text.clone());
            let changed = (force || applied.icon != icon, force || applied.tooltip != tooltip);
            applied.icon = icon;
            applied.tooltip = tooltip;
            changed
        };
        if icon_changed {
            self.effects.set_tray_icon(size, Gauge { available_pct: pct.map(f64::from), status });
        }
        if tooltip_changed {
            self.effects.set_tray_tooltip(&text);
        }
    }
}

impl<E: Effects> EventSink for ShellSink<E> {
    fn snapshot_changed(&self, snapshot: &Snapshot) {
        lock(&self.latest).snapshot = Some(snapshot.clone());
        self.effects.emit_snapshot(snapshot);
        self.render(false);
    }

    fn settings_changed(&self, settings: &Settings) {
        self.effects.emit_settings(settings);
        self.adopt(settings, Origin::UserChange);
    }

    fn notify(&self, notification: &Notification) {
        self.effects.toast(notification);
    }
}
