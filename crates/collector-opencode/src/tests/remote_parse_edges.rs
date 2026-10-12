//! The usage body parser against hostile or drifted bodies: wrong types, out-of-range numbers,
//! implausible dates and bodies of another shape. Anything doubtful becomes unknown.

use serde_json::json;

use super::fake_db::NOW;
use super::iso;
use super::remote_parse::{changed, from_fixture, rolling_and_weekly, run};

#[test]
fn wrong_types_become_unknown_field_by_field() {
    let parsed = run(&from_fixture("wrong_types.json")).unwrap();
    let ids: Vec<_> = parsed.windows.iter().map(|w| w.id.as_str()).collect();
    // monthly has neither a usable percent nor a usable reset: nothing to show.
    assert_eq!(ids, ["rolling", "weekly"]);
    assert_eq!(parsed.windows[0].used_pct, None);
    assert_eq!(parsed.windows[0].resets_at, Some(1_800_007_800));
    assert_eq!(parsed.windows[1].used_pct, Some(34.0));
    assert_eq!(parsed.windows[1].resets_at, None);
}

#[test]
fn percent_is_floored_and_anything_out_of_range_is_unknown() {
    let cases = [
        (json!(12.9), Some(12.0)),
        (json!(0.99), Some(0.0)),
        (json!(99.999), Some(99.0)),
        (json!(100), Some(100.0)),
        (json!(0), Some(0.0)),
        (json!(-0.0), Some(0.0)),
        (json!(100.01), None),
        (json!(150), None),
        (json!(-5), None),
        (json!(-0.01), None),
        (json!(true), None),
        (json!({}), None),
    ];
    for (percent, want) in cases {
        let body = rolling_and_weekly(json!({ "percent": percent, "resetsAt": iso(NOW + 600) }));
        let got = run(&body).unwrap().windows[0].used_pct;
        assert_eq!(got, want, "{percent}");
        assert!(got.is_none_or(|v| v.is_sign_positive()), "negative zero leaked for {percent}");
    }
    let parsed = run(&from_fixture("out_of_range.json")).unwrap();
    let used: Vec<_> = parsed.windows.iter().map(|w| w.used_pct).collect();
    assert_eq!(used, [None, None, Some(12.0)]);
}

#[test]
fn a_body_with_no_usable_percent_anywhere_is_endpoint_changed() {
    let at = iso(NOW + 600);
    assert!(changed(&json!({ "usage": { "rolling": { "percent": "12", "resetsAt": at } } })));
    assert!(changed(
        &json!({ "usage": { "rolling": { "resetsAt": at }, "weekly": { "percent": 150 } } })
    ));
    assert!(changed(&json!({ "usage": { "monthly": { "usedPercent": 40, "resetAt": at } } })));
}

#[test]
fn a_body_of_the_wrong_shape_is_endpoint_changed() {
    let named = from_fixture("mismatch.json");
    let auth_error = from_fixture("error_auth.json");
    let bodies = [
        named,
        auth_error,
        json!({}),
        json!([]),
        json!(null),
        json!("ok"),
        json!(42),
        json!(true),
        json!({ "usage": null }),
        json!({ "usage": [] }),
        json!({ "usage": "x" }),
        json!({ "usage": {} }),
        json!({ "usage": { "rolling": null } }),
        json!({ "usage": { "rolling": [] } }),
        json!({ "usage": { "other": { "percent": 5 } } }),
        json!({ "Usage": { "rolling": { "percent": 5 } } }),
    ];
    for body in bodies {
        assert!(changed(&body), "{body}");
    }
}

#[test]
fn a_window_with_nothing_recognizable_is_dropped() {
    let body = rolling_and_weekly(json!({ "status": "ok", "note": "no numbers here" }));
    let ids: Vec<_> = run(&body).unwrap().windows.into_iter().map(|w| w.id).collect();
    assert_eq!(ids, ["weekly"]);
}

#[test]
fn a_reset_must_be_a_plausible_timestamp() {
    let cases = [
        (iso(NOW - 3_600), true),
        (iso(NOW + 366 * 86_400), true),
        (iso(NOW + 366 * 86_400 + 1), false),
        ("9999-12-31T23:59:59Z".to_owned(), false),
        ("1969-12-31T23:59:59Z".to_owned(), false),
        ("0000-01-01T00:00:00Z".to_owned(), false),
        ("2027-01-15".to_owned(), false),
        ("1800000000".to_owned(), false),
        (String::new(), false),
    ];
    for (text, usable) in cases {
        let body = rolling_and_weekly(json!({ "percent": 5, "resetsAt": text }));
        let got = run(&body).unwrap().windows[0].resets_at.is_some();
        assert_eq!(got, usable, "{text:?}");
    }
}

#[test]
fn fractional_seconds_and_offsets_resolve_to_the_same_instant() {
    for text in
        ["2027-01-15T10:10:00.999Z", "2027-01-15T13:10:00+03:00", "2027-01-15T07:10:00-03:00"]
    {
        let body = rolling_and_weekly(json!({ "percent": 5, "resetsAt": text }));
        assert_eq!(run(&body).unwrap().windows[0].resets_at, Some(1_800_007_800), "{text}");
    }
}

#[test]
fn monthly_length_is_unknown_without_a_reset() {
    let body = json!({ "usage": { "monthly": { "percent": 40 }, "weekly": { "percent": 1 } } });
    let parsed = run(&body).unwrap();
    assert_eq!(parsed.windows[1].id, "monthly");
    assert_eq!(parsed.windows[1].window_minutes, None);
    assert_eq!(parsed.windows[0].window_minutes, Some(10_080));
}

#[test]
fn huge_and_unicode_text_fields_do_not_disturb_the_numbers() {
    let body = rolling_and_weekly(json!({
        "status": "ok".repeat(30_000),
        "percent": 33,
        "resetsAt": "é日本🔑".repeat(100),
        "name": "\u{0000}\u{202e}",
    }));
    let parsed = run(&body).unwrap();
    assert_eq!(parsed.windows[0].used_pct, Some(33.0));
    assert_eq!(parsed.windows[0].resets_at, None);
}
