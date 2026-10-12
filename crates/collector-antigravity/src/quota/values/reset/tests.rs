use serde_json::json;

use super::*;

const NOW: i64 = 1_791_633_600; // 2026-10-10T12:00:00Z

fn reset(object: &Value) -> Option<i64> {
    reset_at(object, NOW)
}

#[test]
fn rfc3339_in_any_offset_is_the_same_instant() {
    let utc = Some(1_792_053_939);
    for text in ["2026-10-15T08:45:39Z", "2026-10-15T05:45:39-03:00", "2026-10-15T08:45:39.987Z"] {
        assert_eq!(reset(&json!({ "resetTime": text })), utc, "{text}");
    }
}

#[test]
fn epoch_seconds_and_milliseconds_as_numbers_or_strings() {
    for value in [
        json!(1_792_053_939),
        json!(1_792_053_939.7),
        json!("1792053939"),
        json!(" 1792053939 "),
        json!(1_792_053_939_000_u64),
        json!("1792053939000"),
    ] {
        assert_eq!(reset(&json!({ "resetTime": value.clone() })), Some(1_792_053_939), "{value}");
    }
}

#[test]
fn dates_outside_the_plausible_range_are_unknown() {
    for value in [
        json!(0),
        json!(-5),
        json!(946_684_799),
        json!(4_102_444_801_i64),
        json!(9_999_999_999_999_u64),
        json!("1999-12-31T23:59:59Z"),
        json!("2100-01-01T00:00:01Z"),
        json!("0000-01-01T00:00:00Z"),
        json!(1e30),
        json!(f64::MAX),
    ] {
        assert_eq!(reset(&json!({ "resetTime": value.clone() })), None, "{value}");
    }
    assert_eq!(reset(&json!({ "resetTime": "2000-01-01T00:00:00Z" })), Some(MIN_EPOCH));
    assert_eq!(reset(&json!({ "resetTime": "2100-01-01T00:00:00Z" })), Some(MAX_EPOCH));
}

#[test]
fn text_that_is_not_a_time_is_unknown() {
    let long = "1".repeat(200);
    for value in [
        json!(""),
        json!("soon"),
        json!("NaN"),
        json!("inf"),
        json!("-1"),
        json!("2026-10-15"),
        json!("2026-10-15T08:45:39"),
        json!(long),
        json!(true),
        json!(null),
        json!([1_792_053_939]),
        json!({ "seconds": 1_792_053_939 }),
    ] {
        assert_eq!(reset(&json!({ "resetTime": value.clone() })), None, "{value}");
    }
    assert_eq!(reset(&json!({})), None);
    assert_eq!(reset(&json!(null)), None);
}

#[test]
fn prose_counts_from_the_moment_of_the_response() {
    let said = |text: &str| reset(&json!({ "description": text }));
    let secs = |days: i64, hours: i64| NOW + days * 86_400 + hours * 3_600;
    assert_eq!(
        said("You have used some of your weekly limit, it will fully refresh in 5 days, 11 hours."),
        Some(secs(5, 11))
    );
    assert_eq!(said("it will fully refresh in 4 hours."), Some(secs(0, 4)));
    assert_eq!(said("Refreshes in 1 hour and 30 minutes"), Some(NOW + 5_400));
    assert_eq!(said("resets in 2 weeks"), Some(NOW + 1_209_600));
    assert_eq!(said("RESETS IN 45 MINS"), Some(NOW + 2_700));
    assert_eq!(said("Included in your plan. Resets in 3 hours."), Some(secs(0, 3)));
}

#[test]
fn prose_without_a_clear_duration_is_unknown() {
    let said = |text: &str| reset(&json!({ "description": text }));
    for text in [
        "",
        "You have used some of your weekly limit.",
        "it will fully refresh in a moment",
        "refreshes in less than a minute",
        "refreshes in 0 hours",
        "refreshes in 3 parsecs",
        "refreshes in 1.5 hours",
        "refreshes in 4h",
        "refreshes tomorrow, 4 hours",
        "refreshes in 401 days",
        "refreshes in 99999999999999999999 days",
    ] {
        assert_eq!(said(text), None, "{text}");
    }
}

#[test]
fn prose_stops_at_the_first_word_that_is_not_part_of_the_duration() {
    let said = |text: &str| reset(&json!({ "description": text }));
    assert_eq!(said("refresh in 2 hours 15 minutes left"), Some(NOW + 8_100));
    assert_eq!(said("refresh in 2 hours, then 15 minutes"), Some(NOW + 7_200));
}

#[test]
fn an_absolute_time_wins_and_prose_covers_for_a_missing_or_broken_one() {
    let bucket = |time: Value| json!({ "resetTime": time, "description": "refreshes in 2 hours" });
    assert_eq!(reset(&bucket(json!("2026-10-15T08:45:39Z"))), Some(1_792_053_939));
    assert_eq!(reset(&bucket(json!(null))), Some(NOW + 7_200));
    assert_eq!(reset(&bucket(json!("garbage"))), Some(NOW + 7_200));
    assert_eq!(reset(&json!({ "description": 5 })), None);
}

#[test]
fn a_clock_at_the_edge_does_not_overflow() {
    let object = json!({ "description": "refreshes in 2 days" });
    assert_eq!(reset_at(&object, i64::MAX), Some(i64::MAX));
}
