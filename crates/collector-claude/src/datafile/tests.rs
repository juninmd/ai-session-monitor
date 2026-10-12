use serde_json::json;

use super::*;

const OK_BOTH: &str = include_str!("../../tests/fixtures/rate_limits/ok_both.json");
const OK_SPEND: &str = include_str!("../../tests/fixtures/rate_limits/ok_spend.json");
const NULL_LIMITS: &str = include_str!("../../tests/fixtures/rate_limits/null_limits.json");

fn doc(rate_limits: &Value) -> Vec<u8> {
    json!({"v": 1, "capturedAt": 1_800_000_000, "rateLimits": rate_limits}).to_string().into_bytes()
}

#[test]
fn both_windows_map_to_the_documented_ids_and_lengths() {
    let file = parse(OK_BOTH.as_bytes()).unwrap();
    assert_eq!(file.captured_at, 1_800_000_000);
    let windows = file.windows.unwrap();
    assert_eq!(windows.len(), 2);
    assert_eq!((windows[0].id.as_str(), windows[0].kind), ("five_hour", WindowKind::FiveHour));
    assert_eq!(windows[0].window_minutes, Some(300));
    assert_eq!((windows[0].used_pct, windows[0].resets_at), (Some(23.5), Some(1_800_009_000)));
    assert_eq!((windows[1].id.as_str(), windows[1].kind), ("weekly", WindowKind::Weekly));
    assert_eq!(windows[1].window_minutes, Some(10_080));
    assert_eq!((windows[1].used_pct, windows[1].resets_at), (Some(41.2), Some(1_800_400_000)));
    assert!(windows.iter().all(|w| w.group.is_none() && w.label.is_none() && w.amount.is_none()));
}

#[test]
fn a_null_or_missing_or_empty_rate_limits_is_none() {
    assert_eq!(parse(NULL_LIMITS.as_bytes()).unwrap().windows, None);
    assert_eq!(parse(&doc(&json!({}))).unwrap().windows, None);
    let missing = br#"{"v":1,"capturedAt":1800000000}"#;
    assert_eq!(parse(missing).unwrap().windows, None);
}

#[test]
fn a_window_dropped_after_its_reset_is_simply_absent() {
    let only_weekly =
        doc(&json!({"seven_day": {"used_percentage": 10, "resets_at": 1_800_400_000}}));
    let windows = parse(&only_weekly).unwrap().windows.unwrap();
    assert_eq!(windows.iter().map(|w| w.id.as_str()).collect::<Vec<_>>(), ["weekly"]);
}

#[test]
fn spend_limit_becomes_a_budget_window_with_dollars() {
    let windows = parse(OK_SPEND.as_bytes()).unwrap().windows.unwrap();
    let spend = windows.iter().find(|w| w.id == "spend_limit").unwrap();
    assert_eq!(spend.kind, WindowKind::Budget);
    assert_eq!(spend.used_pct, Some(62.8));
    assert_eq!(spend.window_minutes, None, "a month has no fixed length");
    let amount = spend.amount.as_ref().unwrap();
    assert_eq!((amount.used, amount.limit, amount.unit), (314.12, Some(500.0), AmountUnit::Usd));
}

#[test]
fn spend_periods_and_optional_dollar_fields() {
    let with = |extra: Value| {
        let mut spend = json!({"used_percentage": 120.0, "resets_at": 1_800_100_000});
        spend.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
        parse(&doc(&json!({"spend_limit": spend}))).unwrap().windows.unwrap().remove(0)
    };
    assert_eq!(with(json!({"period": "daily"})).window_minutes, Some(1_440));
    assert_eq!(with(json!({"period": "weekly"})).window_minutes, Some(10_080));
    assert_eq!(with(json!({"period": "yearly"})).window_minutes, None);
    let over = with(json!({}));
    assert_eq!(over.used_pct, Some(120.0), "a spend limit may be exceeded");
    assert_eq!(over.amount, None, "dollars are optional");
    assert_eq!(with(json!({"used_usd": 5, "limit_usd": 0})).amount, None);
    assert_eq!(with(json!({"used_usd": -5, "limit_usd": 10})).amount, None);
    assert_eq!(with(json!({"used_usd": 5})).amount, None, "no cap given is not 'no cap'");
}

#[test]
fn out_of_range_numbers_are_unknown_not_clamped() {
    for bad in [json!(-1), json!(100.5), json!("50"), json!(null), json!(1e300)] {
        let limits = json!({"five_hour": {"used_percentage": bad, "resets_at": "soon"}});
        let w = parse(&doc(&limits)).unwrap().windows.unwrap().remove(0);
        assert_eq!((w.used_pct, w.resets_at), (None, None), "{bad}");
        assert_eq!(w.window_minutes, Some(300), "the window itself is still listed");
    }
}

#[test]
fn non_object_windows_and_unknown_keys_do_not_count() {
    assert_eq!(
        parse(&doc(&json!({"five_hour": 5, "seven_day": [], "monthly": {"used_percentage": 1}})))
            .unwrap_err()
            .code,
        ErrorCode::EndpointChanged
    );
    let windows = parse(&doc(&json!({"monthly": {}, "five_hour": {"used_percentage": 1}})))
        .unwrap()
        .windows
        .unwrap();
    assert_eq!(windows.len(), 1);
}

#[test]
fn a_shape_that_is_not_ours_is_endpoint_changed() {
    let cases: [&[u8]; 7] = [
        br#"{"v":2,"capturedAt":1800000000,"rateLimits":null}"#,
        br#"{"capturedAt":1800000000,"rateLimits":null}"#,
        br#"{"v":1,"rateLimits":null}"#,
        br#"{"v":1,"capturedAt":"yesterday","rateLimits":null}"#,
        br#"{"v":1,"capturedAt":12,"rateLimits":null}"#,
        br#"{"v":1,"capturedAt":1800000000,"rateLimits":[1]}"#,
        b"[1,2,3]",
    ];
    for case in cases {
        let error = parse(case).unwrap_err();
        assert_eq!((error.code, error.retryable), (ErrorCode::EndpointChanged, false), "{case:?}");
    }
}

#[test]
fn text_that_is_not_json_is_a_retryable_bad_response() {
    for case in [&b""[..], b"not json", b"{\"v\":1,", b"\xff\xfe\x00"] {
        let error = parse(case).unwrap_err();
        assert_eq!((error.code, error.retryable), (ErrorCode::BadResponse, true), "{case:?}");
    }
}

#[test]
fn a_utf8_bom_and_unknown_fields_are_tolerated() {
    let mut bytes = b"\xEF\xBB\xBF".to_vec();
    bytes.extend_from_slice(
        json!({"v": 1, "capturedAt": 1_800_000_000, "extra": "ção ✓", "rateLimits": {"five_hour": {"used_percentage": 1, "tier": "ç"}}})
            .to_string()
            .as_bytes(),
    );
    assert!(parse(&bytes).unwrap().windows.is_some());
}

#[test]
fn read_distinguishes_missing_oversized_and_unreadable() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(read(&dir.path().join("absent.json")).unwrap(), None);

    let big = dir.path().join("big.json");
    std::fs::write(&big, vec![b' '; MAX_FILE_BYTES + 1]).unwrap();
    assert_eq!(read(&big).unwrap_err().code, ErrorCode::ResponseTooLarge);

    // A directory in place of the file is an I/O error, retried on the next tick.
    let error = read(dir.path()).unwrap_err();
    assert_eq!((error.code, error.retryable), (ErrorCode::Internal, true));

    let ok = dir.path().join("ok.json");
    std::fs::write(&ok, OK_BOTH).unwrap();
    assert_eq!(read(&ok).unwrap().unwrap().captured_at, 1_800_000_000);
}
