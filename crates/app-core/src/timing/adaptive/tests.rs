use usage_model::WindowKind;

use super::*;

const NOW: i64 = 1_800_000_000;

const fn secs(n: u64) -> Duration {
    Duration::from_secs(n)
}

fn window(used: Option<f64>, resets_in: Option<i64>) -> UsageWindow {
    let mut w = UsageWindow::new("five_hour", WindowKind::FiveHour);
    w.used_pct = used;
    w.resets_at = resets_in.map(|s| NOW + s);
    w
}

fn wait(base: u64, windows: &[UsageWindow], idle: u32) -> u64 {
    adaptive_wait(secs(base), windows, idle, 70.0, NOW).as_secs()
}

#[test]
fn a_calm_source_stretches_one_step_per_unchanged_read() {
    let calm = [window(Some(20.0), Some(4 * 3600))];
    let steps: Vec<u64> = (0..5).map(|idle| wait(60, &calm, idle)).collect();
    assert_eq!(steps, [120, 180, 240, 300, 360]);
}

#[test]
fn stretching_stops_at_thirty_minutes_but_never_below_the_base() {
    let calm = [window(Some(5.0), Some(48 * 3600))];
    assert_eq!(wait(60, &calm, 500), 1800);
    assert_eq!(wait(60, &calm, u32::MAX), 1800);
    assert_eq!(wait(3600, &calm, 0), 3600, "a slow base is never shortened");
    assert_eq!(wait(300, &calm, 0), 600, "OpenCode keeps its 300 s floor");
}

#[test]
fn a_window_near_its_limit_keeps_the_base_cadence() {
    for used in [70.0, 85.0, 100.0, 140.0] {
        let hot = [window(Some(10.0), Some(5 * 3600)), window(Some(used), Some(5 * 3600))];
        assert_eq!(wait(60, &hot, 9), 60, "{used}%");
    }
    assert_eq!(wait(60, &[window(Some(69.9), Some(5 * 3600))], 0), 120);
}

#[test]
fn a_reset_within_ten_minutes_keeps_the_base_cadence() {
    assert_eq!(wait(60, &[window(Some(5.0), Some(600))], 3), 60);
    assert_eq!(wait(60, &[window(Some(5.0), Some(30))], 3), 60);
    assert_eq!(
        wait(60, &[window(Some(5.0), Some(601))], 3),
        60,
        "one second of calm is below base"
    );
}

#[test]
fn the_stretch_never_sleeps_past_ten_minutes_before_the_nearest_reset() {
    let near = [window(Some(5.0), Some(15 * 60)), window(Some(5.0), Some(9 * 3600))];
    assert_eq!(wait(60, &near, 40), 300, "wakes 10 minutes before the 15-minute reset");
}

#[test]
fn unknown_usage_or_no_window_keeps_the_base_cadence() {
    assert_eq!(wait(60, &[window(None, Some(5 * 3600))], 5), 60);
    assert_eq!(wait(60, &[window(Some(5.0), Some(5 * 3600)), window(None, None)], 5), 60);
    assert_eq!(wait(60, &[], 5), 60);
}

#[test]
fn expired_windows_are_ignored_and_all_expired_means_base() {
    let expired = window(Some(95.0), Some(-5));
    let calm = window(Some(10.0), Some(5 * 3600));
    assert_eq!(wait(60, &[expired.clone(), calm], 0), 120);
    assert_eq!(wait(60, &[expired], 0), 60);
}

#[test]
fn a_window_without_a_reset_time_still_stretches_when_calm() {
    assert_eq!(wait(60, &[window(Some(10.0), None)], 1), 180);
}

#[test]
fn an_early_alert_lowers_the_hot_line_but_the_default_keeps_seventy() {
    let mut settings = Settings::default();
    assert!((hot_pct(&settings) - 70.0).abs() < f64::EPSILON);
    settings.general.alert_thresholds = vec![50, 90];
    assert!((hot_pct(&settings) - 40.0).abs() < f64::EPSILON);
    settings.general.alert_thresholds = vec![5];
    assert!((hot_pct(&settings) - 1.0).abs() < f64::EPSILON);
    settings.general.alert_thresholds.clear();
    assert!((hot_pct(&settings) - 70.0).abs() < f64::EPSILON);
}
