//! The two documents: real limits only, unknown stays unknown.

use serde_json::{Value, json};
use usage_model::ErrorCode;

use super::fixture;
use crate::parse::{Credits, KeyInfo, parse_credits, parse_key};

fn body(name: &str) -> Value {
    serde_json::from_str(&fixture(name)).unwrap()
}

#[test]
fn a_limited_key_reports_its_cap_and_usage() {
    let info = parse_key(&body("key_limited.json")).unwrap();
    assert_eq!(info.limit, Some(10.0));
    assert_eq!(info.limit_remaining, Some(7.5));
    assert_eq!(info.usage, Some(2.5));
    assert_eq!(
        (info.usage_daily, info.usage_weekly, info.usage_monthly),
        (Some(0.25), Some(1.0), Some(2.5))
    );
}

#[test]
fn an_unlimited_key_has_no_limit_which_is_not_zero() {
    let info = parse_key(&body("key_unlimited.json")).unwrap();
    assert_eq!((info.limit, info.limit_remaining), (None, None));
    assert_eq!(info.usage_daily, Some(0.0), "a reported zero stays a zero");
}

#[test]
fn an_overspent_key_keeps_its_negative_remaining() {
    assert_eq!(parse_key(&body("key_overspent.json")).unwrap().limit_remaining, Some(-1.5));
}

#[test]
fn hostile_values_are_unknown_and_leave_nothing_usable() {
    assert_eq!(parse_key(&body("key_hostile.json")).unwrap_err().code, ErrorCode::EndpointChanged);
    let partial = json!({"data": {"usage": 1, "usage_daily": 1e99, "usage_weekly": "2", "usage_monthly": -1}});
    let info = parse_key(&partial).unwrap();
    assert_eq!(info, KeyInfo { usage: Some(1.0), ..KeyInfo::default() });
}

#[test]
fn documents_of_the_wrong_shape_mean_the_endpoint_changed() {
    for doc in
        [json!([]), json!(null), json!({}), json!({"data": []}), json!({"data": {"label": "x"}})]
    {
        assert_eq!(parse_key(&doc).unwrap_err().code, ErrorCode::EndpointChanged, "{doc}");
    }
}

#[test]
fn credits_need_both_figures() {
    assert_eq!(
        parse_credits(&body("credits_ok.json")),
        Some(Credits { total: 100.5, used: 25.75 })
    );
    for doc in [
        json!({}),
        json!({"data": {"total_credits": 5}}),
        json!({"data": {"total_credits": "5", "total_usage": 1}}),
    ] {
        assert_eq!(parse_credits(&doc), None, "{doc}");
    }
}
