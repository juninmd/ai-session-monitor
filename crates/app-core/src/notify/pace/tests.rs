use usage_model::settings::Language;
use usage_model::{HarnessView, UsageWindow, WindowKind};

use super::*;
use crate::notify::NotificationKind;

const NOW: i64 = 1_800_000_000;
const EPOCH_A: i64 = NOW + 150 * 60;
const EPOCH_B: i64 = NOW + 400 * 60;

fn window(used: f64, resets_at: i64) -> UsageWindow {
    let mut w = UsageWindow::new("five_hour", WindowKind::FiveHour);
    w.used_pct = Some(used);
    w.resets_at = Some(resets_at);
    w.window_minutes = Some(300);
    w
}

fn snap(state: ViewState, stale: bool, windows: Vec<UsageWindow>) -> Snapshot {
    let mut view = HarnessView::with_state(HarnessId::Claude, state);
    view.stale = stale;
    view.windows = windows;
    Snapshot { generated_at: NOW, views: vec![view] }
}

fn ready(used: f64, resets_at: i64) -> Snapshot {
    snap(ViewState::Ready, false, vec![window(used, resets_at)])
}

fn on() -> Settings {
    let mut settings = Settings::default();
    settings.general.notify_on_risk = true;
    settings.general.language = Language::PtBr;
    settings
}

#[test]
fn a_window_that_starts_projecting_past_its_renewal_fires_once_per_epoch() {
    let mut state = PaceState::default();
    let calm = ready(20.0, EPOCH_A);
    let risky = ready(80.0, EPOCH_A);
    assert!(state.plan(Some(&calm), &calm, &on(), NOW).is_empty());

    let notes = state.plan(Some(&calm), &risky, &on(), NOW);
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].kind, NotificationKind::PaceRisk);
    assert_eq!((notes[0].harness, notes[0].window_id.as_str()), (HarnessId::Claude, "five_hour"));
    assert_eq!(notes[0].title, "Claude Code: a cota deve acabar antes de renovar");
    assert!(notes[0].body.contains("antes de reiniciar"), "{}", notes[0].body);

    assert!(state.plan(Some(&risky), &risky, &on(), NOW).is_empty());
}

#[test]
fn a_new_reset_epoch_arms_the_window_again() {
    let mut state = PaceState::default();
    let calm = ready(20.0, EPOCH_A);
    let risky = ready(80.0, EPOCH_A);
    assert_eq!(state.plan(Some(&calm), &risky, &on(), NOW).len(), 1);
    let renewed = ready(10.0, EPOCH_B);
    assert!(state.plan(Some(&risky), &renewed, &on(), NOW).is_empty());
    // Same window, new epoch, risky again: it tells again.
    let later = EPOCH_B - 150 * 60;
    assert_eq!(state.plan(Some(&renewed), &ready(80.0, EPOCH_B), &on(), later).len(), 1);
}

#[test]
fn a_window_first_seen_already_at_risk_only_sets_the_baseline() {
    let mut state = PaceState::default();
    assert!(state.plan(None, &ready(80.0, EPOCH_A), &on(), NOW).is_empty());
    let risky = ready(80.0, EPOCH_A);
    assert!(state.plan(Some(&risky), &risky, &on(), NOW).is_empty());
}

#[test]
fn it_stays_silent_when_the_setting_is_off() {
    let mut state = PaceState::default();
    let calm = ready(20.0, EPOCH_A);
    assert!(state.plan(Some(&calm), &ready(80.0, EPOCH_A), &Settings::default(), NOW).is_empty());
}

#[test]
fn stale_failed_and_expired_readings_never_alert() {
    let mut state = PaceState::default();
    let calm = ready(20.0, EPOCH_A);
    let stale = snap(ViewState::Ready, true, vec![window(80.0, EPOCH_A)]);
    assert!(state.plan(Some(&calm), &stale, &on(), NOW).is_empty());
    let failed = snap(ViewState::Error, false, vec![window(80.0, EPOCH_A)]);
    assert!(state.plan(Some(&calm), &failed, &on(), NOW).is_empty());
    let expired = ready(80.0, NOW - 60);
    assert!(state.plan(Some(&calm), &expired, &on(), NOW).is_empty());
}

#[test]
fn a_window_still_too_early_to_extrapolate_does_not_alert() {
    let mut state = PaceState::default();
    // Only a few minutes into a 5 h window: not enough elapsed time to project anything.
    let start = NOW + 297 * 60;
    let calm = ready(0.0, start);
    assert!(state.plan(Some(&calm), &ready(60.0, start), &on(), NOW).is_empty());
}
