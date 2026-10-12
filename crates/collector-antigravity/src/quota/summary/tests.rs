use serde_json::{Value, json};
use usage_model::ErrorCode;

use super::*;
use crate::fixtures::{NOW, json as fixture};
use crate::quota::Cadence;

fn read(value: &Value) -> Read {
    parse_summary(value, NOW).expect("summary parses")
}

fn only(read: &Read, pool: &Pool, cadence: Cadence) -> PoolWindow {
    let mut found = read.windows.iter().filter(|w| &w.pool == pool && w.cadence == Some(cadence));
    let window = found.next().expect("window present").clone();
    assert!(found.next().is_none(), "window listed twice");
    window
}

fn fields(window: &PoolWindow) -> (Option<f64>, Option<i64>) {
    (window.used_pct, window.resets_at)
}

#[test]
fn a_full_summary_has_both_windows_of_both_pools() {
    let read = read(&fixture("summary_full"));
    assert_eq!(read.plan, None);
    assert_eq!(read.windows.len(), 4);
    let gemini = |cadence| fields(&only(&read, &Pool::Gemini, cadence));
    let others = |cadence| fields(&only(&read, &Pool::ClaudeGpt, cadence));
    // The absolute `resetTime` wins over the prose that says "in 2 hours".
    assert_eq!(gemini(Cadence::FiveHour), (Some(54.0), Some(1_791_641_520)));
    assert_eq!(gemini(Cadence::Weekly), (Some(29.0), Some(1_792_053_939)));
    assert_eq!(others(Cadence::FiveHour), (Some(39.0), Some(1_791_649_800)));
    // This bucket writes its share in the oneof form.
    assert_eq!(others(Cadence::Weekly), (Some(12.0), Some(1_792_111_194)));
    assert!(read.windows.iter().all(|w| !w.inferred && w.label.is_none() && w.slug.is_none()));
}

#[test]
fn the_cadence_can_come_from_the_window_field_alone() {
    let read = read(&fixture("summary_hub"));
    assert_eq!(read.windows.len(), 4);
    let used = |pool, cadence| only(&read, &pool, cadence).used_pct;
    assert_eq!(used(Pool::Gemini, Cadence::FiveHour), Some(51.0));
    assert_eq!(used(Pool::Gemini, Cadence::Weekly), Some(10.0));
    assert_eq!(used(Pool::ClaudeGpt, Cadence::FiveHour), Some(75.0));
    assert_eq!(used(Pool::ClaudeGpt, Cadence::Weekly), Some(40.0));
}

#[test]
fn a_weekly_only_plan_lists_only_weekly_windows() {
    let read = read(&fixture("summary_free"));
    assert_eq!(read.windows.len(), 2);
    assert_eq!(
        fields(&only(&read, &Pool::Gemini, Cadence::Weekly)),
        (Some(65.0), Some(1_792_195_200))
    );
    // A pool at its full share is 0 % used, which is a value, not an unknown.
    assert_eq!(only(&read, &Pool::ClaudeGpt, Cadence::Weekly).used_pct, Some(0.0));
}

#[test]
fn prose_fills_in_a_missing_reset_and_a_missing_share_stays_unknown() {
    let read = read(&fixture("summary_prose"));
    // The disabled Claude 5 hour bucket is not listed even though it carries a share.
    assert_eq!(read.windows.len(), 3);
    let five = only(&read, &Pool::Gemini, Cadence::FiveHour);
    assert_eq!(fields(&five), (Some(50.0), Some(NOW + 3 * 3_600 + 1_800)));
    let weekly = only(&read, &Pool::Gemini, Cadence::Weekly);
    assert_eq!(fields(&weekly), (None, Some(NOW + 5 * 86_400 + 11 * 3_600)));
    let claude = only(&read, &Pool::ClaudeGpt, Cadence::Weekly);
    assert_eq!(fields(&claude), (Some(60.0), Some(1_792_111_194)));
}

#[test]
fn the_groups_may_sit_under_response_summary_or_the_root() {
    let groups = fixture("summary_hub")["response"]["groups"].clone();
    let expected = read(&json!({ "response": { "groups": groups.clone() } }));
    for body in [json!({ "summary": { "groups": groups.clone() } }), json!({ "groups": groups })] {
        assert_eq!(read(&body), expected);
    }
}

#[test]
fn a_group_is_a_pool_by_its_name_then_by_the_models_it_lists() {
    let group = |name: Value, description: &str| {
        json!({ "response": { "groups": [{
            "displayName": name,
            "description": description,
            "buckets": [{ "window": "5h", "remainingFraction": 0.5 }],
        }] } })
    };
    let pool_of = |body: Value| read(&body).windows[0].pool.clone();
    assert_eq!(pool_of(group(json!("Gemini Models"), "")), Pool::Gemini);
    assert_eq!(
        pool_of(group(json!("Models"), "Models within this group: Gemini Flash")),
        Pool::Gemini
    );
    assert_eq!(
        pool_of(group(json!(null), "Models within this group: Claude Opus, GPT-OSS")),
        Pool::ClaudeGpt
    );
    // Both families in the name say nothing; the description decides.
    assert_eq!(
        pool_of(group(json!("Claude or Gemini"), "Claude Sonnet, GPT-OSS")),
        Pool::ClaudeGpt
    );
    assert_eq!(pool_of(group(json!("Image Models"), "")), Pool::Other("image_models".into()));
    assert_eq!(pool_of(group(json!(null), "")), Pool::Other("group_0".into()));
}

#[test]
fn an_unnamed_cadence_keeps_the_provider_name_and_an_id_of_its_own() {
    let body = json!({ "response": { "groups": [{ "displayName": "Gemini Models", "buckets": [
        { "bucketId": "gemini-monthly", "displayName": "Monthly Limit", "remainingFraction": 0.2 },
        { "displayName": "Burst", "resetTime": "2026-10-10T13:00:00Z" },
        { "remainingFraction": 0.9 },
    ] }] } });
    let read = read(&body);
    let named: Vec<(Option<Cadence>, Option<&str>, Option<&str>)> =
        read.windows.iter().map(|w| (w.cadence, w.label.as_deref(), w.slug.as_deref())).collect();
    assert_eq!(
        named,
        [
            (None, Some("Monthly Limit"), Some("gemini_monthly")),
            (None, Some("Burst"), Some("burst")),
            (None, None, Some("bucket_2")),
        ]
    );
}

#[test]
fn buckets_with_nothing_to_say_are_not_windows() {
    let body = json!({ "response": { "groups": [{ "displayName": "Gemini Models", "buckets": [
        {},
        { "bucketId": "x", "displayName": "Something" },
        { "bucketId": "gemini-5h" },
        "not an object",
        null,
    ] }] } });
    let read = read(&body);
    // Only the bucket that names its cadence survives, as an unknown window.
    assert_eq!(read.windows.len(), 1);
    assert_eq!(fields(&only(&read, &Pool::Gemini, Cadence::FiveHour)), (None, None));
}

#[test]
fn unusable_shares_stay_unknown_whatever_else_the_bucket_says() {
    for share in [json!(-0.5), json!(1.5), json!("NaN"), json!(null), json!([])] {
        let body = json!({ "response": { "groups": [{ "displayName": "Gemini Models", "buckets": [
            { "window": "5h", "remainingFraction": share.clone(), "resetTime": "2026-10-10T14:12:00Z" },
        ] }] } });
        let read = read(&body);
        assert_eq!(fields(&read.windows[0]), (None, Some(1_791_641_520)), "{share}");
    }
}

#[test]
fn a_body_of_another_shape_is_a_changed_endpoint() {
    let bodies = [
        json!({}),
        json!([]),
        json!(null),
        json!("quota"),
        json!({ "response": {} }),
        json!({ "response": { "groups": [] } }),
        json!({ "response": { "groups": "x" } }),
        json!({ "response": { "groups": [{ "displayName": "Gemini Models" }] } }),
        json!({ "response": { "groups": [{ "buckets": [] }] } }),
        json!({ "response": { "groups": [{ "buckets": [{ "disabled": true, "window": "5h" }] }] } }),
        json!({ "userStatus": { "planStatus": {} } }),
    ];
    for body in bodies {
        let error = parse_summary(&body, NOW).expect_err("must not parse");
        assert_eq!(error.code, ErrorCode::EndpointChanged, "{body}");
        assert!(!error.retryable);
    }
}

#[test]
fn a_status_code_in_the_body_is_reported_before_any_data() {
    let mut body = fixture("summary_hub");
    body["code"] = json!(16);
    assert_eq!(parse_summary(&body, NOW).unwrap_err().code, ErrorCode::Unauthorized);
    body["code"] = json!(0);
    assert_eq!(parse_summary(&body, NOW).expect("success code").windows.len(), 4);
}

#[test]
fn groups_and_buckets_are_bounded() {
    let bucket = json!({ "window": "5h", "remainingFraction": 0.5 });
    let group = json!({ "displayName": "Gemini Models", "buckets": vec![bucket.clone(); 100] });
    let one_group = json!({ "response": { "groups": [group.clone()] } });
    assert_eq!(read(&one_group).windows.len(), MAX_BUCKETS);
    let many = json!({ "response": { "groups": vec![group; 100] } });
    assert_eq!(read(&many).windows.len(), MAX_GROUPS * MAX_BUCKETS);
}

#[test]
fn hostile_names_are_cut_and_stripped() {
    let name = format!("Limit\u{202e}{}", "x".repeat(500));
    let body = json!({ "response": { "groups": [{ "displayName": "Gemini Models", "buckets": [
        { "bucketId": "b", "displayName": name, "remainingFraction": 0.5 },
    ] }] } });
    let read = read(&body);
    let label = read.windows[0].label.as_deref().expect("label");
    assert_eq!(label.chars().count(), MAX_NAME);
    assert!(label.starts_with("Limitxxx") && !label.contains('\u{202e}'));
    assert!(read.windows[0].slug.as_deref().is_some_and(|slug| slug.len() <= MAX_SLUG));
}
