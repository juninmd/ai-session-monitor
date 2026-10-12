use serde_json::json;

use super::*;

fn ids(windows: &[UsageWindow]) -> Vec<&str> {
    windows.iter().map(|w| w.id.as_str()).collect()
}

#[test]
fn scoped_limits_become_per_model_weekly_windows() {
    let body = json!({
        "five_hour": {"utilization": 29.0, "resets_at": "2026-10-11T12:00:00+00:00"},
        "seven_day": {"utilization": 26.0, "resets_at": "2026-10-14T12:00:00+00:00"},
        "limits": [
            {"kind": "session", "group": "session", "percent": 29, "resets_at": "2026-10-11T12:00:00+00:00", "scope": null, "is_active": false},
            {"kind": "weekly_all", "group": "weekly", "percent": 26, "resets_at": "2026-10-14T12:00:00+00:00", "scope": null, "is_active": false},
            {"kind": "weekly_scoped", "percent": 43, "resets_at": "2026-10-14T12:00:00.570750+00:00", "scope": {"model": {"id": null, "display_name": "Fable"}}, "is_active": true},
            {"kind": "weekly_scoped", "percent": 12.5, "resets_at": "2026-10-14T12:00:00Z", "scope": {"model": {"display_name": "Claude Opus 4.1"}}},
            {"kind": "weekly_scoped", "percent": 7, "resets_at": null, "scope": {"model": {"display_name": "Sonnet"}}}
        ]
    });
    let windows = parse(&body).unwrap();
    assert_eq!(ids(&windows), ["fable:weekly", "opus:weekly", "sonnet:weekly"]);
    let fable = &windows[0];
    assert_eq!(
        (fable.kind, fable.group.as_deref(), fable.label.as_deref()),
        (WindowKind::Weekly, Some("fable"), Some("Fable"))
    );
    assert_eq!((fable.used_pct, fable.window_minutes), (Some(43.0), Some(10_080)));
    assert_eq!(fable.resets_at, Some(1_791_979_200), "2026-10-14T12:00:00Z, fraction dropped");
    let opus = &windows[1];
    assert_eq!(
        (opus.group.as_deref(), opus.label.as_deref(), opus.used_pct),
        (Some("opus"), None, Some(12.5))
    );
    assert_eq!(windows[2].resets_at, None, "a null reset stays unknown");
}

#[test]
fn the_flat_keys_still_work_and_null_means_no_such_cap() {
    let body = json!({
        "five_hour": {"utilization": 1, "resets_at": null},
        "seven_day_opus": {"utilization": 31.0, "resets_at": "2026-10-14T12:00:00+00:00"},
        "seven_day_sonnet": null
    });
    let windows = parse(&body).unwrap();
    assert_eq!(ids(&windows), ["opus:weekly"]);
    assert_eq!(windows[0].used_pct, Some(31.0));
    assert_eq!(windows[0].group.as_deref(), Some("opus"));
}

#[test]
fn limits_win_over_flat_keys_and_duplicates_keep_the_binding_cap() {
    let body = json!({
        "seven_day_opus": {"utilization": 99.0, "resets_at": null},
        "limits": [
            {"kind": "weekly_scoped", "percent": 10, "scope": {"model": {"display_name": "Opus"}}},
            {"kind": "weekly_scoped", "percent": 30, "scope": {"model": {"display_name": "Opus 4"}}},
            {"kind": "weekly_scoped", "percent": 20, "scope": {"model": {"display_name": "Opus 3"}}}
        ]
    });
    let windows = parse(&body).unwrap();
    assert_eq!(windows.len(), 1);
    assert_eq!(windows[0].used_pct, Some(30.0));
}

#[test]
fn an_account_without_model_caps_has_no_extras() {
    let body = json!({"five_hour": {"utilization": 3}, "seven_day": {"utilization": 4}, "seven_day_opus": null, "limits": []});
    assert_eq!(parse(&body).unwrap(), Vec::new());
}

#[test]
fn entries_with_a_missing_name_kind_or_bad_numbers_are_skipped_or_unknown() {
    let body = json!({"limits": [
        {"kind": "weekly_scoped", "percent": 10, "scope": null},
        {"kind": "weekly_scoped", "percent": 10},
        {"kind": "weekly_scoped", "percent": 10, "scope": {"model": {"display_name": "  \u{0}  "}}},
        {"kind": "weekly_scoped", "percent": 10, "scope": {"model": {"display_name": 7}}},
        {"kind": "something_new", "percent": 10, "scope": {"model": {"display_name": "X"}}},
        "not an object",
        {"kind": "weekly_scoped", "percent": 140, "resets_at": "tomorrow", "scope": {"model": {"display_name": "Haiku"}}},
        {"kind": "weekly_scoped", "percent": -3, "scope": {"model": {"display_name": "Mini"}}}
    ]});
    let windows = parse(&body).unwrap();
    assert_eq!(ids(&windows), ["haiku:weekly", "mini:weekly"]);
    assert!(
        windows.iter().all(|w| w.used_pct.is_none() && w.resets_at.is_none()),
        "out of range is unknown"
    );
}

#[test]
fn display_names_are_cleaned_and_bounded() {
    let long = "ç".repeat(200);
    let body = json!({"limits": [
        {"kind": "weekly_scoped", "percent": 1, "scope": {"model": {"display_name": format!("Mod\u{7}el\n{long}")}}}
    ]});
    let window = parse(&body).unwrap().remove(0);
    let label = window.label.unwrap();
    assert!(label.chars().count() <= 40 && !label.chars().any(char::is_control), "{label:?}");
    assert_eq!(window.id, "model:weekly", "only the ASCII part names the id");
}

#[test]
fn the_number_of_extras_is_bounded() {
    let limits: Vec<Value> = (0..100)
        .map(|n| json!({"kind": "weekly_scoped", "percent": 1, "scope": {"model": {"display_name": format!("Model{n}")}}}))
        .collect();
    let windows = parse(&json!({"limits": limits})).unwrap();
    assert_eq!(windows.len(), 8);
}

#[test]
fn a_document_without_any_known_key_is_a_changed_endpoint() {
    for body in
        [json!({}), json!({"hello": "world"}), json!([1, 2]), json!("x"), json!(null), json!(7)]
    {
        assert_eq!(
            parse(&body).err().map(|e| (e.code, e.retryable)),
            Some((ErrorCode::EndpointChanged, false)),
            "{body}"
        );
    }
}

#[test]
fn bounded_cleans_what_came_from_a_cache_file() {
    let mut ok = UsageWindow::new("opus:weekly", WindowKind::Weekly);
    ok.group = Some("opus".into());
    ok.used_pct = Some(10.0);
    ok.resets_at = Some(1_800_000_000);
    ok.window_minutes = Some(10_080);
    let mut bad = UsageWindow::new("x\u{0}y", WindowKind::Weekly);
    bad.used_pct = Some(500.0);
    bad.resets_at = Some(5);
    bad.window_minutes = Some(u32::MAX);
    bad.label = Some("l".repeat(500));
    let mut windows = vec![ok.clone(), bad, UsageWindow::new("\u{0}", WindowKind::Weekly)];
    windows.extend((0..20).map(|n| UsageWindow::new(format!("w{n}"), WindowKind::Weekly)));
    let cleaned = bounded(windows);
    assert_eq!(cleaned.len(), 8, "capped; the unnamed one is dropped");
    assert_eq!(cleaned[0], ok);
    assert_eq!(cleaned[1].id, "xy");
    assert_eq!(
        (cleaned[1].used_pct, cleaned[1].resets_at, cleaned[1].window_minutes),
        (None, None, None)
    );
    assert_eq!(cleaned[1].label.as_deref().map(str::len), Some(40));
}
