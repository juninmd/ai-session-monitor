//! Updater composition, public progress events and background checks.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use app_core::AppService;
use app_core::update::UpdateManager;
use tauri::{AppHandle, Emitter, Manager};
use usage_model::{HarnessError, UpdateStatus};

mod installer;

/// Set once the installer is about to start: the instance that remains until it exits must not
/// react to the relaunched app by opening its window.
static INSTALLING: AtomicBool = AtomicBool::new(false);

pub fn installing() -> bool {
    INSTALLING.load(Ordering::SeqCst)
}

pub struct Updater {
    pub manager: Arc<UpdateManager>,
    staging_root: PathBuf,
}

pub fn attach(app: &AppHandle, service: Arc<AppService>, data_dir: PathBuf, enabled: bool) {
    let config: serde_json::Value =
        serde_json::from_str(include_str!("../tauri.conf.json")).unwrap_or_default();
    let key = config["plugins"]["updater"]["pubkey"].as_str().unwrap_or_default().to_owned();
    let emitter = app.clone();
    let manager = UpdateManager::new(
        &app.package_info().version.to_string(),
        key,
        cfg!(all(windows, target_arch = "x86_64", not(debug_assertions))),
        enabled,
        Arc::new(move |status| {
            let _emitted = emitter.emit(crate::events::UPDATE_STATUS, status);
        }),
    );
    app.manage(Updater { manager: Arc::clone(&manager), staging_root: data_dir.join("updates") });
    if enabled {
        spawn_checks(manager, service);
    }
}

fn spawn_checks(manager: Arc<UpdateManager>, service: Arc<AppService>) {
    tauri::async_runtime::spawn(async move {
        let mut timer = tokio::time::interval(Duration::from_secs(60));
        timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            timer.tick().await;
            if !service.settings().general.auto_update_check {
                continue;
            }
            let now =
                SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |t| t.as_secs() as i64);
            if manager.status().checked_at.is_none_or(|last| now.saturating_sub(last) >= 6 * 3600) {
                manager.check().await;
            }
        }
    });
}

pub async fn install(
    app: AppHandle,
    updater: &Updater,
    service: Arc<AppService>,
) -> Result<UpdateStatus, HarnessError> {
    let root = updater.staging_root.clone();
    updater.manager.install(move |bytes| installer::install(app, service, root, bytes)).await
}
