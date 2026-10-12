use usage_model::{HarnessView, UsageWindow, WindowKind};

use super::*;
use crate::notify::NotificationKind;

const NOW: i64 = 1_800_000_000;
const EPOCH_A: i64 = NOW + 3_600;
const EPOCH_B: i64 = NOW + 20_000;

fn window(id: &str, used: Option<f64>, resets_at: Option<i64>) -> UsageWindow {
    let mut w = UsageWindow::new(id, WindowKind::FiveHour);
    w.used_pct = used;
    w.resets_at = resets_at;
    w
}

fn snap(harness: HarnessId, state: ViewState, stale: bool, windows: Vec<UsageWindow>) -> Snapshot {
    let mut view = HarnessView::with_state(harness, state);
    view.stale = stale;
    view.windows = windows;
    Snapshot { generated_at: NOW, views: vec![view] }
}

fn ready(used: f64, resets_at: i64) -> Snapshot {
    snap(
        HarnessId::Claude,
        ViewState::Ready,
        false,
        vec![window("five_hour", Some(used), Some(resets_at))],
    )
}

fn run(state: &mut ThresholdState, prev: &Snapshot, next: &Snapshot) -> Vec<Notification> {
    state.plan(Some(prev), next, &Settings::default(), NOW)
}

fn levels(notes: &[Notification]) -> Vec<String> {
    notes.iter().map(|n| n.title.clone()).collect()
}

#[test]
fn crossing_a_threshold_fires_once_and_not_again_while_it_stays_above() {
    let mut state = ThresholdState::default();
    let notes = run(&mut state, &ready(70.0, EPOCH_A), &ready(81.0, EPOCH_A));
    assert_eq!(levels(&notes), ["Claude Code: 80% do limite usado"]);
    assert_eq!(notes[0].kind, NotificationKind::ThresholdCrossed);
    assert_eq!((notes[0].harness, notes[0].window_id.as_str()), (HarnessId::Claude, "five_hour"));
    assert!(notes[0].body.contains("já usou 81%"), "{}", notes[0].body);

    assert!(run(&mut state, &ready(81.0, EPOCH_A), &ready(88.0, EPOCH_A)).is_empty());
    let next = run(&mut state, &ready(88.0, EPOCH_A), &ready(95.0, EPOCH_A));
    assert_eq!(levels(&next), ["Claude Code: 95% do limite usado"]);
    assert!(run(&mut state, &ready(95.0, EPOCH_A), &ready(99.0, EPOCH_A)).is_empty());
}

#[test]
fn a_jump_across_two_thresholds_fires_only_the_highest() {
    let mut state = ThresholdState::default();
    let notes = run(&mut state, &ready(40.0, EPOCH_A), &ready(97.0, EPOCH_A));
    assert_eq!(levels(&notes), ["Claude Code: 95% do limite usado"]);
    assert!(
        run(&mut state, &ready(97.0, EPOCH_A), &ready(98.0, EPOCH_A)).is_empty(),
        "80 is spent too"
    );
}

#[test]
fn a_reset_re_arms_the_window() {
    let mut state = ThresholdState::default();
    run(&mut state, &ready(40.0, EPOCH_A), &ready(96.0, EPOCH_A));
    assert!(run(&mut state, &ready(96.0, EPOCH_A), &ready(5.0, EPOCH_B)).is_empty());
    let again = run(&mut state, &ready(5.0, EPOCH_B), &ready(82.0, EPOCH_B));
    assert_eq!(levels(&again), ["Claude Code: 80% do limite usado"]);
}

#[test]
fn a_dip_within_the_same_epoch_does_not_repeat_the_toast() {
    let mut state = ThresholdState::default();
    run(&mut state, &ready(70.0, EPOCH_A), &ready(82.0, EPOCH_A));
    run(&mut state, &ready(82.0, EPOCH_A), &ready(74.0, EPOCH_A));
    assert!(run(&mut state, &ready(74.0, EPOCH_A), &ready(83.0, EPOCH_A)).is_empty());
}

#[test]
fn the_first_reading_after_a_start_only_sets_the_baseline() {
    let mut state = ThresholdState::default();
    let loading = snap(HarnessId::Claude, ViewState::Loading, false, Vec::new());
    assert!(run(&mut state, &loading, &ready(85.0, EPOCH_A)).is_empty(), "restart at 85 %");
    assert!(state.plan(None, &ready(85.0, EPOCH_A), &Settings::default(), NOW).is_empty());
    let up = run(&mut state, &ready(85.0, EPOCH_A), &ready(96.0, EPOCH_A));
    assert_eq!(levels(&up), ["Claude Code: 95% do limite usado"], "later crossings still fire");
}

#[test]
fn a_fresh_state_after_a_restart_does_not_repeat_a_toast_for_a_known_level() {
    let mut first_run = ThresholdState::default();
    run(&mut first_run, &ready(70.0, EPOCH_A), &ready(85.0, EPOCH_A));
    let mut second_run = ThresholdState::default();
    let loading = snap(HarnessId::Claude, ViewState::Loading, false, Vec::new());
    assert!(run(&mut second_run, &loading, &ready(85.0, EPOCH_A)).is_empty());
    assert!(run(&mut second_run, &ready(85.0, EPOCH_A), &ready(86.0, EPOCH_A)).is_empty());
}

#[test]
fn unknown_stale_expired_and_non_ready_readings_never_fire() {
    let prev = ready(10.0, EPOCH_A);
    let cases = [
        snap(
            HarnessId::Claude,
            ViewState::Ready,
            false,
            vec![window("five_hour", None, Some(EPOCH_A))],
        ),
        snap(
            HarnessId::Claude,
            ViewState::Ready,
            true,
            vec![window("five_hour", Some(99.0), Some(EPOCH_A))],
        ),
        snap(
            HarnessId::Claude,
            ViewState::Ready,
            false,
            vec![window("five_hour", Some(99.0), Some(NOW - 1))],
        ),
        snap(
            HarnessId::Claude,
            ViewState::Error,
            false,
            vec![window("five_hour", Some(99.0), Some(EPOCH_A))],
        ),
        snap(HarnessId::Claude, ViewState::Loading, false, Vec::new()),
    ];
    for next in cases {
        assert!(run(&mut ThresholdState::default(), &prev, &next).is_empty(), "{next:?}");
    }
}

#[test]
fn a_window_new_to_the_snapshot_is_a_baseline_not_a_crossing() {
    let mut state = ThresholdState::default();
    let both = snap(
        HarnessId::Claude,
        ViewState::Ready,
        false,
        vec![
            window("five_hour", Some(10.0), Some(EPOCH_A)),
            window("weekly", Some(90.0), Some(EPOCH_B)),
        ],
    );
    assert!(run(&mut state, &ready(10.0, EPOCH_A), &both).is_empty());
}

#[test]
fn each_window_and_harness_has_its_own_state() {
    let mut state = ThresholdState::default();
    let pair = |a: f64, b: f64| {
        snap(
            HarnessId::Claude,
            ViewState::Ready,
            false,
            vec![
                window("five_hour", Some(a), Some(EPOCH_A)),
                window("weekly", Some(b), Some(EPOCH_B)),
            ],
        )
    };
    let notes = run(&mut state, &pair(10.0, 10.0), &pair(85.0, 10.0));
    assert_eq!(notes.iter().map(|n| n.window_id.as_str()).collect::<Vec<_>>(), ["five_hour"]);
    let notes = run(&mut state, &pair(85.0, 10.0), &pair(85.0, 96.0));
    assert_eq!(notes.iter().map(|n| n.window_id.as_str()).collect::<Vec<_>>(), ["weekly"]);
}

#[test]
fn custom_thresholds_apply_and_an_empty_list_switches_the_alerts_off() {
    let mut custom = Settings::default();
    custom.general.alert_thresholds = vec![50];
    let mut state = ThresholdState::default();
    let notes = state.plan(Some(&ready(40.0, EPOCH_A)), &ready(55.0, EPOCH_A), &custom, NOW);
    assert_eq!(levels(&notes), ["Claude Code: 50% do limite usado"]);

    let mut off = Settings::default();
    off.general.alert_thresholds.clear();
    let mut state = ThresholdState::default();
    assert!(state.plan(Some(&ready(10.0, EPOCH_A)), &ready(99.0, EPOCH_A), &off, NOW).is_empty());
}

#[test]
fn the_exact_threshold_counts_and_state_is_pruned_with_the_windows() {
    let mut state = ThresholdState::default();
    let notes = run(&mut state, &ready(79.9, EPOCH_A), &ready(80.0, EPOCH_A));
    assert_eq!(notes.len(), 1);
    let gone = snap(HarnessId::Claude, ViewState::Disabled, false, Vec::new());
    run(&mut state, &ready(80.0, EPOCH_A), &gone);
    assert!(state.fired.is_empty());
}

#[test]
fn the_toast_text_carries_only_the_window_name_and_numbers() {
    let mut labelled = window("budget", Some(96.0), Some(EPOCH_A));
    labelled.kind = WindowKind::Budget;
    labelled.label = Some("team-a\u{7}".to_owned());
    let prev = snap(
        HarnessId::Litellm,
        ViewState::Ready,
        false,
        vec![window("budget", Some(10.0), Some(EPOCH_A))],
    );
    let next = snap(HarnessId::Litellm, ViewState::Ready, false, vec![labelled]);
    let notes = run(&mut ThresholdState::default(), &prev, &next);
    assert_eq!(notes[0].title, "LiteLLM: 95% do limite usado");
    assert_eq!(notes[0].body, "O orçamento (team-a) já usou 96%. Reinicia em 1 h.");
}
