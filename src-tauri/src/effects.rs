//! The Tauri side of `Effects`: events to the webview, the tray icon and its tooltip, toasts and
//! the Windows startup entry. Plumbing only; every decision lives in `sink`.

use std::sync::{Arc, Mutex, PoisonError};

use app_core::local_api::{LocalApi, Provider, Reading};
use app_core::{AppService, Notification};
use tauri::image::Image;
use tauri::tray::TrayIcon;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;
use usage_model::settings::Language;
use usage_model::{Settings, Snapshot};

use crate::autostart::{Applier, Origin};
use crate::icon::{self, Gauge};
use crate::sink::Effects;
use crate::{diag, events, flyout, tray};

pub struct TauriEffects {
    app: AppHandle,
    /// `None` under a test override, which must never touch the Run key.
    autostart: Option<Arc<Applier>>,
    local_api: Mutex<ApiSlot>,
}

/// The loopback API while it runs, and whether the last attempt to start it failed.
#[derive(Default)]
struct ApiSlot {
    running: Option<LocalApi>,
    failed: bool,
}

impl TauriEffects {
    pub fn new(app: &AppHandle, autostart: Option<Arc<Applier>>) -> Self {
        Self { app: app.clone(), autostart, local_api: Mutex::new(ApiSlot::default()) }
    }

    /// Looked up per call and never kept: Tauri removes the icon when the last handle drops, so a
    /// handle held until `exit` would leave a ghost icon behind in the notification area.
    fn tray(&self) -> Option<TrayIcon> {
        self.app.tray_by_id(tray::TRAY_ID)
    }

    /// What a request is answered from. The service is managed after the effects exist, so it is
    /// looked up per request: before that the API answers 503.
    fn api_provider(&self) -> Provider {
        let app = self.app.clone();
        Arc::new(move || {
            let service = app.try_state::<Arc<AppService>>()?;
            let warn_at_pct = service.settings().general.warn_at_pct;
            Some(Reading { snapshot: service.snapshot(), warn_at_pct })
        })
    }
}

impl Effects for TauriEffects {
    fn emit_snapshot(&self, snapshot: &Snapshot) {
        if let Err(err) = self.app.emit_to(flyout::LABEL, events::SNAPSHOT, snapshot) {
            diag::warn("cannot send the snapshot to the flyout", &err);
        }
    }

    fn emit_settings(&self, settings: &Settings) {
        if let Err(err) = self.app.emit_to(flyout::LABEL, events::SETTINGS_CHANGED, settings) {
            diag::warn("cannot send the settings to the flyout", &err);
        }
    }

    fn icon_size(&self) -> u32 {
        // The notification area lives on the primary monitor's taskbar.
        let scale = self.app.primary_monitor().ok().flatten().map_or(1.0, |m| m.scale_factor());
        icon::icon_size(scale)
    }

    fn set_tray_icon(&self, size: u32, gauge: Gauge) {
        let Some(tray) = self.tray() else { return };
        let rgba = icon::render(gauge, size);
        let image = Image::new(&rgba.pixels, rgba.size, rgba.size);
        if let Err(err) = tray.set_icon(Some(image)) {
            diag::warn("cannot update the tray icon", &err);
        }
    }

    fn set_tray_tooltip(&self, text: &str) {
        let Some(tray) = self.tray() else { return };
        if let Err(err) = tray.set_tooltip(Some(text)) {
            diag::warn("cannot update the tray tooltip", &err);
        }
    }

    fn set_language(&self, language: Language) {
        tray::set_language(&self.app, language);
    }

    fn toast(&self, notification: &Notification) {
        let shown = self
            .app
            .notification()
            .builder()
            .title(&notification.title)
            .body(&notification.body)
            .show();
        if let Err(err) = shown {
            diag::warn("cannot show a notification", &err);
        }
    }

    fn sync_autostart(&self, enabled: bool, origin: Origin) {
        let Some(applier) = self.autostart.clone() else { return };
        applier.request(enabled, origin);
        // The registry write can stall; it never runs on the scheduler's thread.
        tauri::async_runtime::spawn_blocking(move || {
            if let Err(err) = applier.apply() {
                diag::warn("cannot update the Windows startup entry", &err);
            }
        });
    }

    fn autostart_failed(&self) -> bool {
        self.autostart.as_ref().is_some_and(|applier| applier.failed())
    }

    fn set_content_protected(&self, protected: bool) {
        for window in self.app.webview_windows().values() {
            if let Err(err) = window.set_content_protected(protected) {
                diag::warn("cannot change the capture protection of a window", &err);
            }
        }
    }

    fn sync_local_api(&self, enabled: bool) {
        let mut slot = self.local_api.lock().unwrap_or_else(PoisonError::into_inner);
        if !enabled {
            // Dropping stops the listener and waits for its accept thread (tens of ms).
            slot.running = None;
            slot.failed = false;
        } else if slot.running.is_none() {
            match LocalApi::start(self.api_provider()) {
                Ok(api) => {
                    slot.running = Some(api);
                    slot.failed = false;
                }
                Err(err) => {
                    // One line per failure streak: the tick retries and must not flood stderr.
                    if !slot.failed {
                        diag::warn("cannot start the local API", &err);
                    }
                    slot.failed = true;
                }
            }
        }
    }

    fn local_api_failed(&self) -> bool {
        self.local_api.lock().unwrap_or_else(PoisonError::into_inner).failed
    }
}
