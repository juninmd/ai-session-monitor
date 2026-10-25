//! Composition root: builds every adapter and hands them to the service. Nothing here decides
//! behaviour; it only decides which implementation sits behind which port.

use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use app_core::{AppPaths, AppService, MemorySecretStore, SecretStore};
use tauri::{App, AppHandle, Manager};
use tokio::time::{MissedTickBehavior, interval};
use usage_model::HarnessError;

use crate::autostart::{Applier, RunKey};
use crate::effects::TauriEffects;
use crate::flyout::{self, Flyout};
use crate::paths::Overrides;
use crate::secrets_keyring::{KeyringSecretStore, UnavailableSecretStore};
use crate::sink::{self, ShellSink};
use crate::startup::Reason;
use crate::{claude_dir, diag, shim_path, supervisor, tray, update, wiring};

const APP_NAME: &str = "AI Session Monitor";
/// The tray is re-rendered this often: countdowns move and windows expire between snapshots.
const TICK: Duration = Duration::from_secs(30);

/// A failure here ends the app through `startup::abort`; the cause is already on stderr.
pub fn run(app: &mut App, overrides: &Overrides) -> Result<(), Reason> {
    let handle = app.handle().clone();
    let paths = resolve_paths(&handle, overrides).map_err(|err| Reason::Paths.because(&err))?;
    std::fs::create_dir_all(&paths.data_dir).map_err(|err| Reason::DataDir.because(&err))?;

    let effects = TauriEffects::new(&handle, autostart(overrides));
    let sink = Arc::new(ShellSink::new(effects, sink::system_clock()));

    // The service sets an unreadable settings file aside and announces it through the sink.
    let adapters = wiring::adapters(claude_dir::from_env(overrides));
    let update_dir = paths.data_dir.clone();
    let service = AppService::new(paths, secret_store(overrides), sink.clone(), adapters)
        .map_err(|err| Reason::Service(err.code).because(&"the service refused to start"))?;
    let settings = service.settings();

    // Manage the service and updater before building the webview: WebviewWindowBuilder::build()
    // pumps the Win32 message loop, so the page's initial IPC commands can arrive before setup
    // returns and must find every managed state already in place.
    app.manage(Arc::clone(&service));
    crate::update::attach(&handle, Arc::clone(&service), update_dir, !overrides.is_active());

    let window = flyout::create(&handle).map_err(|err| Reason::Window.because(&err))?;
    let flyout = Arc::new(Flyout::new(&handle, window));
    flyout.attach();
    app.manage(Arc::clone(&flyout));
    flyout.place_hidden();
    tray::build(&handle, Arc::clone(&flyout)).map_err(|err| Reason::Tray.because(&err))?;
    sink.prime(&settings);

    if !update::installing() && (!settings.general.first_run_done || overrides.show_on_start) {
        tray::open_flyout(&handle, &flyout);
        record_first_run(&handle);
    }

    supervise_polling(Arc::clone(&service));
    refresh_shim_copy(service);
    spawn_ticker(sink);
    Ok(())
}

/// Called on a second launch: show the running instance instead of starting another.
pub fn reopen(app: &AppHandle) {
    // The relaunch after an update must leave the app in the tray, not open a window.
    if update::installing() {
        return;
    }
    if let Some(flyout) = app.try_state::<Arc<Flyout>>() {
        tray::open_flyout(app, &flyout);
    }
}

/// The flag goes through `save_settings`, like any other change of settings.
fn record_first_run(app: &AppHandle) {
    let Some(service) = app.try_state::<Arc<AppService>>() else { return };
    let service = Arc::clone(service.inner());
    tauri::async_runtime::spawn_blocking(move || {
        let mut settings = service.settings();
        if settings.general.first_run_done {
            return;
        }
        settings.general.first_run_done = true;
        if let Err(err) = service.save_settings(settings) {
            diag::warn("cannot record the first run", &format!("{:?}", err.code));
        }
    });
}

/// An app update ships a fixed shim, but only an install writes the copy Claude Code runs, so each
/// launch brings the copy up to the bundled build. Off the startup path: a copy Claude Code is
/// running right now stays as it is until the next launch.
fn refresh_shim_copy(service: Arc<AppService>) {
    tauri::async_runtime::spawn_blocking(move || {
        // A build without a real sidecar (a dev run) has nothing to refresh from.
        let Ok(shim) = shim_path::bundled_shim_exe() else { return };
        if let Err(err) = service.refresh_claude_shim_copy(&shim) {
            diag::warn("cannot refresh the Claude shim copy", &failure(&err));
        }
    });
}

/// The error code and its machine-readable reason, never a path or OS text.
fn failure(err: &HarnessError) -> String {
    let reason = err.params.get("reason").map_or("-", String::as_str);
    format!("{:?} ({reason})", err.code)
}

fn resolve_paths(app: &AppHandle, overrides: &Overrides) -> Result<AppPaths, Box<dyn Error>> {
    let resolver = app.path();
    Ok(overrides.resolve(resolver.app_local_data_dir()?, resolver.home_dir()?))
}

fn secret_store(overrides: &Overrides) -> Arc<dyn SecretStore> {
    if overrides.uses_memory_secrets() {
        return Arc::new(MemorySecretStore::new());
    }
    match KeyringSecretStore::open() {
        Ok(store) => Arc::new(store),
        Err(_) => {
            diag::warn("the Credential Manager is unavailable", &"secrets are refused");
            Arc::new(UnavailableSecretStore)
        }
    }
}

/// `None` under a test override: a smoke run must not leave a Run entry behind.
fn autostart(overrides: &Overrides) -> Option<Arc<Applier>> {
    if overrides.is_active() {
        return None;
    }
    match RunKey::for_current_exe(APP_NAME) {
        Ok(key) => Some(Arc::new(Applier::new(Arc::new(key)))),
        Err(err) => {
            diag::warn("cannot prepare the Windows startup entry", &err);
            None
        }
    }
}

/// The polling loop runs in a task of its own so that a panic in it ends that task and not the
/// app; `supervisor` starts it again.
pub(crate) fn supervise_polling(service: Arc<AppService>) {
    let spawn = move || tauri::async_runtime::handle().inner().spawn(Arc::clone(&service).run());
    tauri::async_runtime::spawn(supervisor::supervise(spawn));
}

fn spawn_ticker<E: crate::sink::Effects>(sink: Arc<ShellSink<E>>) {
    tauri::async_runtime::spawn(async move {
        let mut every = interval(TICK);
        every.set_missed_tick_behavior(MissedTickBehavior::Delay);
        // The first tick completes at once and the sink was just primed.
        every.tick().await;
        loop {
            every.tick().await;
            let sink = Arc::clone(&sink);
            if tauri::async_runtime::spawn_blocking(move || sink.tick()).await.is_err() {
                diag::warn("the tray refresh stopped", &"worker failed");
            }
        }
    });
}
