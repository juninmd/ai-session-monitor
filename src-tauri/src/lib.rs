//! Tray app shell: the Tauri window, tray, plugins and IPC around `app-core`. It holds no business
//! rules; `setup` is the composition root and the other modules are adapters or pure helpers.

use std::sync::Arc;
use std::time::Duration;

use app_core::AppService;
use tauri::utils::config::AppDirectoriesOverride;
use tauri::{AppHandle, Manager, RunEvent};

mod autostart;
mod claude_dir;
mod commands;
mod diag;
mod effects;
mod events;
mod flyout;
mod gate;
mod icon;
mod links;
mod paths;
mod position;
mod secrets_keyring;
mod setup;
mod shim_path;
mod sink;
mod startup;
mod summary;
mod supervisor;
mod tooltip;
mod tray;
mod wiring;
#[cfg(feature = "collectors")]
mod wiring_collectors;

pub fn run() {
    let overrides = paths::Overrides::from_env()
        .unwrap_or_else(|err| diag::fatal("invalid test override", &err));
    let mut context = tauri::generate_context!();
    // Debug runs with a test profile keep the webview data in it too, never in the real one.
    if let Some(root) = overrides.data_root() {
        context.config_mut().app.app_directories_override =
            Some(AppDirectoriesOverride::Root(root));
    }
    let built = tauri::Builder::default()
        // First, so a second launch hands over and exits before anything else starts.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| setup::reopen(app)))
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_snapshot,
            commands::refresh,
            commands::get_settings,
            commands::save_settings,
            commands::secret_status,
            commands::set_secret,
            commands::clear_secret,
            commands::claude_shim_status,
            commands::install_claude_shim,
            commands::uninstall_claude_shim,
            commands::open_link,
            commands::hide_flyout,
            commands::set_flyout_height,
            commands::quit_app,
        ])
        .on_page_load(setup::on_page_load)
        .setup(move |app| {
            // Returning the error would make Tauri panic, which nobody sees without a console.
            if let Err(reason) = setup::run(app, &overrides) {
                startup::abort(app.handle(), reason);
            }
            Ok(())
        })
        .build(context);
    match built {
        Ok(app) => app.run(on_run_event),
        Err(err) => diag::fatal("cannot start AI Session Monitor", &err),
    }
}

/// Above the scheduler's own 2 s stop grace.
const EXIT_GRACE: Duration = Duration::from_millis(2500);

fn on_run_event(app: &AppHandle, event: RunEvent) {
    match event {
        // The tray keeps the app alive when the last window goes away; only a programmatic
        // exit (the Sair menu entry, quit_app) carries a code.
        RunEvent::ExitRequested { code: None, api, .. } => api.prevent_exit(),
        RunEvent::Exit => {
            if let Some(service) = app.try_state::<Arc<AppService>>() {
                // The process ends right after this event; give the scheduler time to kill the
                // Codex child tree, but never let a stuck loop hang quit.
                let wait = service.shutdown_and_wait(EXIT_GRACE);
                let _unwound = tauri::async_runtime::block_on(wait);
            }
        }
        _ => {}
    }
}
