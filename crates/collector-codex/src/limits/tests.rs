//! Unit tests for the snapshot reader.

use serde_json::{Value, json};
use usage_model::{UsageWindow, WindowKind};

use super::{Limits, parse_snapshot};

fn windows(snapshot: &Value) -> Vec<UsageWindow> {
    parse_snapshot(snapshot).map(|limits| limits.windows).unwrap_or_default()
}

#[test]
fn app_server_spelling_maps_both_windows() {
    let limits = parse_snapshot(&json!({
        "limitId": "codex", "planType": "plus",
        "primary": {"usedPercent": 42, "windowDurationMins": 300, "resetsAt": 1_791_670_500},
        "secondary": {"usedPercent": 17, "windowDurationMins": 10080, "resetsAt": 1_792_011_600},
        "credits": {"hasCredits": true, "unlimited": false, "balance": "12.50"}
    }))
    .unwrap();
    assert_eq!(limits.plan.as_deref(), Some("Plus"));
    assert_eq!(limits.credits, Some(12.5));
    let [five, week] = limits.windows.as_slice() else { panic!("{:?}", limits.windows) };
    assert_eq!((five.id.as_str(), five.kind), ("five_hour", WindowKind::FiveHour));
    assert_eq!(
        (five.used_pct, five.window_minutes, five.resets_at),
        (Some(42.0), Some(300), Some(1_791_670_500))
    );
    assert_eq!((week.id.as_str(), week.kind), ("weekly", WindowKind::Weekly));
    assert_eq!((week.used_pct, week.window_minutes), (Some(17.0), Some(10_080)));
}

#[test]
fn rollout_spelling_maps_the_same_way() {
    let from_rollout = windows(&json!({
        "primary": {"used_percent": 12.5, "window_minutes": 300, "resets_at": 1_791_670_500},
        "secondary": {"used_percent": 3.0, "window_minutes": 10080, "resets_at": 1_792_011_600},
        "plan_type": "pro"
    }));
    assert_eq!(from_rollout.len(), 2);
    assert_eq!(from_rollout[0].used_pct, Some(12.5));
    assert_eq!(from_rollout[1].resets_at, Some(1_792_011_600));
}

#[test]
fn windows_are_named_by_their_length_not_their_slot() {
    // A plan whose only window is the weekly one reports it as `primary`.
    let only_weekly = windows(&json!({"primary": {"usedPercent": 9, "windowDurationMins": 10080}}));
    assert_eq!(only_weekly.len(), 1);
    assert_eq!((only_weekly[0].id.as_str(), only_weekly[0].kind), ("weekly", WindowKind::Weekly));

    let rounded = windows(&json!({
        "primary": {"usedPercent": 1, "windowDurationMins": 299},
        "secondary": {"usedPercent": 1, "windowDurationMins": 10079}
    }));
    assert_eq!((rounded[0].id.as_str(), rounded[0].window_minutes), ("five_hour", Some(299)));
    assert_eq!((rounded[1].id.as_str(), rounded[1].window_minutes), ("weekly", Some(10_079)));
}

#[test]
fn an_unrecognised_length_keeps_its_slot_and_stated_minutes() {
    let odd = windows(&json!({"primary": {"usedPercent": 5, "windowDurationMins": 90}}));
    assert_eq!(
        (odd[0].id.as_str(), odd[0].kind, odd[0].window_minutes),
        ("primary", WindowKind::Other, Some(90))
    );
}

#[test]
fn without_a_stated_length_the_slot_fixes_it_by_contract() {
    let bare = windows(&json!({
        "primary": {"usedPercent": 5}, "secondary": {"usedPercent": 6}
    }));
    assert_eq!((bare[0].id.as_str(), bare[0].window_minutes), ("five_hour", Some(300)));
    assert_eq!((bare[1].id.as_str(), bare[1].window_minutes), ("weekly", Some(10_080)));
}

#[test]
fn two_windows_of_one_length_keep_distinct_ids() {
    let twins = windows(&json!({
        "primary": {"usedPercent": 1, "windowDurationMins": 10080},
        "secondary": {"usedPercent": 2, "windowDurationMins": 10080}
    }));
    let ids: Vec<_> = twins.iter().map(|window| window.id.as_str()).collect();
    assert_eq!(ids, ["weekly", "secondary"]);
    assert_eq!(twins[1].kind, WindowKind::Other);
}

#[test]
fn broken_numbers_become_unknown_not_clamped() {
    let broken = windows(&json!({
        "primary": {"usedPercent": -1, "windowDurationMins": 0, "resetsAt": 0},
        "secondary": {"usedPercent": "50", "windowDurationMins": -300, "resetsAt": 1_791_670_500_000_i64}
    }));
    // Nothing usable in either window: neither is invented.
    assert!(broken.is_empty(), "{broken:?}");

    let partial = windows(&json!({
        "primary": {"usedPercent": 1e9, "windowDurationMins": 300, "resetsAt": 1_791_670_500}
    }));
    assert_eq!(partial[0].used_pct, None);
    assert_eq!(partial[0].resets_at, Some(1_791_670_500));
}

#[test]
fn over_budget_values_are_kept_as_reported() {
    let over = windows(&json!({"primary": {"usedPercent": 130, "windowDurationMins": 300}}));
    assert_eq!(over[0].used_pct, Some(130.0));
    let edge = windows(&json!({"primary": {"usedPercent": 0, "windowDurationMins": 300}}));
    assert_eq!(edge[0].used_pct, Some(0.0), "a reported 0 is a real 0");
}

#[test]
fn whole_floats_count_as_integers_and_fractions_do_not() {
    let floaty = windows(
        &json!({"primary": {"usedPercent": 1, "windowDurationMins": 300.0, "resetsAt": 1_791_670_500.0}}),
    );
    assert_eq!((floaty[0].window_minutes, floaty[0].resets_at), (Some(300), Some(1_791_670_500)));
    let fractional = windows(
        &json!({"primary": {"usedPercent": 1, "windowDurationMins": 300.5, "resetsAt": 1.5}}),
    );
    assert_eq!((fractional[0].window_minutes, fractional[0].resets_at), (Some(300), None));
}

#[test]
fn null_empty_and_foreign_windows_are_skipped() {
    for value in [json!(null), json!({}), json!("x"), json!(7), json!([1]), json!({"other": 1})] {
        let got = windows(&json!({"primary": value.clone(), "secondary": value}));
        assert!(got.is_empty());
    }
    assert_eq!(
        parse_snapshot(&json!({})),
        Some(Limits { windows: vec![], plan: None, credits: None, reset_credits: None })
    );
    for value in [json!(null), json!([]), json!("x"), json!(1)] {
        assert_eq!(parse_snapshot(&value), None);
    }
}

#[test]
fn credits_need_a_finite_balance_on_an_account_that_has_credits() {
    let balance = |credits: Value| parse_snapshot(&json!({"credits": credits})).unwrap().credits;
    assert_eq!(balance(json!({"hasCredits": true, "unlimited": false, "balance": "0"})), Some(0.0));
    assert_eq!(balance(json!({"has_credits": true, "balance": 7})), Some(7.0));
    assert_eq!(balance(json!({"hasCredits": false, "balance": "5"})), None);
    assert_eq!(balance(json!({"hasCredits": true, "unlimited": true, "balance": "5"})), None);
    // Strings are where NaN and infinity can sneak in: JSON numbers cannot spell them.
    let hostile = ["abc", "-1", "NaN", "nan", "inf", "-inf", "Infinity", "1e99", "1e309", ""];
    for bad in hostile.map(|text| json!(text)).into_iter().chain([json!(null), json!([])]) {
        assert_eq!(balance(json!({"hasCredits": true, "balance": bad})), None, "{bad}");
    }
    assert_eq!(balance(json!("nope")), None);
}

#[test]
fn plan_reads_either_spelling_and_hides_unknown() {
    let plan = |snapshot: Value| parse_snapshot(&snapshot).unwrap().plan;
    assert_eq!(plan(json!({"planType": "pro"})).as_deref(), Some("Pro"));
    assert_eq!(plan(json!({"plan_type": "team"})).as_deref(), Some("Team"));
    assert_eq!(plan(json!({"planType": "unknown"})), None);
    assert_eq!(plan(json!({"planType": 5})), None);
}

#[test]
fn swapped_slots_are_still_named_by_their_length() {
    let swapped = windows(&json!({
        "primary": {"usedPercent": 30, "windowDurationMins": 10080, "resetsAt": 1_792_011_600},
        "secondary": {"usedPercent": 8, "windowDurationMins": 300, "resetsAt": 1_791_670_500}
    }));
    let by_id = |id: &str| swapped.iter().find(|window| window.id == id).map(|w| w.used_pct);
    assert_eq!(by_id("weekly"), Some(Some(30.0)));
    assert_eq!(by_id("five_hour"), Some(Some(8.0)));
    assert_eq!(swapped.len(), 2);
}

#[test]
fn a_weekly_only_secondary_without_length_is_not_taken_for_a_session() {
    // Only `secondary` present and unlabelled: it keeps the weekly contract, no 5 h is invented.
    let only = windows(&json!({"secondary": {"usedPercent": 4}}));
    assert_eq!(only.len(), 1);
    assert_eq!((only[0].id.as_str(), only[0].kind), ("weekly", WindowKind::Weekly));
}

#[test]
fn unknown_extra_windows_are_ignored_not_promoted() {
    let got = windows(&json!({
        "primary": {"usedPercent": 1, "windowDurationMins": 10080},
        "tertiary": {"usedPercent": 99, "windowDurationMins": 300},
        "additional": [{"usedPercent": 98, "windowDurationMins": 300}]
    }));
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].id, "weekly");
}

#[test]
fn reset_credit_count_reads_only_a_whole_non_negative_available_count() {
    let count = |answer: Value| super::reset_credit_count(&answer);
    let with = |value: Value| json!({"rateLimitResetCredits": value});
    assert_eq!(count(with(json!({"availableCount": 3, "credits": null}))), Some(3));
    assert_eq!(count(with(json!({"availableCount": 2.0}))), Some(2));
    assert_eq!(count(with(json!({"availableCount": 0}))), Some(0), "a known zero is not unknown");
    for unknown in [
        json!({}),
        with(json!(null)),
        with(json!({})),
        with(json!({"availableCount": null})),
        with(json!({"availableCount": -1})),
        with(json!({"availableCount": 1.5})),
        with(json!({"availableCount": "3"})),
        with(json!({"availableCount": 1_000_001})),
        with(json!([3])),
        with(json!(3)),
        json!(null),
    ] {
        assert_eq!(count(unknown.clone()), None, "{unknown}");
    }
}

#[test]
fn reset_credit_details_are_never_read() {
    let answer = json!({"rateLimitResetCredits": {
        "availableCount": 1,
        "credits": [{"id": "opaque-fixture-id", "title": "fixture", "status": "available"}]
    }});
    assert_eq!(super::reset_credit_count(&answer), Some(1));
}
