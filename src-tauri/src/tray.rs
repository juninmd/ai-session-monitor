//! The notification-area icon: a left click toggles the flyout, a right click opens the menu.

use std::sync::Arc;

use app_core::AppService;
use tauri::menu::{Menu, MenuEvent, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};

use crate::flyout::Flyout;
use crate::icon::{self, Gauge};

#[cfg(test)]
mod tests;

pub const TRAY_ID: &str = "main";
const APP_NAME: &str = "AI Session Monitor";

/// Menu entries, in the order they appear: id and pt-BR label.
const MENU: [(&str, &str); 3] =
    [("open", "Abrir"), ("refresh", "Atualizar agora"), ("quit", "Sair")];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Open,
    Refresh,
    Quit,
}

fn action(id: &str) -> Option<Action> {
    match id {
        "open" => Some(Action::Open),
        "refresh" => Some(Action::Refresh),
        "quit" => Some(Action::Quit),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Gesture {
    Press,
    Release,
}

/// Only the left button toggles the flyout; the toggle happens on release, after the press had
/// the chance to note whether the flyout was open (see `gate`).
fn gesture(button: MouseButton, state: MouseButtonState) -> Option<Gesture> {
    match (button, state) {
        (MouseButton::Left, MouseButtonState::Down) => Some(Gesture::Press),
        (MouseButton::Left, MouseButtonState::Up) => Some(Gesture::Release),
        _ => None,
    }
}

/// Creates the tray icon with a "no reading yet" ring; the sink replaces it right after startup.
/// Tauri owns the icon from here on: callers reach it with `tray_by_id(TRAY_ID)`, so no handle of
/// ours outlives the app's own cleanup (see `TauriEffects::tray`).
pub fn build(app: &AppHandle, flyout: Arc<Flyout>) -> tauri::Result<()> {
    let items = MENU
        .iter()
        .map(|(id, label)| MenuItem::with_id(app, *id, *label, true, None::<&str>))
        .collect::<tauri::Result<Vec<_>>>()?;
    let entries: Vec<&dyn tauri::menu::IsMenuItem<_>> =
        items.iter().map(|item| item as &dyn tauri::menu::IsMenuItem<_>).collect();
    let menu = Menu::with_items(app, &entries)?;

    let scale = app.primary_monitor().ok().flatten().map_or(1.0, |m| m.scale_factor());
    let size = icon::icon_size(scale);
    let blank = icon::render(Gauge { available_pct: None, status: None }, size);

    let for_clicks = Arc::clone(&flyout);
    TrayIconBuilder::with_id(TRAY_ID)
        .icon(tauri::image::Image::new(&blank.pixels, blank.size, blank.size))
        .tooltip(APP_NAME)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(move |_tray, event| on_icon_event(&for_clicks, &event))
        .on_menu_event(move |app, event| on_menu_event(app, &flyout, &event))
        .build(app)?;
    Ok(())
}

/// Opens the flyout next to the tray icon (or in the notification corner when its position is
/// unknown). Used by the menu and by a second launch of the app.
pub fn open_flyout(app: &AppHandle, flyout: &Flyout) {
    let rect = app.tray_by_id(TRAY_ID).and_then(|tray| tray.rect().ok().flatten());
    flyout.show(rect.as_ref());
}

fn on_icon_event(flyout: &Flyout, event: &TrayIconEvent) {
    let TrayIconEvent::Click { button, button_state, rect, .. } = event else { return };
    match gesture(*button, *button_state) {
        Some(Gesture::Press) => flyout.tray_pressed(),
        Some(Gesture::Release) => flyout.tray_released(rect),
        None => {}
    }
}

fn on_menu_event(app: &AppHandle, flyout: &Flyout, event: &MenuEvent) {
    match action(event.id().as_ref()) {
        Some(Action::Open) => open_flyout(app, flyout),
        Some(Action::Refresh) => {
            if let Some(service) = app.try_state::<Arc<AppService>>() {
                service.refresh(None);
            }
        }
        Some(Action::Quit) => app.exit(0),
        None => {}
    }
}
