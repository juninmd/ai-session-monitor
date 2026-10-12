//! The remains document: used shares per model bucket, lengths and resets only when stated.

use serde_json::{Value, json};
use usage_model::{ErrorCode, UsageWindow, WindowKind};

use super::{NOW, fixture};
use crate::parse::parse_remains;

fn parse(name: &str) -> Result<Vec<UsageWindow>, ErrorCode> {
    let body: Value = serde_json::from_str(&fixture(name)).unwrap();
    parse_remains(&body, NOW).map_err(|error| error.code)
}

fn find<'a>(windows: &'a [UsageWindow], id: &str) -> &'a UsageWindow {
    windows.iter().find(|w| w.id == id).unwrap_or_else(|| panic!("no window {id}"))
}

#[test]
fn each_model_gets_an_interval_and_a_weekly_window_when_the_plan_has_them() {
    let windows = parse("ok.json").unwrap();
    let ids: Vec<&str> = windows.iter().map(|w| w.id.as_str()).collect();
    assert_eq!(ids, ["minimax_m:interval", "minimax_m:weekly", "image_01:interval"]);
    assert!(windows.iter().take(2).all(|w| w.group.as_deref() == Some("MiniMax-M*")));
}

#[test]
fn the_interval_length_and_reset_come_from_the_stated_times() {
    let windows = parse("ok.json").unwrap();
    let interval = find(&windows, "minimax_m:interval");
    assert_eq!((interval.kind, interval.window_minutes), (WindowKind::FiveHour, Some(300)));
    assert_eq!((interval.used_pct, interval.resets_at), (Some(25.0), Some(NOW + 14_400)));
    let daily = find(&windows, "image_01:interval");
    assert_eq!(
        (daily.kind, daily.window_minutes, daily.used_pct),
        (WindowKind::Daily, Some(1440), Some(60.0))
    );
}

#[test]
fn the_weekly_window_has_no_invented_length_or_reset() {
    let windows = parse("ok.json").unwrap();
    let weekly = find(&windows, "minimax_m:weekly");
    assert_eq!((weekly.kind, weekly.used_pct), (WindowKind::Weekly, Some(20.0)));
    assert_eq!((weekly.window_minutes, weekly.resets_at), (None, None));
}

#[test]
fn a_zero_total_is_no_quota_not_zero_percent() {
    let windows = parse("ok.json").unwrap();
    assert!(windows.iter().all(|w| !w.id.starts_with("speech_hd")));
}

#[test]
fn an_explicit_remaining_field_wins_over_the_ambiguous_usage_field() {
    let windows = parse("explicit_remaining.json").unwrap();
    assert_eq!(find(&windows, "minimax_m:interval").used_pct, Some(75.0));
    assert_eq!(find(&windows, "minimax_m:weekly").used_pct, Some(0.0));
}

#[test]
fn odd_numbers_are_clamped_or_unknown() {
    let windows = parse("odd_numbers.json").unwrap();
    let interval = find(&windows, "m1:interval");
    assert_eq!(interval.used_pct, Some(0.0), "more left than the total reads as unused");
    assert_eq!(interval.window_minutes, None, "a span beyond a week is a garbled time");
    assert_eq!(interval.resets_at, None);
    assert!(windows.iter().all(|w| w.id != "m1:weekly" && !w.id.starts_with("m2")));
}

#[test]
fn unusable_names_are_skipped_and_nothing_readable_means_the_endpoint_changed() {
    assert_eq!(parse("hostile.json").unwrap_err(), ErrorCode::EndpointChanged);
}

#[test]
fn an_empty_list_is_a_plan_without_buckets_not_an_error() {
    assert_eq!(parse("empty.json").unwrap(), Vec::<UsageWindow>::new());
}

#[test]
fn a_nonzero_base_resp_status_is_a_bad_response() {
    assert_eq!(parse("failed.json").unwrap_err(), ErrorCode::BadResponse);
}

#[test]
fn documents_of_the_wrong_shape_mean_the_endpoint_changed() {
    for body in
        [json!([]), json!(null), json!({}), json!({"model_remains": {}}), json!({"data": []})]
    {
        assert_eq!(
            parse_remains(&body, NOW).unwrap_err().code,
            ErrorCode::EndpointChanged,
            "{body}"
        );
    }
}
