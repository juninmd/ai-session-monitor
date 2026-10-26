use tauri::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize, Position, Size};

use super::*;

#[test]
fn a_physical_tray_rect_is_taken_as_is() {
    let rect = tauri::Rect {
        position: Position::Physical(PhysicalPosition::new(1700, 1048)),
        size: Size::Physical(PhysicalSize::new(24, 24)),
    };
    assert_eq!(tray_rect(&rect, 1.5), Rect::new(1700, 1048, 24, 24));
}

#[test]
fn a_logical_tray_rect_is_scaled_to_physical_pixels() {
    let rect = tauri::Rect {
        position: Position::Logical(LogicalPosition::new(1000.0, 700.0)),
        size: Size::Logical(LogicalSize::new(16.0, 16.0)),
    };
    assert_eq!(tray_rect(&rect, 1.5), Rect::new(1500, 1050, 24, 24));
}

#[test]
fn pixel_conversion_saturates_instead_of_wrapping() {
    assert_eq!(px(1920), 1920);
    assert_eq!(px(u32::MAX), i32::MAX);
}

#[test]
fn the_flyout_is_the_window_the_config_declares() {
    let config: serde_json::Value =
        serde_json::from_str(include_str!("../../tauri.conf.json")).expect("json");
    let window = &config["app"]["windows"][0];
    assert_eq!(window["label"], LABEL);
    assert_eq!(window["width"].as_f64(), Some(WIDTH_LOGICAL));
    assert_eq!(window["height"].as_f64(), Some(INITIAL_HEIGHT_LOGICAL));
    assert_eq!(window["visible"], false);
}

#[test]
fn the_shell_builds_the_window_itself_and_places_it() {
    // `create: false` keeps Tauri from opening the webview before the service and updater are
    // managed; `center: false` keeps the OS from choosing the position; `shadow: false` with
    // `decorations: false` and `transparent: true` prevents tao's undecorated-shadow non-client
    // frame (which otherwise adds an opaque 16x9 border and breaks inner_size settling).
    let config: serde_json::Value =
        serde_json::from_str(include_str!("../../tauri.conf.json")).expect("json");
    let window = &config["app"]["windows"][0];
    assert_eq!(window["create"], false, "the shell builds the window once the service exists");
    assert_eq!(window["center"], false, "the shell positions the window from the tray");
    assert_eq!(window["decorations"], false, "the flyout draws its own rounded frame");
    assert_eq!(window["transparent"], true, "the flyout surface is transparent");
    assert_eq!(window["resizable"], true, "the flyout can be resized by its edges");
    assert_eq!(window["shadow"], false, "undecorated shadow adds an opaque Win32 non-client frame");
}
