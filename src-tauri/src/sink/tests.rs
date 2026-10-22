use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU32, Ordering};

use usage_model::settings::{GeneralSettings, Language};
use usage_model::{HarnessView, UsageWindow, ViewState, WindowKind};

use super::*;

mod autostart;
mod language;

const NOW: i64 = 1_800_000_000;

#[derive(Default)]
struct Recorder {
    calls: Mutex<Vec<String>>,
    /// Every notification the sink asked to show, whole.
    toasts: Mutex<Vec<Notification>>,
    size: AtomicU32,
    /// How many upcoming startup-entry updates fail, like a registry that refuses the write.
    autostart_failures: AtomicU32,
    autostart_failed: AtomicBool,
    /// Every language the tray menu was relabelled to.
    languages: Mutex<Vec<Language>>,
}

impl Recorder {
    fn log(&self, line: String) {
        self.calls.lock().expect("lock").push(line);
    }

    fn calls(&self) -> Vec<String> {
        self.calls.lock().expect("lock").clone()
    }

    fn take(&self) -> Vec<String> {
        std::mem::take(&mut *self.calls.lock().expect("lock"))
    }
}

impl Effects for Arc<Recorder> {
    fn emit_snapshot(&self, snapshot: &Snapshot) {
        self.log(format!("snapshot:{}", snapshot.views.len()));
    }

    fn emit_settings(&self, settings: &Settings) {
        self.log(format!("settings:{}", settings.general.interval_secs));
    }

    fn icon_size(&self) -> u32 {
        self.size.load(Ordering::SeqCst)
    }

    fn set_tray_icon(&self, size: u32, gauge: Gauge) {
        self.log(format!("icon:{size}:{:?}:{:?}", gauge.available_pct, gauge.status));
    }

    fn set_tray_tooltip(&self, text: &str) {
        self.log(format!("tooltip:{text}"));
    }

    fn set_language(&self, language: Language) {
        self.languages.lock().expect("lock").push(language);
    }

    fn toast(&self, notification: &Notification) {
        self.log(format!("toast:{}", notification.title));
        self.toasts.lock().expect("lock").push(notification.clone());
    }

    fn sync_autostart(&self, enabled: bool, origin: Origin) {
        self.log(format!("autostart:{enabled}:{origin:?}"));
        let update = |left: u32| left.checked_sub(1);
        let refused =
            self.autostart_failures.fetch_update(Ordering::SeqCst, Ordering::SeqCst, update);
        self.autostart_failed.store(refused.is_ok(), Ordering::SeqCst);
    }

    fn autostart_failed(&self) -> bool {
        self.autostart_failed.load(Ordering::SeqCst)
    }
}

struct Rig {
    sink: ShellSink<Arc<Recorder>>,
    rec: Arc<Recorder>,
    now: Arc<AtomicI64>,
}

fn rig() -> Rig {
    let rec = Arc::new(Recorder::default());
    rec.size.store(16, Ordering::SeqCst);
    let now = Arc::new(AtomicI64::new(NOW));
    let clock_now = now.clone();
    let clock: Clock = Arc::new(move || clock_now.load(Ordering::SeqCst));
    Rig { sink: ShellSink::new(rec.clone(), clock), rec, now }
}

fn view(id: HarnessId, used: f64, resets_in_min: i64) -> HarnessView {
    let mut window = UsageWindow::new("five_hour", WindowKind::FiveHour);
    window.used_pct = Some(used);
    window.resets_at = Some(NOW + resets_in_min * 60);
    let mut view = HarnessView::with_state(id, ViewState::Ready);
    view.windows = vec![window];
    view
}

fn snapshot(views: Vec<HarnessView>) -> Snapshot {
    Snapshot { generated_at: NOW, views }
}

fn settings(tray_source: Option<HarnessId>, autostart: bool) -> Settings {
    Settings {
        general: GeneralSettings {
            tray_source,
            autostart,
            language: Language::PtBr,
            ..GeneralSettings::default()
        },
        ..Settings::default()
    }
}

#[test]
fn a_snapshot_reaches_the_webview_and_drives_the_tray() {
    let rig = rig();
    rig.sink.prime(&settings(None, false));
    rig.rec.take();
    rig.sink.snapshot_changed(&snapshot(vec![view(HarnessId::Claude, 28.0, 113)]));
    assert_eq!(
        rig.rec.take(),
        [
            "snapshot:1".to_owned(),
            "icon:16:Some(72.0):Some(Ok)".to_owned(),
            "tooltip:Claude Code · 5 horas: 72% disponível · reseta em 1h 53m".to_owned(),
        ]
    );
}

#[test]
fn before_any_snapshot_the_tray_reads_sem_dados() {
    let rig = rig();
    rig.sink.prime(&settings(None, false));
    assert_eq!(
        rig.rec.calls(),
        ["autostart:false:Startup", "icon:16:None:None", "tooltip:AI Session Monitor: sem dados"]
    );
}

#[test]
fn an_unchanged_tray_is_not_touched_again() {
    let rig = rig();
    rig.sink.prime(&settings(None, false));
    let snap = snapshot(vec![view(HarnessId::Claude, 28.0, 113)]);
    rig.sink.snapshot_changed(&snap);
    rig.rec.take();
    rig.sink.snapshot_changed(&snap);
    assert_eq!(rig.rec.take(), ["snapshot:1"]);
}

#[test]
fn the_tick_moves_the_countdown_and_reapplies_the_tray() {
    let rig = rig();
    rig.sink.prime(&settings(None, false));
    rig.sink.snapshot_changed(&snapshot(vec![view(HarnessId::Claude, 28.0, 113)]));
    rig.rec.take();
    rig.now.store(NOW + 60, Ordering::SeqCst);
    rig.sink.tick();
    let calls = rig.rec.take();
    assert!(
        calls.contains(
            &"tooltip:Claude Code · 5 horas: 72% disponível · reseta em 1h 52m".to_owned()
        ),
        "{calls:?}"
    );
    assert!(calls.contains(&"icon:16:Some(72.0):Some(Ok)".to_owned()), "{calls:?}");
}

#[test]
fn a_window_that_resets_between_snapshots_stops_colouring_the_tray() {
    let rig = rig();
    rig.sink.prime(&settings(None, false));
    rig.sink.snapshot_changed(&snapshot(vec![view(HarnessId::Claude, 90.0, 10)]));
    rig.rec.take();
    rig.now.store(NOW + 11 * 60, Ordering::SeqCst);
    rig.sink.tick();
    let calls = rig.rec.take();
    assert!(calls.contains(&"icon:16:None:None".to_owned()), "{calls:?}");
    assert!(calls.contains(&"tooltip:AI Session Monitor: sem dados".to_owned()), "{calls:?}");
}

#[test]
fn changing_the_followed_harness_retargets_the_tray_and_tells_the_webview() {
    let rig = rig();
    rig.sink.prime(&settings(None, false));
    rig.sink.snapshot_changed(&snapshot(vec![
        view(HarnessId::Claude, 28.0, 113),
        view(HarnessId::Codex, 90.0, 60),
    ]));
    rig.rec.take();
    rig.sink.settings_changed(&settings(Some(HarnessId::Claude), false));
    let calls = rig.rec.take();
    assert_eq!(calls[0], "settings:60");
    assert!(calls.contains(&"icon:16:Some(72.0):Some(Ok)".to_owned()), "{calls:?}");
    assert!(calls.iter().any(|c| c.starts_with("tooltip:Claude Code")), "{calls:?}");
}

#[test]
fn a_followed_harness_without_data_names_itself() {
    let rig = rig();
    rig.sink.prime(&settings(Some(HarnessId::Codex), false));
    assert!(rig.rec.calls().contains(&"tooltip:Codex: sem dados".to_owned()));
}

#[test]
fn a_new_icon_size_replaces_the_icon_even_when_the_gauge_is_the_same() {
    let rig = rig();
    rig.sink.prime(&settings(None, false));
    rig.sink.snapshot_changed(&snapshot(vec![view(HarnessId::Claude, 28.0, 113)]));
    rig.rec.take();
    rig.rec.size.store(24, Ordering::SeqCst);
    rig.sink.snapshot_changed(&snapshot(vec![view(HarnessId::Claude, 28.0, 113)]));
    assert_eq!(rig.rec.take(), ["snapshot:1", "icon:24:Some(72.0):Some(Ok)"]);
}

#[test]
fn autostart_is_synced_once_per_change() {
    let rig = rig();
    rig.sink.prime(&settings(None, true));
    rig.sink.settings_changed(&settings(None, true));
    rig.sink.settings_changed(&settings(None, false));
    rig.sink.settings_changed(&settings(None, false));
    let syncs: Vec<String> =
        rig.rec.calls().into_iter().filter(|c| c.starts_with("autostart:")).collect();
    // The start adopts the stored setting; only a later change is the user's own.
    assert_eq!(syncs, ["autostart:true:Startup", "autostart:false:UserChange"]);
}

#[test]
fn priming_does_not_emit_a_settings_event() {
    let rig = rig();
    rig.sink.prime(&settings(None, false));
    assert!(!rig.rec.calls().iter().any(|c| c.starts_with("settings:")));
}

#[test]
fn a_notification_becomes_a_toast() {
    let rig = rig();
    let note = Notification {
        kind: NotificationKind::WindowOut,
        harness: HarnessId::Claude,
        window_id: "five_hour".to_owned(),
        title: "Claude Code esgotou".to_owned(),
        body: "A janela de 5 horas acabou.".to_owned(),
    };
    rig.sink.notify(&note);
    assert_eq!(rig.rec.calls(), ["toast:Claude Code esgotou"]);
}

#[test]
fn the_settings_recovered_notice_is_shown_whole_and_touches_nothing_else() {
    let rig = rig();
    // app-core fills `harness` and `window_id` with placeholders for this kind; the sink must
    // not drop a notice because it names no real window.
    let note = Notification {
        kind: NotificationKind::SettingsRecovered,
        harness: HarnessId::Claude,
        window_id: String::new(),
        title: "Configurações restauradas".to_owned(),
        body: "O arquivo de configurações estava ilegível ou inválido. Uma cópia foi guardada \
               ao lado dele (settings.corrupt-1800000000.json) e os valores padrão foram \
               aplicados."
            .to_owned(),
    };
    rig.sink.notify(&note);
    assert_eq!(*rig.rec.toasts.lock().expect("lock"), [note]);
    assert_eq!(rig.rec.calls(), ["toast:Configurações restauradas"]);
}

#[test]
fn the_system_clock_reads_a_plausible_epoch() {
    // 2026-01-01: a clock stuck at zero or in the wrong unit would fail this.
    assert!(system_clock()() > 1_767_225_600);
}
