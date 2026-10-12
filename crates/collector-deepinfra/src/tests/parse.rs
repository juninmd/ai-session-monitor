//! The two documents: dollars out, nothing invented.

use serde_json::{Value, json};
use usage_model::ErrorCode;

use super::fixture;
use crate::parse::{parse_checklist, parse_month_spend};

fn body(name: &str) -> Value {
    serde_json::from_str(&fixture(name)).unwrap()
}

#[test]
fn a_limited_account_keeps_balance_recent_usage_and_limit() {
    let parsed = parse_checklist(&body("checklist_limited.json")).unwrap();
    assert_eq!(
        (parsed.stripe_balance, parsed.recent, parsed.limit),
        (-12.5, Some(2.5), Some(10.0))
    );
}

#[test]
fn a_null_limit_is_no_limit_not_zero() {
    let parsed = parse_checklist(&body("checklist_unlimited.json")).unwrap();
    assert_eq!(parsed.limit, None);
}

#[test]
fn hostile_figures_are_unknown_and_a_non_numeric_balance_means_the_endpoint_changed() {
    assert_eq!(
        parse_checklist(&body("checklist_hostile.json")).unwrap_err().code,
        ErrorCode::EndpointChanged
    );
    let parsed = parse_checklist(&json!({"stripe_balance": 0, "recent": -5, "limit": 0})).unwrap();
    assert_eq!((parsed.recent, parsed.limit), (None, None));
    assert_eq!(parsed.stripe_balance, 0.0, "a zero balance is a real zero");
}

#[test]
fn documents_of_the_wrong_shape_mean_the_endpoint_changed() {
    for document in [json!([]), json!(null), json!({}), json!({"stripe_balance": null})] {
        assert_eq!(
            parse_checklist(&document).unwrap_err().code,
            ErrorCode::EndpointChanged,
            "{document}"
        );
    }
}

#[test]
fn the_month_total_is_cents_turned_into_dollars() {
    assert_eq!(parse_month_spend(&body("usage_ok.json")), Some(12.34));
}

#[test]
fn an_unreadable_usage_document_leaves_the_month_unknown() {
    for document in [
        json!({}),
        json!({"months": []}),
        json!({"months": [{"total_cost": -1}]}),
        json!({"months": [{"total_cost": "9"}]}),
        json!([]),
    ] {
        assert_eq!(parse_month_spend(&document), None, "{document}");
    }
}
