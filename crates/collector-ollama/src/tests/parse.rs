//! The balance document in both of its plan shapes: percentages stay in range, nothing is invented.

use serde_json::{Value, json};
use usage_model::{AmountUnit, ErrorCode, UsageWindow, WindowKind};

use super::fixture;
use crate::parse::{Balance, parse_balance};

fn parse(name: &str) -> Result<Balance, ErrorCode> {
    let body: Value = serde_json::from_str(&fixture(name)).unwrap();
    parse_balance(&body).map_err(|error| error.code)
}

fn facts(balance: &Balance) -> Vec<(&str, f64)> {
    balance.facts.iter().map(|f| (f.id.as_str(), f.number.unwrap())).collect()
}

fn window<'a>(balance: &'a Balance, id: &str) -> &'a UsageWindow {
    balance.windows.iter().find(|w| w.id == id).unwrap()
}

#[test]
fn a_current_plan_is_a_monthly_money_window_with_its_period_and_two_facts() {
    let balance = parse("current.json").unwrap();
    let monthly = window(&balance, "monthly");
    assert_eq!((monthly.kind, monthly.used_pct), (WindowKind::Monthly, Some(27.5)));
    assert_eq!(monthly.resets_at, Some(1_801_353_600), "2027-01-31T00:00:00Z");
    assert_eq!(monthly.window_minutes, Some(30 * 24 * 60));
    let amount = monthly.amount.as_ref().unwrap();
    assert_eq!((amount.used, amount.limit, amount.unit), (27.5, Some(100.0), AmountUnit::Usd));
    assert_eq!(facts(&balance), [("included_balance", 72.5), ("purchased_balance", 25.0)]);
}

#[test]
fn more_balance_than_allowance_clamps_to_zero_used_instead_of_going_negative() {
    let balance = parse("overfull.json").unwrap();
    let monthly = window(&balance, "monthly");
    assert_eq!(monthly.used_pct, Some(0.0));
    assert_eq!(monthly.amount.as_ref().unwrap().used, 0.0);
}

#[test]
fn a_legacy_plan_has_a_session_and_a_weekly_window_from_the_remaining_percent() {
    let balance = parse("legacy.json").unwrap();
    let session = window(&balance, "session");
    assert_eq!((session.kind, session.used_pct), (WindowKind::Other, Some(20.0)));
    assert_eq!(session.window_minutes, None, "the length is not stated, so it is not guessed");
    assert!(session.resets_at.is_some());
    let weekly = window(&balance, "weekly");
    assert_eq!((weekly.kind, weekly.used_pct), (WindowKind::Weekly, Some(44.5)));
    assert_eq!(facts(&balance), [("purchased_balance", 0.0)], "a zero purchase is a real zero");
}

#[test]
fn a_usage_fraction_is_clamped_and_a_bad_reset_stays_unknown() {
    let balance = parse("usage_fraction.json").unwrap();
    assert_eq!(window(&balance, "session").used_pct, Some(25.0));
    let weekly = window(&balance, "weekly");
    assert_eq!((weekly.used_pct, weekly.resets_at), (Some(100.0), None));
}

#[test]
fn out_of_range_percentages_are_clamped() {
    let body = json!({"included": {"session": {"remaining_percent": 130}, "weekly": {"remaining_percent": -20}}});
    let balance = parse_balance(&body).unwrap();
    assert_eq!(window(&balance, "session").used_pct, Some(0.0));
    assert_eq!(window(&balance, "weekly").used_pct, Some(100.0));
}

#[test]
fn hostile_values_are_unknown_and_never_zero() {
    assert_eq!(parse("hostile.json").unwrap_err(), ErrorCode::EndpointChanged);
}

#[test]
fn documents_of_the_wrong_shape_mean_the_endpoint_changed() {
    for body in [json!([]), json!(null), json!({}), json!({"included": {}, "purchased": {}})] {
        assert_eq!(parse_balance(&body).unwrap_err().code, ErrorCode::EndpointChanged, "{body}");
    }
}
