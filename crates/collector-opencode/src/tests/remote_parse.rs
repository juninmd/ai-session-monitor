//! The usage body parser: the happy path, the limit notices and the idle-window rule.

use serde_json::{Value, json};
use usage_model::{ErrorCode, HarnessError, Severity, UsageWindow, WindowKind};

use super::fake_db::NOW;
use super::{fixture, iso};
use crate::notice;
use crate::remote::parse::{Parsed, parse_usage};

pub(super) fn run(body: &Value) -> Result<Parsed, HarnessError> {
    parse_usage(body, NOW)
}

pub(super) fn from_fixture(name: &str) -> Value {
    serde_json::from_str(&fixture(name)).unwrap()
}

fn expected(
    id: &str,
    kind: WindowKind,
    minutes: Option<u32>,
    used: f64,
    reset: i64,
) -> UsageWindow {
    let mut window = UsageWindow::new(id, kind);
    window.window_minutes = minutes;
    window.used_pct = Some(used);
    window.resets_at = Some(reset);
    window
}

pub(super) fn rolling_and_weekly(rolling: Value) -> Value {
    json!({ "usage": { "rolling": rolling, "weekly": { "percent": 7, "resetsAt": iso(NOW + 3_600) } } })
}

pub(super) fn changed(body: &Value) -> bool {
    run(body).err().is_some_and(|e| e.code == ErrorCode::EndpointChanged && !e.retryable)
}

#[test]
fn full_body_maps_the_three_windows() {
    let parsed = run(&from_fixture("ok.json")).unwrap();
    assert_eq!(
        parsed.windows,
        [
            expected("rolling", WindowKind::FiveHour, Some(300), 12.0, 1_800_007_800),
            expected("weekly", WindowKind::Weekly, Some(10_080), 34.0, 1_800_230_400),
            // 2027-02-03 14:20Z, the January 3rd anniversary before it: 31 days.
            expected("monthly", WindowKind::Monthly, Some(44_640), 56.0, 1_801_664_400),
        ]
    );
    assert!(parsed.notices.is_empty());
}

#[test]
fn extra_keys_are_ignored() {
    let plain = run(&from_fixture("ok.json")).unwrap();
    let noisy = run(&from_fixture("extra_keys.json")).unwrap();
    assert_eq!(noisy.windows, plain.windows);
    assert!(noisy.notices.is_empty());
}

#[test]
fn rate_limited_window_reads_full_and_warns() {
    let parsed = run(&from_fixture("rate_limited.json")).unwrap();
    assert_eq!(parsed.windows[2].used_pct, Some(100.0));
    assert_eq!(
        parsed.notices,
        [notice::make(notice::RATE_LIMITED, Severity::Warn, &[("window", "monthly")])]
    );
}

#[test]
fn rate_limited_status_wins_over_a_missing_or_lower_percent() {
    for percent in [json!(null), json!(40), json!("x")] {
        let body = rolling_and_weekly(json!({ "status": "rate-limited", "percent": percent }));
        let parsed = run(&body).unwrap();
        assert_eq!(parsed.windows[0].used_pct, Some(100.0), "{percent}");
        assert_eq!(parsed.windows[0].resets_at, None);
    }
}

#[test]
fn only_the_exact_status_token_means_rate_limited() {
    for status in
        ["rate\u{2011}limited", "rate limited", " rate-limited", "rate-limited ", "ok", ""]
    {
        let body = rolling_and_weekly(json!({ "status": status, "percent": 40 }));
        let parsed = run(&body).unwrap();
        assert_eq!(parsed.windows[0].used_pct, Some(40.0), "{status:?}");
        assert!(parsed.notices.is_empty(), "{status:?}");
    }
    let body = rolling_and_weekly(json!({ "status": "Rate-Limited", "percent": 40 }));
    assert_eq!(run(&body).unwrap().windows[0].used_pct, Some(100.0));
}

#[test]
fn zero_percent_with_a_full_length_reset_is_idle() {
    let parsed = run(&from_fixture("idle.json")).unwrap();
    assert_eq!(parsed.windows[0].used_pct, Some(0.0));
    assert_eq!(
        parsed.notices,
        [notice::make(notice::WINDOW_IDLE, Severity::Info, &[("window", "rolling")])]
    );
}

#[test]
fn idle_needs_zero_percent_and_a_reset_one_window_away_within_the_clock_slack() {
    let full = 300 * 60;
    for (pct, offset, idle) in [
        (0, 0, true),
        (0, 120, true),
        (0, -120, true),
        (0, 121, false),
        (0, -121, false),
        (0, -full + 60, false),
        (1, 0, false),
    ] {
        let body =
            rolling_and_weekly(json!({ "percent": pct, "resetsAt": iso(NOW + full + offset) }));
        let flagged = run(&body).unwrap().notices.iter().any(|n| n.code == notice::WINDOW_IDLE);
        assert_eq!(flagged, idle, "pct {pct} offset {offset}");
    }
}

#[test]
fn idle_applies_to_every_window_with_a_known_length() {
    let body = json!({ "usage": {
        "weekly": { "percent": 0, "resetsAt": iso(NOW + 10_080 * 60) },
        "monthly": { "percent": 0, "resetsAt": "2027-02-15T08:00:00.000Z" },
    } });
    let windows: Vec<_> =
        run(&body).unwrap().notices.iter().map(|n| n.params["window"].clone()).collect();
    // 2027-02-15T08:00Z is 31 days after NOW and the month before it (January) has 31 days.
    assert_eq!(windows, ["weekly", "monthly"]);
}

#[test]
fn notices_list_limits_before_idle_windows() {
    let body = json!({ "usage": {
        "rolling": { "percent": 0, "resetsAt": iso(NOW + 300 * 60) },
        "weekly": { "status": "rate-limited", "resetsAt": iso(NOW + 3_600) },
    } });
    let codes: Vec<_> = run(&body).unwrap().notices.into_iter().map(|n| n.code).collect();
    assert_eq!(codes, [notice::RATE_LIMITED, notice::WINDOW_IDLE]);
}

#[test]
fn a_missing_window_is_not_emitted_not_zero() {
    let parsed = run(&from_fixture("missing_window.json")).unwrap();
    let ids: Vec<_> = parsed.windows.iter().map(|w| w.id.as_str()).collect();
    assert_eq!(ids, ["weekly", "monthly"]);
}
