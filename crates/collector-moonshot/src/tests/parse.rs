//! The balance document: dollars as reported, nothing invented.

use serde_json::{Value, json};
use usage_model::{ErrorCode, Fact};

use super::fixture;
use crate::parse::parse_balance;

fn parse(name: &str) -> Result<Vec<Fact>, ErrorCode> {
    let body: Value = serde_json::from_str(&fixture(name)).unwrap();
    parse_balance(&body).map_err(|error| error.code)
}

fn summary(facts: &[Fact]) -> Vec<(&str, f64)> {
    facts.iter().map(|f| (f.id.as_str(), f.number.unwrap())).collect()
}

#[test]
fn the_three_figures_become_dollar_facts_in_order() {
    let facts = parse("ok.json").unwrap();
    assert_eq!(summary(&facts), [("balance", 49.58894), ("voucher", 46.58893), ("cash", 3.00001)]);
    assert!(facts.iter().all(|f| f.unit.as_deref() == Some("usd") && f.text.is_none()));
}

#[test]
fn a_negative_cash_balance_is_money_owed_and_a_zero_balance_a_real_zero() {
    let facts = parse("owes.json").unwrap();
    assert_eq!(summary(&facts), [("balance", 0.0), ("voucher", 0.0), ("cash", -2.5)]);
}

#[test]
fn hostile_values_are_unknown_and_never_zero() {
    let body =
        json!({"data": {"available_balance": "9", "voucher_balance": 1e30, "cash_balance": 1.5}});
    assert_eq!(summary(&parse_balance(&body).unwrap()), [("cash", 1.5)]);
    assert_eq!(parse("hostile.json").unwrap_err(), ErrorCode::EndpointChanged);
}

#[test]
fn a_body_that_says_it_failed_is_a_bad_response_not_a_balance() {
    assert_eq!(parse("failed.json").unwrap_err(), ErrorCode::BadResponse);
}

#[test]
fn documents_of_the_wrong_shape_mean_the_endpoint_changed() {
    assert_eq!(parse("empty_data.json").unwrap_err(), ErrorCode::EndpointChanged);
    for body in [json!([]), json!(null), json!({}), json!({"data": []}), json!({"data": 3})] {
        assert_eq!(parse_balance(&body).unwrap_err().code, ErrorCode::EndpointChanged, "{body}");
    }
}
