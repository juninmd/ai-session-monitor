//! The quota body as windows: the captured shape, and every way a field can be wrong.

use serde_json::{Value, json};
use usage_model::{AmountUnit, ErrorCode, UsageWindow, WindowKind};

use super::{NOW, fixture};
use crate::parse::{Parsed, parse_quota};

fn parse_fixture(name: &str) -> Result<Parsed, usage_model::HarnessError> {
    let body: Value = serde_json::from_str(&fixture(name)).unwrap();
    parse_quota(&body, NOW)
}

fn window<'a>(parsed: &'a Parsed, id: &str) -> &'a UsageWindow {
    parsed.windows.iter().find(|window| window.id == id).unwrap_or_else(|| panic!("no {id}"))
}

fn ids(parsed: &Parsed) -> Vec<&str> {
    parsed.windows.iter().map(|window| window.id.as_str()).collect()
}

#[test]
fn the_captured_shape_gives_five_hour_weekly_and_the_monthly_mcp_allowance() {
    let parsed = parse_fixture("ok_pro.json").unwrap();
    assert_eq!(ids(&parsed), ["five_hour", "weekly", "mcp"]);
    assert_eq!(parsed.plan.as_deref(), Some("Pro"));

    let five = window(&parsed, "five_hour");
    assert_eq!((five.kind, five.window_minutes), (WindowKind::FiveHour, Some(300)));
    assert_eq!((five.used_pct, five.resets_at), (Some(12.0), Some(NOW + 7_200)));
    assert_eq!(five.amount, None, "the body gives a share, not a count");

    let weekly = window(&parsed, "weekly");
    assert_eq!((weekly.kind, weekly.window_minutes), (WindowKind::Weekly, Some(10_080)));
    assert_eq!((weekly.used_pct, weekly.resets_at), (Some(40.0), Some(NOW + 3 * 86_400)));

    let mcp = window(&parsed, "mcp");
    assert_eq!((mcp.kind, mcp.window_minutes), (WindowKind::Monthly, None));
    assert_eq!((mcp.used_pct, mcp.resets_at), (Some(25.0), Some(1_801_440_000)));
    let amount = mcp.amount.as_ref().unwrap();
    assert_eq!(
        (amount.used, amount.limit, amount.unit),
        (250.0, Some(1_000.0), AmountUnit::Requests)
    );
}

#[test]
fn a_window_with_no_reset_is_kept_with_its_percentage_and_zero_stays_zero() {
    let parsed = parse_fixture("ok_lite.json").unwrap();
    assert_eq!(ids(&parsed), ["five_hour"]);
    assert_eq!(parsed.plan.as_deref(), Some("Lite"));
    let five = &parsed.windows[0];
    assert_eq!((five.used_pct, five.resets_at), (Some(0.0), None));
}

#[test]
fn credit_limits_use_their_counts_when_no_percentage_is_given() {
    let parsed = parse_fixture("credit_limit.json").unwrap();
    let five = window(&parsed, "five_hour");
    assert_eq!(five.used_pct, Some(25.0));
    let amount = five.amount.as_ref().unwrap();
    assert_eq!((amount.used, amount.limit, amount.unit), (100.0, Some(400.0), AmountUnit::Credits));
    assert_eq!(parsed.plan.as_deref(), Some("Max"));
}

#[test]
fn the_length_comes_from_the_unit_and_number_and_a_second_claim_on_a_slot_is_ignored() {
    let parsed = parse_fixture("duplicates.json").unwrap();
    assert_eq!(ids(&parsed), ["five_hour", "daily", "window_90m"]);
    assert_eq!(window(&parsed, "five_hour").used_pct, Some(11.0), "the first of two stands");
    assert_eq!(window(&parsed, "daily").window_minutes, Some(1_440));
    let odd = window(&parsed, "window_90m");
    assert_eq!(
        (odd.kind, odd.window_minutes, odd.used_pct),
        (WindowKind::Other, Some(90), Some(70.0))
    );
}

#[test]
fn a_reset_farther_than_the_window_is_long_is_unknown_but_the_percentage_stays() {
    let parsed = parse_fixture("far_reset.json").unwrap();
    let five = window(&parsed, "five_hour");
    assert_eq!((five.used_pct, five.resets_at), (Some(30.0), None), "10 h away for a 5 h window");
    let weekly = window(&parsed, "weekly");
    assert_eq!((weekly.used_pct, weekly.resets_at), (Some(10.0), None));
}

#[test]
fn a_reset_at_the_window_length_is_still_accepted() {
    let body = json!({ "code": 200, "success": true, "data": { "limits": [
        { "type": "TOKENS_LIMIT", "unit": 3, "number": 5, "percentage": 0,
          "nextResetTime": (NOW + 5 * 3_600) * 1_000 + 999 } ] } });
    let parsed = parse_quota(&body, NOW).unwrap();
    assert_eq!(parsed.windows[0].resets_at, Some(NOW + 5 * 3_600), "milliseconds are floored");
}

#[test]
fn bodies_without_a_readable_percentage_are_endpoint_changed() {
    for name in ["unknown_units.json", "hostile.json", "wrong_types.json"] {
        let error = parse_fixture(name).unwrap_err();
        assert_eq!((error.code, error.retryable), (ErrorCode::EndpointChanged, false), "{name}");
        assert!(error.params.is_empty(), "no body text in the error: {name}");
    }
    for body in [
        json!({}),
        json!([]),
        json!(null),
        json!("quota"),
        json!({ "data": [] }),
        json!({ "data": { "limits": null } }),
    ] {
        let error = parse_quota(&body, NOW).unwrap_err();
        assert_eq!(error.code, ErrorCode::EndpointChanged, "{body}");
    }
}

#[test]
fn an_empty_limit_list_is_an_unsupported_plan_not_zero_percent() {
    let error = parse_fixture("empty_limits.json").unwrap_err();
    assert_eq!((error.code, error.retryable), (ErrorCode::UnsupportedPlan, false));
}

#[test]
fn failure_envelopes_map_by_code_and_never_carry_the_message() {
    for (name, code) in [
        ("auth_in_band.json", ErrorCode::Unauthorized),
        ("rate_in_band.json", ErrorCode::RateLimited),
        ("failure_in_band.json", ErrorCode::ServerError),
        ("unknown_failure.json", ErrorCode::EndpointChanged),
    ] {
        let error = parse_fixture(name).unwrap_err();
        assert_eq!(error.code, code, "{name}");
        assert!(!format!("{error:?}").contains("fabricated"), "{name}");
    }
}

#[test]
fn success_false_alone_is_a_failure_even_with_data() {
    let body = json!({ "success": false, "data": { "limits": [
        { "type": "TOKENS_LIMIT", "unit": 3, "number": 5, "percentage": 5 } ] } });
    assert_eq!(parse_quota(&body, NOW).unwrap_err().code, ErrorCode::EndpointChanged);
}

#[test]
fn the_plan_is_shown_only_as_a_plain_token() {
    let with = |level: Value| {
        let body = json!({ "data": { "level": level, "limits": [
            { "type": "TOKENS_LIMIT", "unit": 3, "number": 5, "percentage": 5 } ] } });
        parse_quota(&body, NOW).unwrap().plan
    };
    assert_eq!(with(json!("MAX")).as_deref(), Some("Max"));
    assert_eq!(with(json!("")), None);
    assert_eq!(with(json!("<script>")), None);
    assert_eq!(with(json!("a-very-long-level-name-indeed")), None);
    assert_eq!(with(json!(7)), None);
    assert_eq!(with(json!("pro\u{202e}")), None);
}

#[test]
fn an_overage_in_counts_clamps_the_percent_but_the_amount_keeps_the_real_number() {
    let body = json!({ "code": 200, "success": true, "data": { "limits": [
        { "type": "CREDIT_LIMIT", "unit": 3, "number": 5, "usage": 100, "currentValue": 150 } ] } });
    let five = parse_quota(&body, NOW).unwrap().windows.remove(0);
    assert_eq!(five.used_pct, Some(100.0));
    assert_eq!(five.amount.map(|a| a.used), Some(150.0));
}

#[test]
fn documented_in_band_auth_codes_are_the_same_ambiguous_unauthorized_as_http_401() {
    for code in [1000, 1001, 1003] {
        let body = json!({ "code": code, "success": false, "msg": "fabricated" });
        let error = parse_quota(&body, NOW).unwrap_err();
        assert_eq!(error.code, ErrorCode::Unauthorized, "{code}");
        assert_eq!(error.params.get("ambiguous").map(String::as_str), Some("true"), "{code}");
    }
    // 1002 and 1004 are not documented anywhere; they stay an unknown failure.
    for code in [1002, 1004] {
        let body = json!({ "code": code, "success": false });
        assert_eq!(parse_quota(&body, NOW).unwrap_err().code, ErrorCode::EndpointChanged, "{code}");
    }
}
