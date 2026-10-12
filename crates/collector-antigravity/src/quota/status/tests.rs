use serde_json::{Value, json};
use usage_model::ErrorCode;

use super::*;
use crate::fixtures::{NOW, json as fixture};

fn read(value: &Value) -> Read {
    parse_status(value, NOW).expect("status parses")
}

fn status_of(configs: Value) -> Value {
    json!({ "userStatus": { "cascadeModelConfigData": { "clientModelConfigs": configs } } })
}

fn gemini(quota: Value) -> Value {
    status_of(json!([{ "label": "Gemini 3 Pro", "quotaInfo": quota }]))
}

fn window_of(read: &Read, pool: &Pool) -> PoolWindow {
    let mut found = read.windows.iter().filter(|window| &window.pool == pool);
    let window = found.next().expect("pool present").clone();
    assert!(found.next().is_none(), "pool listed twice");
    window
}

#[test]
fn an_older_ide_reports_one_five_hour_window_per_pool() {
    let read = read(&fixture("status_ide"));
    assert_eq!(read.plan.as_deref(), Some("Google AI Pro"));
    assert_eq!(read.windows.len(), 2);
    // Two Gemini Pro modes at 10 % and 30 % used: the pool is as spent as its hungriest model.
    let gemini = window_of(&read, &Pool::Gemini);
    assert_eq!((gemini.used_pct, gemini.resets_at), (Some(30.0), Some(1_791_641_520)));
    let others = window_of(&read, &Pool::ClaudeGpt);
    assert_eq!((others.used_pct, others.resets_at), (Some(50.0), Some(1_791_649_800)));
    for window in &read.windows {
        // The length is a guess from the reset time, and says so.
        assert_eq!((window.cadence, window.inferred), (Some(Cadence::FiveHour), true));
        assert_eq!((&window.label, &window.slug), (&None, &None));
    }
}

#[test]
fn nothing_personal_is_carried_over() {
    let read = read(&fixture("status_ide"));
    let shown = format!("{read:?}");
    for private in ["fixture@example.invalid", "Fixture User", "fixture-pro-tier"] {
        assert!(!shown.contains(private), "{private} leaked");
    }
}

#[test]
fn a_newer_hub_reports_the_plan_and_no_quota() {
    let read = read(&fixture("status_hub"));
    assert_eq!(read.plan.as_deref(), Some("Pro"));
    assert!(read.windows.is_empty());
}

#[test]
fn a_weekly_plan_shows_in_a_reset_days_away() {
    let read = read(&fixture("status_free"));
    assert_eq!(read.plan.as_deref(), Some("Free"));
    let window = window_of(&read, &Pool::Gemini);
    assert_eq!(window.used_pct, Some(20.0));
    assert_eq!((window.cadence, window.inferred), (Some(Cadence::Weekly), true));
    assert_eq!(window.resets_at, Some(NOW + 3 * 86_400));
}

#[test]
fn the_cadence_follows_how_far_away_the_reset_is() {
    let at = |ahead: i64| {
        let body = gemini(json!({ "remainingFraction": 0.5, "resetTime": NOW + ahead }));
        let window = window_of(&read(&body), &Pool::Gemini);
        (window.cadence, window.slug.is_some())
    };
    assert_eq!(at(-4_000), (Some(Cadence::FiveHour), false));
    assert_eq!(at(60), (Some(Cadence::FiveHour), false));
    assert_eq!(at(FIVE_HOUR_REACH), (Some(Cadence::FiveHour), false));
    assert_eq!(at(FIVE_HOUR_REACH + 1), (Some(Cadence::Weekly), false));
    assert_eq!(at(WEEKLY_REACH), (Some(Cadence::Weekly), false));
    // Further than any plan resets: the window keeps its model's name instead of a guess.
    assert_eq!(at(WEEKLY_REACH + 1), (None, true));
}

#[test]
fn without_a_reset_the_window_is_named_by_its_model() {
    let window = window_of(&read(&gemini(json!({ "remainingFraction": 0.5 }))), &Pool::Gemini);
    assert_eq!(window.cadence, None);
    assert_eq!(
        (window.label.as_deref(), window.slug.as_deref()),
        (Some("Gemini 3 Pro"), Some("quota"))
    );
    assert_eq!(window.used_pct, Some(50.0));
}

#[test]
fn a_model_without_a_share_is_unknown_but_keeps_its_reset() {
    let reset = "2026-10-10T14:12:00Z";
    for quota in
        [json!({ "resetTime": reset }), json!({ "remainingFraction": 7, "resetTime": reset })]
    {
        let window = window_of(&read(&gemini(quota)), &Pool::Gemini);
        assert_eq!((window.used_pct, window.resets_at), (None, Some(1_791_641_520)));
    }
    // An empty object is what proto3 writes for "nothing set": unknown, not 100 % used.
    let empty = window_of(&read(&gemini(json!({}))), &Pool::Gemini);
    assert_eq!((empty.used_pct, empty.resets_at), (None, None));
}

#[test]
fn a_known_share_leads_and_the_others_lend_their_reset() {
    let body = status_of(json!([
        { "label": "Gemini 3 Pro (High)", "quotaInfo": { "resetTime": "2026-10-10T14:12:00Z" } },
        { "label": "Gemini 3 Pro (Low)", "quotaInfo": { "remainingFraction": 0.6 } },
        { "label": "Gemini 3 Flash", "quotaInfo": { "remainingFraction": 0.8, "resetTime": "2026-10-10T15:00:00Z" } },
    ]));
    let window = window_of(&read(&body), &Pool::Gemini);
    // Pro (Low) is the most used model but has no reset: the earliest reset of the pool stands in.
    assert_eq!((window.used_pct, window.resets_at), (Some(40.0), Some(1_791_641_520)));
    // The lead's own reset wins when it has one.
    let body = status_of(json!([
        { "label": "Gemini 3 Pro", "quotaInfo": { "remainingFraction": 0.3, "resetTime": "2026-10-10T15:00:00Z" } },
        { "label": "Gemini 3 Flash", "quotaInfo": { "remainingFraction": 0.9, "resetTime": "2026-10-10T14:12:00Z" } },
    ]));
    assert_eq!(window_of(&read(&body), &Pool::Gemini).resets_at, Some(1_791_644_400));
}

#[test]
fn the_model_id_decides_the_pool_before_the_label() {
    let body = status_of(json!([
        { "label": "Gemini lookalike", "modelOrAlias": { "model": "MODEL_CLAUDE_4_5_SONNET" },
          "quotaInfo": { "remainingFraction": 0.5 } },
        { "label": "GPT-OSS 120B (Medium)", "quotaInfo": { "remainingFraction": 0.4 } },
        { "label": "Image Generator", "modelOrAlias": { "model": "MODEL_IMAGEN" },
          "quotaInfo": { "remainingFraction": 0.1 } },
        { "label": "Gemini 3 Flash" },
        "not a config",
    ]));
    let read = read(&body);
    assert_eq!(read.windows.len(), 1);
    assert_eq!(window_of(&read, &Pool::ClaudeGpt).used_pct, Some(60.0));
}

#[test]
fn the_plan_is_the_tier_then_the_first_named_plan_field() {
    let plan = |status: Value| parse_status(&json!({ "userStatus": status }), NOW).unwrap().plan;
    let info = |fields: Value| json!({ "planStatus": { "planInfo": fields } });
    assert_eq!(plan(info(json!({ "planName": "Pro" }))).as_deref(), Some("Pro"));
    assert_eq!(
        plan(info(json!({ "planDisplayName": "Google AI Pro", "planName": "Pro" }))).as_deref(),
        Some("Pro")
    );
    assert_eq!(
        plan(info(json!({ "productName": "Antigravity", "planShortName": "AG" }))).as_deref(),
        Some("Antigravity")
    );
    let tiered = json!({ "userTier": { "name": "Google AI Ultra" }, "planStatus": { "planInfo": { "planName": "Pro" } } });
    assert_eq!(plan(tiered).as_deref(), Some("Google AI Ultra"));
    let blank = json!({ "userTier": { "name": "  " }, "planStatus": { "planInfo": { "planName": "Pro" } } });
    assert_eq!(plan(blank).as_deref(), Some("Pro"));
    assert_eq!(plan(json!({ "userTier": { "id": "x" }, "planStatus": {} })), None);
    assert_eq!(plan(info(json!({ "planName": 7 }))), None);
}

#[test]
fn a_hostile_plan_name_is_cut_and_stripped() {
    let name = format!("Pro\u{202e}{}", "x".repeat(500));
    let plan =
        parse_status(&json!({ "userStatus": { "userTier": { "name": name } } }), NOW).unwrap().plan;
    let plan = plan.expect("plan");
    assert_eq!(plan.chars().count(), MAX_TEXT);
    assert!(plan.starts_with("Prox") && !plan.contains('\u{202e}'));
}

#[test]
fn a_body_without_a_user_status_is_a_changed_endpoint() {
    for body in [
        json!({}),
        json!([]),
        json!(null),
        json!({ "userStatus": {} }),
        json!({ "userStatus": null }),
        json!({ "userStatus": "ok" }),
        json!({ "userStatus": { "email": "someone@example.invalid" } }),
        json!({ "userStatus": { "planStatus": "Pro" } }),
        json!({ "status": { "planStatus": {} } }),
    ] {
        let error = parse_status(&body, NOW).expect_err("must not parse");
        assert_eq!(error.code, ErrorCode::EndpointChanged, "{body}");
        assert!(!error.retryable);
    }
}

#[test]
fn a_status_code_in_the_body_is_reported_before_any_data() {
    let refused = parse_status(&json!({ "code": 16 }), NOW).unwrap_err();
    assert_eq!((refused.code, refused.retryable), (ErrorCode::Unauthorized, false));
    let mut body = fixture("status_ide");
    body["code"] = json!(16);
    assert_eq!(parse_status(&body, NOW).unwrap_err().code, ErrorCode::Unauthorized);
    body["code"] = json!(5);
    assert_eq!(parse_status(&body, NOW).unwrap_err().code, ErrorCode::EndpointChanged);
}

#[test]
fn only_the_first_models_are_read() {
    let mut configs: Vec<Value> = (0..MAX_MODELS)
        .map(|_| json!({ "label": "Gemini 3 Pro", "quotaInfo": { "remainingFraction": 0.9 } }))
        .collect();
    configs.push(json!({ "label": "Gemini 3 Pro", "quotaInfo": { "remainingFraction": 0.0 } }));
    let window = window_of(&read(&status_of(Value::Array(configs))), &Pool::Gemini);
    assert_eq!(window.used_pct, Some(10.0));
}
