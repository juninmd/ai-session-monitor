use usage_model::{HarnessId, UsageWindow, WindowKind};

use super::ledger::{Ledger, MAX_POINTS, RETENTION_SECS, SAMPLE_GAP_SECS};
use super::query::thin;

pub(super) const NOW: i64 = 1_800_000_000;
const RESET: i64 = NOW + 3 * 3600;

pub(super) fn window(id: &str, used: Option<f64>, resets_at: Option<i64>) -> UsageWindow {
    let mut w = UsageWindow::new(id, WindowKind::FiveHour);
    w.used_pct = used;
    w.resets_at = resets_at;
    w
}

fn record(ledger: &mut Ledger, w: UsageWindow, now: i64) -> bool {
    ledger.record(HarnessId::Claude, &[w], now)
}

#[test]
fn an_unknown_percentage_is_skipped_never_stored_as_zero() {
    let mut ledger = Ledger::default();
    assert!(!record(&mut ledger, window("five_hour", None, Some(RESET)), NOW));
    assert!(!record(&mut ledger, window("five_hour", Some(f64::NAN), Some(RESET)), NOW));
    assert!(ledger.is_empty());
}

#[test]
fn a_window_without_a_reset_or_already_reset_has_no_epoch_to_join() {
    let mut ledger = Ledger::default();
    assert!(!record(&mut ledger, window("w", Some(10.0), None), NOW));
    assert!(!record(&mut ledger, window("w", Some(10.0), Some(NOW)), NOW));
    assert!(ledger.is_empty());
}

#[test]
fn an_unchanged_value_is_sampled_again_only_after_the_gap() {
    let mut ledger = Ledger::default();
    let w = || window("five_hour", Some(40.0), Some(RESET));
    assert!(record(&mut ledger, w(), NOW));
    assert!(!record(&mut ledger, w(), NOW + 60));
    assert!(!record(&mut ledger, w(), NOW + SAMPLE_GAP_SECS - 1));
    assert!(record(&mut ledger, w(), NOW + SAMPLE_GAP_SECS));
    assert_eq!(ledger.point_count(), 2);
}

#[test]
fn a_changed_value_is_sampled_at_once() {
    let mut ledger = Ledger::default();
    assert!(record(&mut ledger, window("five_hour", Some(40.0), Some(RESET)), NOW));
    assert!(record(&mut ledger, window("five_hour", Some(41.0), Some(RESET)), NOW + 30));
    assert_eq!(ledger.point_count(), 2);
}

#[test]
fn a_clock_that_moved_back_adds_nothing() {
    let mut ledger = Ledger::default();
    record(&mut ledger, window("w", Some(40.0), Some(RESET)), NOW);
    assert!(!record(&mut ledger, window("w", Some(50.0), Some(RESET)), NOW - 5));
}

#[test]
fn a_reset_restated_a_few_seconds_apart_stays_one_epoch_but_a_new_window_starts_another() {
    let mut ledger = Ledger::default();
    record(&mut ledger, window("w", Some(10.0), Some(RESET)), NOW);
    record(&mut ledger, window("w", Some(20.0), Some(RESET + 2)), NOW + 60);
    assert_eq!(ledger.epochs(HarnessId::Claude, 8).epochs.len(), 1);
    record(&mut ledger, window("w", Some(3.0), Some(RESET + 5 * 3600)), RESET + 60);
    let epochs = ledger.epochs(HarnessId::Claude, 8).epochs;
    assert_eq!(epochs.len(), 2);
    assert_eq!(epochs[0].points.len(), 2);
}

#[test]
fn values_are_clamped_to_the_scale_and_ids_must_be_plain() {
    let mut ledger = Ledger::default();
    record(&mut ledger, window("w", Some(140.0), Some(RESET)), NOW);
    assert_eq!(ledger.epochs(HarnessId::Claude, 8).epochs[0].points[0].used_pct, 100.0);
    for bad in ["", "a b", "caminho\\x", &"x".repeat(65), "a\nb"] {
        assert!(!record(&mut ledger, window(bad, Some(1.0), Some(RESET)), NOW), "{bad:?}");
    }
}

#[test]
fn points_older_than_the_retention_are_pruned_and_empty_epochs_go_with_them() {
    let mut ledger = Ledger::default();
    record(&mut ledger, window("w", Some(10.0), Some(NOW + 100)), NOW);
    let later = NOW + RETENTION_SECS + 1;
    record(&mut ledger, window("w", Some(5.0), Some(later + 1000)), later);
    ledger.prune(later);
    let epochs = ledger.epochs(HarnessId::Claude, 8).epochs;
    assert_eq!(epochs.len(), 1);
    assert_eq!(epochs[0].resets_at, later + 1000);
}

#[test]
fn over_the_cap_the_oldest_points_go_first() {
    let mut ledger = Ledger::default();
    for n in 0..10 {
        record(&mut ledger, window("w", Some(f64::from(n)), Some(RESET)), NOW + i64::from(n));
    }
    ledger.cap_points(4);
    let points = &ledger.epochs(HarnessId::Claude, 8).epochs[0].points;
    assert_eq!(points.iter().map(|p| p.t - NOW).collect::<Vec<_>>(), [6, 7, 8, 9]);
    assert!(ledger.point_count() <= MAX_POINTS);
}

#[test]
fn only_the_newest_epochs_of_each_window_are_returned_in_order() {
    let mut ledger = Ledger::default();
    for n in 0..5 {
        let at = NOW + i64::from(n) * 10_000;
        record(&mut ledger, window("a", Some(1.0), Some(at + 5_000)), at);
    }
    record(&mut ledger, window("b", Some(1.0), Some(RESET)), NOW);
    let history = ledger.epochs(HarnessId::Claude, 2);
    let ids: Vec<_> = history.epochs.iter().map(|e| (e.window_id.as_str(), e.resets_at)).collect();
    assert_eq!(ids, [("a", NOW + 35_000), ("a", NOW + 45_000), ("b", RESET)]);
    assert!(ledger.epochs(HarnessId::Codex, 8).epochs.is_empty());
}

#[test]
fn thinning_keeps_the_ends_and_the_peak_within_the_budget() {
    let points: Vec<(i64, f64)> =
        (0..1000_u32).map(|n| (i64::from(n), f64::from(n) / 10.0)).collect();
    let thinned = thin(&points, 120);
    assert_eq!(thinned.len(), 120);
    assert_eq!((thinned[0].t, thinned[119].t), (0, 999));
    assert!(thinned.windows(2).all(|pair| pair[0].t < pair[1].t));
    assert_eq!(thinned.iter().map(|p| p.used_pct).fold(0.0, f64::max), 99.9);
    assert_eq!(thin(&points[..50], 120).len(), 50);
}

#[test]
fn sanitizing_drops_what_a_damaged_file_could_carry() {
    let mut ledger = Ledger::default();
    record(&mut ledger, window("w", Some(10.0), Some(RESET)), NOW);
    ledger.series[0].points.extend([(NOW - 5, 20.0), (NOW + 5, f64::NAN), (NOW + 6, 250.0)]);
    ledger.sanitize();
    let times: Vec<i64> = ledger.series[0].points.iter().map(|p| p.0).collect();
    assert_eq!(times, [NOW - 5, NOW]);
}
