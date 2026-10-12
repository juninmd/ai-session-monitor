//! The balance document: per-currency facts, nothing invented, nothing summed.

use serde_json::{Value, json};
use usage_model::{ErrorCode, Fact};

use super::fixture;
use crate::parse::parse_balance;

fn parse(name: &str) -> Result<Vec<Fact>, ErrorCode> {
    let body: Value = serde_json::from_str(&fixture(name)).unwrap();
    parse_balance(&body).map_err(|error| error.code)
}

fn summary(facts: &[Fact]) -> Vec<(String, f64, String)> {
    facts.iter().map(|f| (f.id.clone(), f.number.unwrap(), f.unit.clone().unwrap())).collect()
}

#[test]
fn each_currency_keeps_its_own_facts_and_is_never_summed() {
    let facts = parse("ok_both.json").unwrap();
    let ids: Vec<&str> = facts.iter().map(|f| f.id.as_str()).collect();
    assert_eq!(
        ids,
        [
            "balance_cny",
            "granted_cny",
            "topped_up_cny",
            "balance_usd",
            "granted_usd",
            "topped_up_usd"
        ]
    );
    let usd = |f: &&Fact| f.unit.as_deref() == Some("usd");
    let usd_numbers: Vec<f64> = facts.iter().filter(usd).map(|f| f.number.unwrap()).collect();
    assert_eq!(usd_numbers, [3.5, 0.5, 3.0]);
    assert!(facts.iter().all(|f| f.text.is_none()));
}

#[test]
fn a_zero_balance_is_a_real_zero_not_unknown() {
    let facts = parse("ok_cny.json").unwrap();
    assert_eq!(summary(&facts)[0], ("balance_cny".to_owned(), 0.0, "cny".to_owned()));
}

#[test]
fn an_empty_list_is_an_account_without_balance_not_an_error() {
    assert_eq!(parse("empty.json").unwrap(), Vec::<Fact>::new());
}

#[test]
fn hostile_values_are_unknown_and_the_first_entry_of_a_currency_stands() {
    let facts = parse("hostile.json").unwrap();
    assert_eq!(summary(&facts), [("balance_cny".to_owned(), 7.25, "cny".to_owned())]);
}

#[test]
fn only_unknown_currencies_means_the_endpoint_changed() {
    assert_eq!(parse("unknown_currency.json").unwrap_err(), ErrorCode::EndpointChanged);
}

#[test]
fn documents_of_the_wrong_shape_mean_the_endpoint_changed() {
    let bodies = [json!([]), json!(null), json!({}), json!({"balance_infos": {}})];
    for body in bodies {
        assert_eq!(parse_balance(&body).unwrap_err().code, ErrorCode::EndpointChanged, "{body}");
    }
}

#[test]
fn json_numbers_are_tolerated_but_other_types_are_unknown() {
    let info = json!({"currency": "USD", "total_balance": 2.5, "granted_balance": true});
    let facts = parse_balance(&json!({ "balance_infos": [info] })).unwrap();
    assert_eq!(summary(&facts), [("balance_usd".to_owned(), 2.5, "usd".to_owned())]);
}
