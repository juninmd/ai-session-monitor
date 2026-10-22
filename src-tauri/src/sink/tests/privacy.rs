//! The capture-protection and local-API settings reaching the shell, through the same seam as
//! the autostart entry: applied at start-up, on every change, and only when the value changes.

use super::*;

fn with(hide: bool, api: bool) -> Settings {
    let mut settings = settings(None, false);
    settings.general.hide_from_capture = hide;
    settings.general.local_api = api;
    settings
}

fn protections(rig: &Rig) -> Vec<bool> {
    rig.rec.protections.lock().expect("lock").clone()
}

fn api_calls(rig: &Rig) -> Vec<bool> {
    rig.rec.local_api.lock().expect("lock").clone()
}

#[test]
fn capture_protection_is_applied_at_start_and_follows_the_setting() {
    let rig = rig();
    rig.sink.prime(&with(true, false));
    assert_eq!(protections(&rig), [true], "the windows exist at start-up: applied at once");
    rig.sink.settings_changed(&with(true, false));
    assert_eq!(protections(&rig), [true], "an unrelated change leaves it alone");
    rig.sink.settings_changed(&with(false, false));
    rig.sink.settings_changed(&with(true, false));
    assert_eq!(protections(&rig), [true, false, true]);
}

#[test]
fn capture_protection_starts_off_for_the_default_settings() {
    let rig = rig();
    rig.sink.prime(&with(false, false));
    assert_eq!(protections(&rig), [false]);
}

#[test]
fn the_local_api_starts_and_stops_with_its_setting() {
    let rig = rig();
    rig.sink.prime(&with(false, true));
    assert_eq!(api_calls(&rig), [true], "an API left on is started at launch");
    rig.sink.settings_changed(&with(true, true));
    assert_eq!(api_calls(&rig), [true], "an unrelated change does not touch it");
    rig.sink.settings_changed(&with(true, false));
    rig.sink.settings_changed(&with(true, true));
    assert_eq!(api_calls(&rig), [true, false, true], "a later toggle tries again");
}

#[test]
fn a_failed_start_is_retried_on_the_tick_and_only_while_wanted() {
    let rig = rig();
    rig.sink.prime(&with(false, true));
    rig.sink.tick();
    assert_eq!(api_calls(&rig), [true], "nothing failed: the tick leaves it alone");
    rig.rec.local_api_failed.store(true, Ordering::SeqCst);
    rig.sink.tick();
    rig.sink.tick();
    assert_eq!(api_calls(&rig), [true, true, true]);
    rig.sink.settings_changed(&with(false, false));
    rig.rec.local_api_failed.store(false, Ordering::SeqCst);
    rig.sink.tick();
    assert_eq!(api_calls(&rig), [true, true, true, false]);
}

#[test]
fn a_switched_off_api_is_not_retried_even_if_the_last_start_failed() {
    let rig = rig();
    rig.sink.prime(&with(false, false));
    rig.rec.local_api_failed.store(true, Ordering::SeqCst);
    rig.sink.tick();
    assert_eq!(api_calls(&rig), [false]);
}
