//! The mapping from the response body to windows and facts.

use serde_json::{Value, json};
use usage_model::{Amount, AmountUnit, ErrorCode, Fact, WindowKind};

use super::{NOW, fixture};
use crate::parse::{Parsed, parse_summary};

/// 2027-01-01T00:00:00Z and 2027-02-01T00:00:00Z.
const CYCLE_START: i64 = 1_798_761_600;
const CYCLE_END: i64 = 1_801_440_000;
const CYCLE_MINUTES: u32 = 44_640;

fn parsed(name: &str) -> Parsed {
    let body: Value = serde_json::from_str(&fixture(name)).unwrap();
    parse_summary(&body, NOW).unwrap()
}

fn parse(body: &Value) -> Result<Parsed, ErrorCode> {
    parse_summary(body, NOW).map_err(|error| error.code)
}

#[test]
fn a_pro_account_mid_cycle_has_one_monthly_window_from_the_servers_own_percent() {
    let pro = parsed("ok_pro.json");
    assert_eq!(pro.plan.as_deref(), Some("Pro"));
    assert!(pro.facts.is_empty(), "a switched-off on-demand block says nothing");
    assert_eq!(pro.windows.len(), 1);
    let window = &pro.windows[0];
    assert_eq!((window.id.as_str(), window.kind), ("monthly", WindowKind::Monthly));
    // totalPercentUsed (37.5) wins over used / limit (75).
    assert_eq!(window.used_pct, Some(37.5));
    assert_eq!(window.resets_at, Some(CYCLE_END));
    assert_eq!(window.window_minutes, Some(CYCLE_MINUTES));
    assert_eq!(CYCLE_END - CYCLE_START, i64::from(CYCLE_MINUTES) * 60);
    assert!(window.amount.is_none(), "cents or requests: the unit is not stated, so no amount");
    assert!(window.group.is_none() && window.label.is_none());
}

#[test]
fn without_a_percent_the_share_is_used_over_limit() {
    let body = json!({
        "billingCycleEnd": "2027-02-01T00:00:00Z",
        "individualUsage": { "plan": { "used": 500, "limit": 2000 } }
    });
    assert_eq!(parse(&body).unwrap().windows[0].used_pct, Some(25.0));
    let strings = json!({ "individualUsage": { "plan": { "used": "500", "limit": "2000" } } });
    assert_eq!(parse(&strings).unwrap().windows[0].used_pct, Some(25.0));
}

#[test]
fn a_real_zero_stays_zero_and_a_missing_share_stays_unknown() {
    let zero = json!({ "individualUsage": { "plan": { "totalPercentUsed": 0 } } });
    assert_eq!(parse(&zero).unwrap().windows[0].used_pct, Some(0.0));
    let zero_used = json!({ "individualUsage": { "plan": { "used": 0, "limit": 500 } } });
    assert_eq!(parse(&zero_used).unwrap().windows[0].used_pct, Some(0.0));
    let unknown = json!({
        "billingCycleEnd": "2027-02-01T00:00:00Z",
        "individualUsage": { "plan": { "enabled": true } }
    });
    let window = &parse(&unknown).unwrap().windows[0];
    assert_eq!((window.used_pct, window.resets_at), (None, Some(CYCLE_END)));
    // Used without any limit is not a share.
    let no_limit = json!({ "individualUsage": { "plan": { "used": 50, "limit": 0 } } });
    assert_eq!(parse(&no_limit), Err(ErrorCode::EndpointChanged));
}

#[test]
fn an_overage_clamps_the_percent_to_one_hundred() {
    let over = json!({ "individualUsage": { "plan": { "totalPercentUsed": 180.5 } } });
    assert_eq!(parse(&over).unwrap().windows[0].used_pct, Some(100.0));
    let over_ratio = json!({ "individualUsage": { "plan": { "used": 3000, "limit": 2000 } } });
    assert_eq!(parse(&over_ratio).unwrap().windows[0].used_pct, Some(100.0));
}

#[test]
fn on_demand_spend_with_a_cap_is_a_usd_budget_window() {
    let spend = parsed("ok_on_demand.json");
    assert_eq!(spend.plan.as_deref(), Some("Pro Plus"));
    assert_eq!(spend.windows.len(), 2);
    assert_eq!(spend.windows[0].used_pct, Some(100.0));
    let budget = &spend.windows[1];
    assert_eq!((budget.id.as_str(), budget.kind), ("on_demand", WindowKind::Budget));
    assert_eq!(budget.used_pct, Some(25.0));
    assert_eq!(
        budget.amount,
        Some(Amount { used: 12.5, limit: Some(50.0), unit: AmountUnit::Usd })
    );
    assert_eq!((budget.resets_at, budget.window_minutes), (Some(CYCLE_END), Some(CYCLE_MINUTES)));
}

#[test]
fn on_demand_used_can_be_derived_from_remaining() {
    let body = json!({ "individualUsage": { "onDemand": { "enabled": true, "limit": 5000, "remaining": 4000 } } });
    let window = &parse(&body).unwrap().windows[0];
    assert_eq!(window.used_pct, Some(20.0));
    assert_eq!(window.amount.as_ref().map(|a| a.used), Some(10.0));
    // A limit without a usable used figure is a window with an unknown share, not 0 %.
    let unknown = json!({ "individualUsage": { "onDemand": { "limit": 5000 } } });
    let window = &parse(&unknown).unwrap().windows[0];
    assert_eq!((window.used_pct, window.amount.clone()), (None, None));
}

#[test]
fn an_on_demand_block_that_is_switched_off_or_empty_adds_nothing() {
    let off = json!({ "individualUsage": { "plan": { "totalPercentUsed": 5 }, "onDemand": { "enabled": false, "used": 900, "limit": 1000 } } });
    assert_eq!(parse(&off).unwrap().windows.len(), 1);
    let zero_spend = json!({ "individualUsage": { "plan": { "totalPercentUsed": 5 }, "onDemand": { "enabled": true, "used": 0, "limit": null } } });
    let result = parse(&zero_spend).unwrap();
    assert_eq!((result.windows.len(), result.facts.len()), (1, 0));
}

#[test]
fn an_unlimited_account_has_no_window_and_a_fact_instead_of_a_zero() {
    let unlimited = parsed("unlimited.json");
    assert_eq!(unlimited.plan.as_deref(), Some("Enterprise"));
    assert!(unlimited.windows.is_empty());
    assert_eq!(
        unlimited.facts,
        [
            Fact { id: "unlimited".into(), number: None, text: Some("true".into()), unit: None },
            Fact {
                id: "on_demand_spend".into(),
                number: Some(3.21),
                text: None,
                unit: Some("usd".into())
            },
        ]
    );
}

#[test]
fn hostile_values_become_unknown_never_numbers_or_text() {
    let hostile = parsed("hostile.json");
    // Control characters are stripped, then the rest is not a plain token: no label.
    assert_eq!(hostile.plan, None);
    assert!(hostile.facts.is_empty(), "isUnlimited was the string \"yes\", not true");
    assert_eq!(hostile.windows.len(), 1, "the absurd on-demand cap is dropped");
    let window = &hostile.windows[0];
    assert_eq!(window.used_pct, Some(100.0));
    // The cycle ends before it starts: no length, but the stated reset stays.
    assert_eq!(window.window_minutes, None);
    assert_eq!(window.resets_at, Some(1_801_440_000));
    let json = serde_json::to_string(&hostile.windows).unwrap();
    assert!(!json.contains("script") && !json.contains("FAKE"));
}

#[test]
fn reset_dates_that_cannot_be_right_are_unknown() {
    let at = |end: &str| {
        let body = json!({ "billingCycleEnd": end, "individualUsage": { "plan": { "totalPercentUsed": 1 } } });
        parse(&body).unwrap().windows[0].resets_at
    };
    assert_eq!(at("9999-12-31T23:59:59Z"), None);
    assert_eq!(at("2030-01-01T00:00:00Z"), None, "more than a year away");
    assert_eq!(at("1969-12-31T23:59:59Z"), None);
    assert_eq!(at("not a date"), None);
    assert_eq!(at("2027-02-01T00:00:00Z"), Some(CYCLE_END));
    // Already passed: kept, so the view can be marked stale.
    assert_eq!(at("2026-12-01T00:00:00Z"), Some(1_796_083_200));
}

#[test]
fn a_reset_farther_than_the_stated_cycle_is_a_clock_fault() {
    let body = json!({
        "billingCycleStart": "2027-01-14T00:00:00Z",
        "billingCycleEnd": "2027-01-16T00:00:00Z",
        "individualUsage": { "plan": { "totalPercentUsed": 1 } }
    });
    let window = &parse(&body).unwrap().windows[0];
    assert_eq!((window.window_minutes, window.resets_at), (Some(2_880), Some(1_800_057_600)));
    let skewed = json!({
        "billingCycleStart": "2020-01-01T00:00:00Z",
        "billingCycleEnd": "2020-01-02T00:00:00Z",
        "individualUsage": { "plan": { "totalPercentUsed": 1 } }
    });
    let window = &parse(&skewed).unwrap().windows[0];
    assert_eq!(window.window_minutes, Some(1_440));
    assert_eq!(window.resets_at, Some(1_577_923_200), "in the past is allowed: it is history");
}

#[test]
fn plan_labels_are_title_cased_capped_and_plain() {
    let label = |raw: &str| {
        let body = json!({ "membershipType": raw, "isUnlimited": true });
        parse(&body).unwrap().plan
    };
    assert_eq!(label("pro").as_deref(), Some("Pro"));
    assert_eq!(label("pro_plus").as_deref(), Some("Pro Plus"));
    assert_eq!(label("ULTRA").as_deref(), Some("Ultra"));
    assert_eq!(label("free_trial").as_deref(), Some("Free Trial"));
    assert_eq!(label("enterprise\n").as_deref(), Some("Enterprise"));
    assert_eq!(
        label("pro\u{7}_plus").as_deref(),
        Some("Pro Plus"),
        "control characters are stripped"
    );
    for hostile in [
        "",
        "   ",
        "<b>pro</b>",
        "pro; drop",
        "pro\u{202e}",
        "caf\u{e9}",
        &"a".repeat(25),
        "../../etc",
    ] {
        assert_eq!(label(hostile), None, "{hostile:?}");
    }
    assert_eq!(label(&"a".repeat(24)).map(|s| s.len()), Some(24));
    assert_eq!(parse(&json!({ "membershipType": 7, "isUnlimited": true })).unwrap().plan, None);
}

#[test]
fn bodies_that_are_not_the_summary_are_endpoint_changed() {
    for name in ["wrong_types.json", "not_an_object.json", "error_auth.json"] {
        let body: Value = serde_json::from_str(&fixture(name)).unwrap();
        assert_eq!(parse(&body), Err(ErrorCode::EndpointChanged), "{name}");
    }
    for body in [
        json!({}),
        json!([]),
        json!(null),
        json!("x"),
        json!(7),
        json!({ "membershipType": "pro" }),
    ] {
        assert_eq!(parse(&body), Err(ErrorCode::EndpointChanged), "{body}");
    }
}

#[test]
fn unknown_fields_and_team_usage_are_ignored() {
    let body = json!({
        "individualUsage": { "plan": { "totalPercentUsed": 10, "surprise": [1] }, "extra": true },
        "teamUsage": { "plan": { "totalPercentUsed": 99 }, "onDemand": { "enabled": true, "limit": 100, "used": 50 } },
        "somethingNew": { "a": 1 }
    });
    let result = parse(&body).unwrap();
    assert_eq!(result.windows.len(), 1);
    assert_eq!(result.windows[0].used_pct, Some(10.0));
}
