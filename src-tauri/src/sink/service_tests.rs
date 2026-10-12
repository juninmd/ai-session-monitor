//! The sink behind a real `AppService`: what the user hears when the settings file is unusable.
//! `AppService` owns the recovery, so these tests pin that its notice survives the trip through
//! the shell's sink and that the shell adds none of its own.

use std::path::Path;
use std::time::Duration;

use app_core::{Adapters, AppPaths, AppService, ClaudeShim, MemorySecretStore, NotificationKind};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tokio::time::timeout;
use usage_model::settings::ClaudeSettings;
use usage_model::{Ctx, ErrorCode, HarnessError, ShimState, ShimStatus};

use super::*;

enum Heard {
    Toast(Notification),
    Snapshot,
}

/// Effects that report toasts and snapshots over a channel, so a test waits for them in order.
struct Ears(UnboundedSender<Heard>);

impl Effects for Ears {
    fn emit_snapshot(&self, _: &Snapshot) {
        let _ = self.0.send(Heard::Snapshot);
    }

    fn emit_settings(&self, _: &Settings) {}

    fn icon_size(&self) -> u32 {
        16
    }

    fn set_tray_icon(&self, _: u32, _: Gauge) {}

    fn set_tray_tooltip(&self, _: &str) {}

    fn toast(&self, notification: &Notification) {
        let _ = self.0.send(Heard::Toast(notification.clone()));
    }

    fn sync_autostart(&self, _: bool, _: Origin) {}

    fn autostart_failed(&self) -> bool {
        false
    }
}

struct NoShim;

impl ClaudeShim for NoShim {
    fn status(&self, _: &ClaudeSettings, _: &Ctx) -> ShimStatus {
        ShimStatus { state: ShimState::NotInstalled, chained: false, settings_path: None }
    }

    fn install(&self, _: &ClaudeSettings, _: &Ctx, _: &Path) -> Result<ShimStatus, HarnessError> {
        Err(HarnessError::new(ErrorCode::InstallFailed, false))
    }

    fn uninstall(&self, _: &ClaudeSettings, _: &Ctx) -> Result<ShimStatus, HarnessError> {
        Err(HarnessError::new(ErrorCode::InstallFailed, false))
    }
}

struct Running {
    dir: tempfile::TempDir,
    service: Arc<AppService>,
    polling: tokio::task::JoinHandle<()>,
    heard: UnboundedReceiver<Heard>,
}

/// A real service over a temp profile whose settings file holds `settings`, already polling.
fn start(settings: &[u8]) -> Running {
    let dir = tempfile::tempdir().expect("tempdir");
    let paths = AppPaths { data_dir: dir.path().to_path_buf(), home_dir: dir.path().join("home") };
    std::fs::write(paths.settings_file(), settings).expect("write settings");
    let (tx, heard) = unbounded_channel();
    let sink = Arc::new(ShellSink::new(Ears(tx), system_clock()));
    let adapters = Adapters { collectors: Vec::new(), claude_shim: Arc::new(NoShim) };
    let secrets = Arc::new(MemorySecretStore::new());
    let service = AppService::new(paths, secrets, sink, adapters).expect("service starts");
    let polling = tokio::spawn(Arc::clone(&service).run());
    Running { dir, service, polling, heard }
}

async fn next(heard: &mut UnboundedReceiver<Heard>) -> Heard {
    let wait = timeout(Duration::from_secs(10), heard.recv());
    wait.await.expect("something within ten seconds").expect("the sink is still alive")
}

#[tokio::test]
async fn an_unreadable_settings_file_reaches_the_user_as_one_toast_before_any_snapshot() {
    let mut run = start(b"{ not json");

    let Heard::Toast(toast) = next(&mut run.heard).await else {
        panic!("the first thing the user hears is the recovery notice");
    };
    assert_eq!(toast.kind, NotificationKind::SettingsRecovered);
    assert_eq!(toast.title, "Configurações restauradas");
    assert!(toast.body.contains("settings.corrupt-"), "{}", toast.body);
    assert!(
        !toast.body.contains(&*run.dir.path().to_string_lossy()),
        "the toast names the copy, never a path: {}",
        toast.body
    );

    assert!(matches!(next(&mut run.heard).await, Heard::Snapshot));
    run.service.shutdown();
    run.polling.await.expect("polling ends cleanly");
    while let Ok(heard) = run.heard.try_recv() {
        assert!(matches!(heard, Heard::Snapshot), "a second toast for the same recovery");
    }

    let kept: Vec<_> = std::fs::read_dir(run.dir.path())
        .expect("read_dir")
        .map(|entry| entry.expect("entry").path())
        .filter(|path| path.to_string_lossy().contains("settings.corrupt-"))
        .collect();
    assert_eq!(kept.len(), 1);
    assert_eq!(std::fs::read(&kept[0]).expect("read"), b"{ not json", "the original is kept");
}

#[tokio::test]
async fn a_readable_settings_file_stays_silent() {
    let mut run = start(b"{}");

    // The first snapshot is published after the loop would have announced a recovery.
    assert!(matches!(next(&mut run.heard).await, Heard::Snapshot));
    run.service.shutdown();
    run.polling.await.expect("polling ends cleanly");
    while let Ok(heard) = run.heard.try_recv() {
        assert!(matches!(heard, Heard::Snapshot), "a toast with nothing to report");
    }
}
