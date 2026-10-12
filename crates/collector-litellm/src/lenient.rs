//! Scalar readers for server-supplied JSON. A value of the wrong type, a non-finite, a negative
//! or an absurdly large number is unknown, never clamped into something that looks valid.

use serde_json::Value;
use usage_support::parse_rfc3339_secs;

/// Largest integer a JavaScript number holds exactly; the UI cannot show more faithfully.
const MAX_NUMBER: f64 = 9_007_199_254_740_991.0;

fn in_range(value: Option<&Value>) -> Option<f64> {
    value?.as_f64().filter(|n| n.is_finite() && n.abs() <= MAX_NUMBER)
}

pub(crate) fn non_negative(value: Option<&Value>) -> Option<f64> {
    // `-0.0` is the same amount as `0.0` but serializes differently.
    in_range(value).filter(|n| *n >= 0.0).map(|n| if n == 0.0 { 0.0 } else { n })
}

pub(crate) fn positive(value: Option<&Value>) -> Option<f64> {
    in_range(value).filter(|n| *n > 0.0)
}

pub(crate) fn text(value: Option<&Value>) -> Option<String> {
    value?.as_str().map(str::to_owned)
}

/// RFC 3339 text to epoch seconds; the proxy never sends a bare number for a date.
pub(crate) fn epoch(value: Option<&Value>) -> Option<i64> {
    parse_rfc3339_secs(value?.as_str()?)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn numbers_outside_the_exact_range_are_unknown() {
        let edge = json!(9_007_199_254_740_991_u64);
        assert_eq!(positive(Some(&edge)), Some(MAX_NUMBER));
        for beyond in [json!(9_007_199_254_740_993_u64), json!(1e300), json!(u64::MAX)] {
            assert_eq!(positive(Some(&beyond)), None, "{beyond}");
            assert_eq!(non_negative(Some(&beyond)), None, "{beyond}");
        }
        assert_eq!(non_negative(Some(&json!(-9_007_199_254_740_993_i64))), None);
    }

    #[test]
    fn only_json_numbers_count_as_numbers() {
        for value in [json!("12.5"), json!(true), json!([1]), json!({"n": 1}), json!(null)] {
            assert_eq!(non_negative(Some(&value)), None, "{value}");
            assert_eq!(positive(Some(&value)), None, "{value}");
        }
        assert_eq!(non_negative(None), None);
        assert_eq!(positive(None), None);
    }

    #[test]
    fn zero_is_a_spend_but_not_a_cap_and_negative_zero_is_plain_zero() {
        assert_eq!(non_negative(Some(&json!(0))), Some(0.0));
        assert_eq!(positive(Some(&json!(0))), None);
        assert_eq!(positive(Some(&json!(-0.0))), None);
        let zero = non_negative(Some(&json!(-0.0)));
        assert!(zero.is_some_and(|n| n == 0.0 && n.is_sign_positive()));
        assert_eq!(non_negative(Some(&json!(-0.01))), None);
    }

    #[test]
    fn text_needs_a_string_and_dates_need_rfc_3339() {
        assert_eq!(text(Some(&json!("ci-bot"))).as_deref(), Some("ci-bot"));
        assert_eq!(text(Some(&json!(7))), None);
        assert_eq!(text(None), None);
        assert_eq!(epoch(Some(&json!("2026-11-01T00:00:00Z"))), Some(1_793_491_200));
        assert_eq!(epoch(Some(&json!("2026-11-01T00:00:00+02:00"))), Some(1_793_484_000));
        for bad in [json!(1_793_491_200), json!("tomorrow"), json!(""), json!(null)] {
            assert_eq!(epoch(Some(&bad)), None, "{bad}");
        }
    }
}
