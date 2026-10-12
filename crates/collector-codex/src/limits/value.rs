//! Reading loosely typed JSON numbers and keys without trusting them.

use serde_json::{Map, Value};

/// Whole numbers survive an f64 round trip below this.
const MAX_EXACT_INT: f64 = 9.0e15;

/// The first of `keys` that is present and not null.
pub(super) fn field<'a>(object: &'a Map<String, Value>, keys: &[&str]) -> Option<&'a Value> {
    keys.iter().find_map(|key| object.get(*key)).filter(|value| !value.is_null())
}

pub(super) fn number(value: Option<&Value>) -> Option<f64> {
    value?.as_f64().filter(|number| number.is_finite())
}

/// A whole number, also when written as a float (`300.0`); fractions are not whole.
pub(super) fn integer(value: Option<&Value>) -> Option<i64> {
    let value = value?;
    value.as_i64().or_else(|| {
        let whole = value.as_f64().filter(|n| n.fract() == 0.0 && n.abs() <= MAX_EXACT_INT)?;
        Some(whole as i64)
    })
}
