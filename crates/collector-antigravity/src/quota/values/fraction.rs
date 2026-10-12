//! The share of a pool that is still left, as the server writes it.

use serde_json::Value;

const FIELD: &str = "remainingFraction";
/// The `agy` CLI writes the same field in snake case.
const FIELD_SNAKE: &str = "remaining_fraction";
/// A numeric string longer than this is not a share.
const MAX_NUMBER_TEXT: usize = 32;

/// Percent used (0..=100) from an object carrying the remaining share in any shape seen so far:
/// `remainingFraction`, `remaining.remainingFraction` or the oneof form
/// `remaining: {"case": "remainingFraction", "value": ..}`.
///
/// Absent, not a number, not finite or outside 0..=1 is `None`, never clamped: proto3 JSON omits
/// zero values, so a missing share cannot be read as "used up", and a share of 7 is a changed
/// meaning, not a full pool.
pub(crate) fn used_pct(object: &Value) -> Option<f64> {
    let remaining = object.get("remaining");
    let share = object
        .get(FIELD)
        .or_else(|| object.get(FIELD_SNAKE))
        .or_else(|| remaining.and_then(|inner| inner.get(FIELD)))
        .or_else(|| remaining.and_then(oneof))?;
    let fraction = number(share)?;
    let valid = fraction.is_finite() && (0.0..=1.0).contains(&fraction);
    // Four decimals keep `(1 - 0.9) * 100` from showing as 9.999999999999998.
    valid.then(|| (((1.0 - fraction) * 100.0) * 10_000.0).round() / 10_000.0)
}

fn oneof(remaining: &Value) -> Option<&Value> {
    let named = remaining.get("case")?.as_str()? == FIELD;
    named.then(|| remaining.get("value")).flatten()
}

/// Proto3 JSON writes doubles as numbers, and also as strings (`"0.5"`, `"NaN"`).
fn number(value: &Value) -> Option<f64> {
    match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) if text.len() <= MAX_NUMBER_TEXT => text.trim().parse().ok(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn the_three_shapes_give_the_same_answer() {
        for shape in [
            json!({ "remainingFraction": 0.9 }),
            json!({ "remaining": { "remainingFraction": 0.9 } }),
            json!({ "remaining": { "case": "remainingFraction", "value": 0.9 } }),
        ] {
            assert_eq!(used_pct(&shape), Some(10.0), "{shape}");
        }
    }

    #[test]
    fn used_is_what_is_not_left() {
        for (left, used) in [(1.0, 0.0), (0.75, 25.0), (0.333_333, 66.6667), (0.0, 100.0)] {
            assert_eq!(used_pct(&json!({ "remainingFraction": left })), Some(used), "{left}");
        }
        // An explicit zero is "nothing left"; only a missing field is unknown.
        assert_eq!(used_pct(&json!({ "remainingFraction": 0 })), Some(100.0));
        assert_eq!(used_pct(&json!({})), None);
    }

    #[test]
    fn a_direct_field_wins_over_the_nested_ones() {
        let both = json!({ "remainingFraction": 0.5, "remaining": { "remainingFraction": 0.1 } });
        assert_eq!(used_pct(&both), Some(50.0));
        // Present but unusable is unknown: the other shapes are not consulted.
        let broken =
            json!({ "remainingFraction": "NaN", "remaining": { "remainingFraction": 0.1 } });
        assert_eq!(used_pct(&broken), None);
    }

    #[test]
    fn strings_that_hold_numbers_are_read_like_numbers() {
        assert_eq!(used_pct(&json!({ "remainingFraction": "0.25" })), Some(75.0));
        assert_eq!(used_pct(&json!({ "remainingFraction": " 1e-1 " })), Some(90.0));
        let oneof = json!({ "remaining": { "case": "remainingFraction", "value": "0.5" } });
        assert_eq!(used_pct(&oneof), Some(50.0));
    }

    #[test]
    fn anything_else_is_unknown_and_never_clamped() {
        for bad in [
            json!("NaN"),
            json!("Infinity"),
            json!("-Infinity"),
            json!("inf"),
            json!("abc"),
            json!(""),
            json!("0.5 left"),
            json!("1".repeat(64)),
            json!(-0.1),
            json!(1.000_000_1),
            json!(2),
            json!(100),
            json!(-1),
            json!(1e308),
            json!(u64::MAX),
            json!(null),
            json!(true),
            json!([0.5]),
            json!({ "value": 0.5 }),
        ] {
            assert_eq!(used_pct(&json!({ "remainingFraction": bad.clone() })), None, "{bad}");
        }
    }

    #[test]
    fn the_oneof_form_needs_the_matching_case() {
        let other = json!({ "remaining": { "case": "remainingPercent", "value": 0.5 } });
        assert_eq!(used_pct(&other), None);
        let no_case = json!({ "remaining": { "value": 0.5 } });
        assert_eq!(used_pct(&no_case), None);
        let no_value = json!({ "remaining": { "case": "remainingFraction" } });
        assert_eq!(used_pct(&no_value), None);
        // A bare number under `remaining` could be a count: not a share.
        assert_eq!(used_pct(&json!({ "remaining": 0.5 })), None);
    }

    #[test]
    fn non_objects_are_unknown() {
        for value in [json!(null), json!(0.5), json!("0.5"), json!([]), json!(true)] {
            assert_eq!(used_pct(&value), None, "{value}");
        }
    }
}
