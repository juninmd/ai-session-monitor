//! The usages body as windows: both response shapes, and every way a field can be wrong.

use serde_json::{Value, json};
use usage_model::{AmountUnit, ErrorCode, UsageWindow, WindowKind};

use super::{NOW, fixture};
use crate::parse::{Parsed, parse_usage};

fn parse_fixture(name: &str) -> Result<Parsed, usage_model::HarnessError> {
    let body: Value = serde_json::from_str(&fixture(name)).unwrap();
    parse_usage(&body, NOW)
}

fn window<'a>(parsed: &'a Parsed, id: &str) -> &'a UsageWindow {
    parsed.windows.iter().find(|window| window.id == id).unwrap_or_else(|| panic!("no {id}"))
}

fn ids(parsed: &Parsed) -> Vec<&str> {
    parsed.windows.iter().map(|window| window.id.as_str()).collect()
}

#[test]
fn the_legacy_shape_gives_a_five_hour_and_a_weekly_window_with_counts() {
    let parsed = parse_fixture("ok_legacy.json").unwrap();
    assert_eq!(ids(&parsed), ["five_hour", "weekly"]);
    assert_eq!(parsed.plan.as_deref(), Some("Intermediate"));

    let five = window(&parsed, "five_hour");
    assert_eq!(five.kind, WindowKind::FiveHour);
    assert_eq!(five.window_minutes, Some(300), "the body states the length");
    assert_eq!(five.used_pct, Some(25.0));
    assert_eq!(five.resets_at, Some(NOW + 7_200));
    let amount = five.amount.as_ref().unwrap();
    assert_eq!((amount.used, amount.limit, amount.unit), (50.0, Some(200.0), AmountUnit::Requests));

    let weekly = window(&parsed, "weekly");
    assert_eq!(weekly.kind, WindowKind::Weekly);
    assert_eq!(weekly.window_minutes, None, "the body names no length for the top-level block");
    assert_eq!(weekly.used_pct, Some(25.0));
    assert_eq!(weekly.resets_at, Some(NOW + 3 * 86_400), "nanoseconds are dropped");
}

#[test]
fn the_newer_shape_gives_the_monthly_pool_and_completes_an_empty_rolling_window() {
    let parsed = parse_fixture("ok_newer.json").unwrap();
    assert_eq!(ids(&parsed), ["five_hour", "monthly"]);
    assert_eq!(parsed.plan, None);

    let five = window(&parsed, "five_hour");
    assert_eq!(five.used_pct, Some(25.0), "limits[] carried limit 0, the ratio fills it in");
    assert_eq!(five.resets_at, Some(NOW + 7_200));
    assert_eq!(five.amount, None, "a ratio implies no request counts");

    let monthly = window(&parsed, "monthly");
    assert_eq!(monthly.kind, WindowKind::Monthly);
    assert_eq!(monthly.window_minutes, None);
    assert_eq!(monthly.used_pct, Some(40.0));
    assert_eq!(monthly.resets_at, Some(1_801_440_000));
    assert!(
        parsed.windows.iter().all(|w| w.id != "limit_month_code"),
        "the Code slice is not a pool"
    );
}

#[test]
fn a_limits_percentage_wins_over_the_ratio_for_the_same_window() {
    let body = json!({
        "limits": [{
            "window": { "duration": 300, "timeUnit": "TIME_UNIT_MINUTE" },
            "detail": { "limit": "100", "used": "10", "resetTime": "2027-01-15T10:00:00Z" }
        }],
        "usages": { "limit_5h": { "used_ratio": 0.9, "reset_time": "2027-01-15T10:00:00Z" } }
    });
    let parsed = parse_usage(&body, NOW).unwrap();
    assert_eq!(window(&parsed, "five_hour").used_pct, Some(10.0));
    assert_eq!(parsed.windows.len(), 1);
}

#[test]
fn a_ratio_alone_gives_the_rolling_window_when_limits_is_absent() {
    let body = json!({ "limits": null, "usages": {
        "limit_5h": { "used_ratio": 0.5, "reset_time": "2027-01-15T10:00:00Z" } } });
    let parsed = parse_usage(&body, NOW).unwrap();
    let five = window(&parsed, "five_hour");
    assert_eq!((five.used_pct, five.window_minutes), (Some(50.0), Some(300)));
}

#[test]
fn a_past_reset_is_kept_so_the_view_can_call_the_window_expired() {
    let parsed = parse_fixture("expired.json").unwrap();
    for window in &parsed.windows {
        assert_eq!(window.resets_at, Some(NOW - 3_600), "{}", window.id);
        assert_eq!(window.used_pct, Some(100.0), "{}", window.id);
    }
    assert_eq!(window(&parsed, "five_hour").window_minutes, Some(300), "5 hours is 300 minutes");
}

#[test]
fn other_lengths_are_named_by_their_stated_duration_and_an_overshoot_stops_at_100() {
    let parsed = parse_fixture("extra_windows.json").unwrap();
    assert_eq!(ids(&parsed), ["five_hour", "daily", "window_90m"]);
    assert_eq!(window(&parsed, "five_hour").used_pct, Some(10.0), "the first of two stands");
    let daily = window(&parsed, "daily");
    assert_eq!(
        (daily.kind, daily.window_minutes, daily.used_pct),
        (WindowKind::Daily, Some(1_440), Some(100.0))
    );
    assert_eq!(daily.amount.as_ref().map(|a| a.used), Some(60.0), "the count is not clamped");
    let odd = window(&parsed, "window_90m");
    assert_eq!(
        (odd.kind, odd.window_minutes, odd.used_pct),
        (WindowKind::Other, Some(90), Some(60.0))
    );
}

#[test]
fn a_block_without_counts_keeps_its_reset_and_an_unknown_percentage() {
    let body = json!({
        "usage": { "resetTime": "2027-01-18T08:00:00Z" },
        "limits": [{
            "window": { "duration": 300, "timeUnit": "MINUTE" },
            "detail": { "limit": 200, "used": 0, "resetTime": "2027-01-15T10:00:00Z" }
        }]
    });
    let parsed = parse_usage(&body, NOW).unwrap();
    let weekly = window(&parsed, "weekly");
    assert_eq!((weekly.used_pct, weekly.resets_at), (None, Some(NOW + 3 * 86_400)));
    assert_eq!(window(&parsed, "five_hour").used_pct, Some(0.0), "a real zero stays zero");
}

#[test]
fn bodies_that_are_not_the_usage_document_are_endpoint_changed() {
    for body in [
        json!({}),
        json!([]),
        json!(null),
        json!("usage"),
        json!({ "usage": {}, "limits": [] }),
        json!({ "limits": [{ "window": { "duration": 300, "timeUnit": "MINUTE" },
                             "detail": { "limit": "0", "used": "0" } }] }),
    ] {
        let error = parse_usage(&body, NOW).unwrap_err();
        assert_eq!((error.code, error.retryable), (ErrorCode::EndpointChanged, false), "{body}");
        assert!(error.params.is_empty(), "no body text in the error: {body}");
    }
}

#[test]
fn wrong_types_and_hostile_numbers_never_become_a_percentage() {
    for name in ["wrong_types.json", "hostile_numbers.json"] {
        let error = parse_fixture(name).unwrap_err();
        assert_eq!(error.code, ErrorCode::EndpointChanged, "{name}");
    }
}

#[test]
fn a_reset_in_the_far_future_or_before_the_epoch_is_unknown() {
    for reset in ["9999-12-31T00:00:00Z", "1969-12-31T23:59:59Z", "soon", ""] {
        let body = json!({ "usage": { "limit": 10, "used": 5, "resetTime": reset } });
        let parsed = parse_usage(&body, NOW).unwrap();
        assert_eq!(window(&parsed, "weekly").resets_at, None, "{reset:?}");
        assert_eq!(window(&parsed, "weekly").used_pct, Some(50.0));
    }
}

#[test]
fn the_plan_is_shown_only_when_it_is_a_plain_token() {
    let with = |level: Value| {
        let body = json!({ "user": { "membership": { "level": level } },
                           "usage": { "limit": 10, "used": 5 } });
        parse_usage(&body, NOW).unwrap().plan
    };
    assert_eq!(with(json!("LEVEL_TOP_TIER")).as_deref(), Some("Top Tier"));
    assert_eq!(with(json!("LEVEL_")), None);
    assert_eq!(with(json!("<script>alert(1)</script>")), None);
    assert_eq!(with(json!("Andante 49")), None);
    assert_eq!(with(json!(42)), None);
    assert_eq!(with(json!("LEVEL_ÇÃO")), None);
}

#[test]
fn string_and_number_counts_read_the_same() {
    let strings = json!({ "usage": { "limit": "2048", "used": "214" } });
    let numbers = json!({ "usage": { "limit": 2048, "used": 214 } });
    assert_eq!(
        parse_usage(&strings, NOW).unwrap().windows,
        parse_usage(&numbers, NOW).unwrap().windows
    );
}

#[test]
fn an_overage_clamps_the_percent_but_the_amount_keeps_the_real_number() {
    let body = json!({ "usage": { "limit": 100, "used": 150 } });
    let weekly = parse_usage(&body, NOW).unwrap().windows.remove(0);
    assert_eq!(weekly.used_pct, Some(100.0));
    assert_eq!(weekly.amount.map(|a| a.used), Some(150.0));
}

fn weekly_entry(reset: &str) -> Value {
    json!({ "window": { "duration": 10080, "timeUnit": "TIME_UNIT_MINUTE" },
            "detail": { "limit": 100, "used": 10, "resetTime": reset } })
}

#[test]
fn a_ten_thousand_minute_entry_alone_is_limit_weekly_with_its_stated_length() {
    let body = json!({ "limits": [weekly_entry("2027-01-18T08:00:00Z")] });
    let parsed = parse_usage(&body, NOW).unwrap();
    assert_eq!(ids(&parsed), ["limit_weekly"]);
    let weekly = window(&parsed, "limit_weekly");
    assert_eq!((weekly.kind, weekly.window_minutes), (WindowKind::Weekly, Some(10_080)));
}

#[test]
fn a_weekly_entry_and_the_top_level_block_fold_into_one_window_preferring_a_stated_reset() {
    let top = json!({ "limit": 100, "used": 50, "resetTime": "2027-01-18T08:00:00Z" });
    // Both state a reset: the rolling entry stands.
    let body = json!({ "usage": top, "limits": [weekly_entry("2027-01-17T08:00:00Z")] });
    let parsed = parse_usage(&body, NOW).unwrap();
    assert_eq!(ids(&parsed), ["limit_weekly"]);
    assert_eq!(parsed.windows[0].resets_at, Some(NOW + 2 * 86_400));
    // Only the top-level block states one: it wins.
    let body = json!({ "usage": top, "limits": [{ "window": { "duration": 10080, "timeUnit": "MINUTE" },
        "detail": { "limit": 100, "used": 10 } }] });
    let parsed = parse_usage(&body, NOW).unwrap();
    assert_eq!(ids(&parsed), ["weekly"]);
    assert_eq!(parsed.windows[0].resets_at, Some(NOW + 3 * 86_400));
}

#[test]
fn a_reset_farther_than_a_five_hour_window_is_unknown_but_the_percentage_stays() {
    let body = json!({ "limits": [{ "window": { "duration": 300, "timeUnit": "MINUTE" },
        "detail": { "limit": 100, "used": 30, "resetTime": "2027-01-17T08:00:00Z" } }] });
    let five = parse_usage(&body, NOW).unwrap().windows.remove(0);
    assert_eq!((five.used_pct, five.resets_at), (Some(30.0), None));
    let body = json!({ "limits": [{ "window": { "duration": 300, "timeUnit": "MINUTE" },
        "detail": { "limit": 100, "used": 30, "resetTime": "2027-01-15T13:02:00Z" } }] });
    let five = parse_usage(&body, NOW).unwrap().windows.remove(0);
    assert_eq!(five.resets_at, Some(NOW + 5 * 3_600 + 120), "the ceiling itself is accepted");
}
