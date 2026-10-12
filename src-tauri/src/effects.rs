//! The Tauri side of `Effects`: events to the webview, the tray icon and its tooltip, toasts and
//! the Windows startup entry. Plumbing only; every decision lives in `sink`.

use std::sync::Arc;

use app_core::Notification;
use tauri::image::Image;
use tauri::tray::TrayIcon;
use tauri::{AppHandle, Emitter};
use tauri_plugin_notification::NotificationExt;
use usage_model::{Settings, Snapshot};

use crate::autostart::{Applier, Origin};
use crate::icon::{self, Gauge};
use crate::sink::Effects;
use crate::{diag, events, flyout, tray};

pub struct TauriEffects {
    app: AppHandle,
    /// `None` under a test override, which must never touch the Run key.
    autostart: Option<Arc<Applier>>,
}

impl TauriEffects {
    pub fn new(app: &AppHandle, autostart: Option<Arc<Applier>>) -> Self {
        Self { app: app.clone(), autostart }
    }

    /// Looked up per call and never kept: Tauri removes the icon when the last handle drops, so a
    /// handle held until `exit` would leave a ghost icon behind in the notification area.
    fn tray(&self) -> Option<TrayIcon> {
        self.app.tray_by_id(tray::TRAY_ID)
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
}
