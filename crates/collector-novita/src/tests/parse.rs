//! The balance document: 1/10000 dollars turned into dollars, nothing invented.

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
fn amounts_in_ten_thousandths_become_dollar_facts_in_order() {
    let facts = parse("ok.json").unwrap();
    assert_eq!(
        summary(&facts),
        [
            ("balance", 100.0),
            ("cash", 80.0),
            ("credit_limit", 20.0),
            ("pending", 0.0),
            ("owed", 0.0)
        ]
    );
    assert!(facts.iter().all(|f| f.unit.as_deref() == Some("usd") && f.text.is_none()));
}

#[test]
fn a_negative_balance_is_money_owed_and_stays_negative() {
    let facts = parse("owes.json").unwrap();
    assert_eq!(
        summary(&facts),
        [
            ("balance", -2.5),
            ("cash", -2.5),
            ("credit_limit", 0.0),
            ("pending", 0.15),
            ("owed", 2.5)
        ]
    );
}

#[test]
fn hostile_values_are_unknown_never_zero() {
    assert_eq!(parse("hostile.json").unwrap_err(), ErrorCode::EndpointChanged);
    let body = json!({"availableBalance": "10000", "creditLimit": "-5", "pendingCharges": true});
    assert_eq!(summary(&parse_balance(&body).unwrap()), [("balance", 1.0)]);
}

#[test]
fn json_numbers_are_tolerated() {
    let facts = parse_balance(&json!({"availableBalance": 25000})).unwrap();
    assert_eq!(summary(&facts), [("balance", 2.5)]);
}

#[test]
fn documents_of_the_wrong_shape_mean_the_endpoint_changed() {
    assert_eq!(parse("empty.json").unwrap_err(), ErrorCode::EndpointChanged);
    for body in [json!([]), json!(null), json!({"data": {}})] {
        assert_eq!(parse_balance(&body).unwrap_err().code, ErrorCode::EndpointChanged, "{body}");
    }
}
