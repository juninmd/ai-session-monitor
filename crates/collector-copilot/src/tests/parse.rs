//! The response body as windows and facts: the shapes seen in the wild, and every way a field can
//! be wrong.

use serde_json::{Value, json};
use usage_model::{AmountUnit, ErrorCode, HarnessError, UsageWindow, WindowKind};

use super::{FIRST_OF_FEBRUARY, NOW, fixture};
use crate::parse::{Parsed, parse_usage};

fn parse_fixture(name: &str) -> Result<Parsed, HarnessError> {
    let body: Value = serde_json::from_str(&fixture(name)).unwrap();
    parse_usage(&body, NOW)
}

fn parse_json(body: &Value) -> Result<Parsed, HarnessError> {
    parse_usage(body, NOW)
}

fn window<'a>(parsed: &'a Parsed, id: &str) -> &'a UsageWindow {
    parsed.windows.iter().find(|window| window.id == id).unwrap_or_else(|| panic!("no {id}"))
}

fn ids(parsed: &Parsed) -> Vec<&str> {
    parsed.windows.iter().map(|window| window.id.as_str()).collect()
}

fn fact_ids(parsed: &Parsed) -> Vec<&str> {
    parsed.facts.iter().map(|fact| fact.id.as_str()).collect()
}

#[test]
fn an_individual_pro_account_has_a_premium_window_and_unlimited_facts() {
    let parsed = parse_fixture("individual_pro.json").unwrap();
    assert_eq!(parsed.plan.as_deref(), Some("Individual Pro"));
    assert_eq!(ids(&parsed), ["premium_requests"]);
    let premium = window(&parsed, "premium_requests");
    assert_eq!(premium.kind, WindowKind::Monthly);
    assert_eq!(premium.window_minutes, None, "months differ in length: no pace alarm");
    assert!((premium.used_pct.unwrap() - 26.6667).abs() < 1e-9, "{:?}", premium.used_pct);
    assert_eq!(premium.resets_at, Some(FIRST_OF_FEBRUARY));
    let amount = premium.amount.as_ref().unwrap();
    assert_eq!((amount.used, amount.limit, amount.unit), (80.0, Some(300.0), AmountUnit::Requests));
    assert!(!parsed.reset_derived, "the response gave the date");

    assert_eq!(fact_ids(&parsed), ["chat_unlimited", "completions_unlimited"]);
    for fact in &parsed.facts {
        assert_eq!(
            (fact.number, fact.text.as_deref(), fact.unit.as_deref()),
            (None, Some("unlimited"), None)
        );
    }
}

#[test]
fn unlimited_quotas_are_facts_never_windows_and_never_zero() {
    let parsed = parse_fixture("free_unlimited_chat.json").unwrap();
    assert!(parsed.windows.is_empty(), "entitlement 0 on premium is no quota, not 0 %");
    assert_eq!(fact_ids(&parsed), ["chat_unlimited", "completions_unlimited"]);
    assert_eq!(parsed.plan.as_deref(), Some("Free"));
    assert!(!parsed.reset_derived, "no window, nothing derived");
}

#[test]
fn finite_chat_and_completions_become_windows_of_their_own() {
    let parsed = parse_fixture("free_finite.json").unwrap();
    assert_eq!(ids(&parsed), ["chat", "completions"]);
    let chat = window(&parsed, "chat");
    assert_eq!(chat.used_pct, Some(24.0), "100 - percent_remaining");
    assert_eq!(chat.amount.as_ref().map(|a| (a.used, a.limit)), Some((12.0, Some(50.0))));
    let completions = window(&parsed, "completions");
    assert_eq!(completions.used_pct, Some(25.0), "from entitlement and remaining");
    assert_eq!(completions.resets_at, Some(FIRST_OF_FEBRUARY));
    assert!(parsed.facts.is_empty());
}

#[test]
fn counts_and_percentages_sent_as_strings_are_read() {
    let parsed = parse_fixture("business.json").unwrap();
    assert_eq!(parsed.plan.as_deref(), Some("Business"));
    let premium = window(&parsed, "premium_requests");
    assert_eq!(premium.used_pct, Some(75.0));
    assert_eq!(premium.amount.as_ref().map(|a| (a.used, a.limit)), Some((225.0, Some(300.0))));
    assert_eq!(premium.resets_at, Some(FIRST_OF_FEBRUARY), "a bare date is midnight UTC");
}

#[test]
fn an_overdrawn_quota_clamps_the_share_but_keeps_the_real_count() {
    let parsed = parse_fixture("overage.json").unwrap();
    let premium = window(&parsed, "premium_requests");
    assert_eq!(premium.used_pct, Some(100.0));
    assert_eq!(premium.amount.as_ref().map(|a| a.used), Some(1_042.0), "limit - (-42)");
    assert_eq!(parsed.plan.as_deref(), Some("Enterprise"));
    assert_eq!(fact_ids(&parsed), ["premium_requests_overage"]);
    let fact = &parsed.facts[0];
    assert_eq!((fact.number, fact.unit.as_deref()), (Some(42.0), Some("requests")));
}

#[test]
fn a_missing_reset_date_is_derived_as_the_first_of_next_month_and_flagged() {
    let parsed = parse_fixture("no_reset.json").unwrap();
    let premium = window(&parsed, "premium_requests");
    assert_eq!(premium.resets_at, Some(FIRST_OF_FEBRUARY), "2027-01-15 -> 2027-02-01T00:00Z");
    assert_eq!(premium.used_pct, Some(20.0));
    assert!(parsed.reset_derived);
}

#[test]
fn an_unusable_reported_reset_falls_back_to_the_derived_one() {
    let with = |date: Value| {
        json!({"quota_reset_date_utc": date, "quota_snapshots": {"premium_interactions":
            {"entitlement": 100, "remaining": 90}}})
    };
    for date in [
        json!("2099-01-01T00:00:00Z"),
        json!("2027-01-14T00:00:00Z"),
        json!("2027-01-15T08:00:00Z"),
        json!("yesterday"),
        json!(20_270_201),
        Value::Null,
    ] {
        let parsed = parse_json(&with(date.clone())).unwrap();
        assert!(parsed.reset_derived, "{date}");
        assert_eq!(parsed.windows[0].resets_at, Some(FIRST_OF_FEBRUARY), "{date}");
    }
}

#[test]
fn the_date_only_field_is_used_when_the_utc_one_is_missing_or_bad() {
    let body = json!({"quota_reset_date_utc": "garbage", "quota_reset_date": "2027-02-01",
        "quota_snapshots": {"premium_interactions": {"entitlement": 100, "remaining": 90}}});
    let parsed = parse_json(&body).unwrap();
    assert!(!parsed.reset_derived);
    assert_eq!(parsed.windows[0].resets_at, Some(FIRST_OF_FEBRUARY));
}

#[test]
fn hostile_values_give_unknown_never_a_believable_number() {
    // Every number is out of range or the wrong type: nothing to show, so the shape is unknown.
    let error = parse_fixture("hostile.json").unwrap_err();
    assert_eq!((error.code, error.retryable), (ErrorCode::EndpointChanged, false));
}

#[test]
fn each_bad_field_degrades_alone() {
    let premium = |quota: Value| {
        let body = json!({"quota_snapshots": {"premium_interactions": quota}});
        parse_json(&body)
    };
    // An absurd percentage is ignored; the counts still give the share.
    let parsed =
        premium(json!({"entitlement": 200, "remaining": 150, "percent_remaining": 1e300})).unwrap();
    assert_eq!(parsed.windows[0].used_pct, Some(25.0));
    // A negative entitlement is no entitlement: percentage only, no amount.
    let parsed =
        premium(json!({"entitlement": -1, "remaining": 5, "percent_remaining": 60})).unwrap();
    assert_eq!((parsed.windows[0].used_pct, parsed.windows[0].amount.clone()), (Some(40.0), None));
    // A percentage beyond 100 is clamped, not shown as negative use.
    let parsed = premium(json!({"percent_remaining": 150})).unwrap();
    assert_eq!(parsed.windows[0].used_pct, Some(0.0));
    // No usable number at all: no window (and, being the only content, an unknown shape).
    for quota in [json!({}), json!({"entitlement": 0, "remaining": 0}), json!({"entitlement": "x"})]
    {
        assert_eq!(premium(quota).unwrap_err().code, ErrorCode::EndpointChanged);
    }
}

#[test]
fn the_unlimited_flag_is_read_leniently_and_overrides_the_numbers() {
    for flag in [json!(true), json!("true"), json!("TRUE")] {
        let body = json!({"quota_snapshots": {"chat": {"unlimited": flag,
            "entitlement": 50, "remaining": 1, "percent_remaining": 2}}});
        let parsed = parse_json(&body).unwrap();
        assert!(parsed.windows.is_empty(), "{flag}");
        assert_eq!(fact_ids(&parsed), ["chat_unlimited"]);
    }
}

#[test]
fn plan_labels_are_cleaned_capped_and_never_carry_markup_or_control_characters() {
    let plan = |raw: Value| {
        let body = json!({"copilot_plan": raw, "quota_snapshots": {"chat": {"unlimited": true}}});
        parse_json(&body).unwrap().plan
    };
    assert_eq!(plan(json!("individual")).as_deref(), Some("Individual"));
    assert_eq!(plan(json!("copilot_for-business")).as_deref(), Some("Copilot For Business"));
    assert_eq!(plan(json!("  ENTERPRISE ")).as_deref(), Some("Enterprise"));
    let long = plan(json!("a".repeat(100))).unwrap();
    assert_eq!(long.len(), 32);
    for hostile in [
        json!("<b>x</b>"),
        json!("a\u{7}b"),
        json!("a\nb"),
        json!(""),
        json!("   "),
        json!(5),
        json!(["x"]),
        Value::Null,
        json!("pro\u{202e}"),
    ] {
        assert_eq!(plan(hostile.clone()), None, "{hostile}");
    }
}

#[test]
fn bodies_that_are_not_the_quota_document_are_endpoint_changed() {
    for body in [
        json!({"hello": 1}),
        json!([]),
        json!(null),
        json!("text"),
        json!({"quota_snapshots": []}),
        json!({"quota_snapshots": {}}),
        json!({"copilot_plan": "free"}),
        json!({"quota_snapshots": {"unknown_future": {"entitlement": 10, "remaining": 5}}}),
    ] {
        let error = parse_json(&body).unwrap_err();
        assert_eq!((error.code, error.retryable), (ErrorCode::EndpointChanged, false), "{body}");
    }
}

#[test]
fn unknown_fields_are_ignored() {
    let mut body: Value = serde_json::from_str(&fixture("individual_pro.json")).unwrap();
    body["brand_new"] = json!({"nested": [1, 2, 3]});
    body["quota_snapshots"]["premium_interactions"]["extra"] = json!("x");
    assert_eq!(parse_json(&body).unwrap().windows.len(), 1);
}
