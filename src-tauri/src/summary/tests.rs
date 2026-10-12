use usage_model::{HarnessView, UsageWindow, ViewState};

use super::*;

const NOW: i64 = 1_800_000_000;

fn window(
    id: &str,
    kind: WindowKind,
    used: Option<f64>,
    resets_in_min: Option<i64>,
) -> UsageWindow {
    let mut w = UsageWindow::new(id, kind);
    w.used_pct = used;
    w.resets_at = resets_in_min.map(|m| NOW + m * 60);
    w.window_minutes = Some(300);
    w
}

fn ready(id: HarnessId, windows: Vec<UsageWindow>) -> HarnessView {
    let mut view = HarnessView::with_state(id, ViewState::Ready);
    view.windows = windows;
    view
}

fn snapshot(views: Vec<HarnessView>) -> Snapshot {
    Snapshot { generated_at: NOW, views }
}

fn general(source: Option<HarnessId>) -> GeneralSettings {
    GeneralSettings { tray_source: source, ..GeneralSettings::default() }
}

fn five_hour(used: f64) -> UsageWindow {
    window("five_hour", WindowKind::FiveHour, Some(used), Some(120))
}

#[test]
fn follows_the_lowest_available_percentage_of_all_harnesses() {
    let snap = snapshot(vec![
        ready(HarnessId::Claude, vec![five_hour(30.0)]),
        ready(HarnessId::Codex, vec![five_hour(80.0)]),
        ready(HarnessId::Litellm, vec![five_hour(10.0)]),
    ]);
    let got = pick(&snap, &general(None), NOW).expect("a reading");
    assert_eq!(got.harness, HarnessId::Codex);
    assert_eq!(got.available_pct, 20.0);
}

#[test]
fn a_followed_harness_wins_even_when_another_is_lower() {
    let snap = snapshot(vec![
        ready(HarnessId::Claude, vec![five_hour(30.0)]),
        ready(HarnessId::Codex, vec![five_hour(80.0)]),
    ]);
    let got = pick(&snap, &general(Some(HarnessId::Claude)), NOW).expect("a reading");
    assert_eq!(got.harness, HarnessId::Claude);
}

#[test]
fn a_followed_harness_without_a_reading_leaves_the_tray_empty() {
    let snap = snapshot(vec![
        HarnessView::with_state(HarnessId::Claude, ViewState::NoData),
        ready(HarnessId::Codex, vec![five_hour(80.0)]),
    ]);
    assert_eq!(pick(&snap, &general(Some(HarnessId::Claude)), NOW), None);
}

#[test]
fn picks_the_most_constrained_window_of_a_harness() {
    let weekly = window("weekly", WindowKind::Weekly, Some(95.0), Some(5000));
    let snap = snapshot(vec![ready(HarnessId::Claude, vec![five_hour(10.0), weekly])]);
    let got = pick(&snap, &general(None), NOW).expect("a reading");
    assert_eq!(got.kind, WindowKind::Weekly);
    assert_eq!(got.available_pct, 5.0);
    assert_eq!(got.resets_at, Some(NOW + 5000 * 60));
}

#[test]
fn stale_and_unready_views_are_ignored() {
    let mut stale = ready(HarnessId::Claude, vec![five_hour(99.0)]);
    stale.stale = true;
    let loading = HarnessView::loading(HarnessId::Codex);
    let snap = snapshot(vec![stale, loading, ready(HarnessId::Litellm, vec![five_hour(40.0)])]);
    let got = pick(&snap, &general(None), NOW).expect("a reading");
    assert_eq!(got.harness, HarnessId::Litellm);
}

#[test]
fn expired_and_unknown_windows_are_ignored() {
    let expired = window("five_hour", WindowKind::FiveHour, Some(99.0), Some(-1));
    let unknown = window("weekly", WindowKind::Weekly, None, Some(100));
    let snap = snapshot(vec![ready(HarnessId::Claude, vec![expired, unknown])]);
    assert_eq!(pick(&snap, &general(None), NOW), None);
}

#[test]
fn a_tie_goes_to_the_earlier_tab() {
    let snap = snapshot(vec![
        ready(HarnessId::Codex, vec![five_hour(50.0)]),
        ready(HarnessId::Claude, vec![five_hour(50.0)]),
    ]);
    assert_eq!(pick(&snap, &general(None), NOW).expect("a reading").harness, HarnessId::Codex);
}

#[test]
fn the_warning_threshold_comes_from_the_settings() {
    // No window length, so the pace rules stay out of the way and only the threshold decides.
    let mut no_pace = five_hour(80.0);
    no_pace.window_minutes = None;
    let snap = snapshot(vec![ready(HarnessId::Claude, vec![no_pace])]);
    let calm = GeneralSettings { warn_at_pct: 10, ..general(None) };
    let wary = GeneralSettings { warn_at_pct: 25, ..general(None) };
    assert_eq!(pick(&snap, &calm, NOW).expect("a reading").status, Status::Ok);
    assert_eq!(pick(&snap, &wary, NOW).expect("a reading").status, Status::Warn);
}

#[test]
fn an_empty_snapshot_has_no_reading() {
    assert_eq!(pick(&snapshot(Vec::new()), &general(None), NOW), None);
}
