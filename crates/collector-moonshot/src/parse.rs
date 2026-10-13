//! Turns the balance document into facts. Every field is typed defensively; what cannot be trusted
//! is left out (unknown), never turned into a made-up number.

use serde_json::{Map, Value};
use usage_model::{ErrorCode, Fact, HarnessError};

/// A balance beyond this magnitude is not money.
const MAX_AMOUNT: f64 = 1e12;

fn endpoint_changed() -> HarnessError {
    HarnessError::new(ErrorCode::EndpointChanged, false)
}

/// `EndpointChanged` unless `data` is an object with a usable figure; a body that says it failed
/// (`status: false` or a non-zero `code`) is a bad response, not a balance.
pub(crate) fn parse_balance(body: &Value) -> Result<Vec<Fact>, HarnessError> {
    let root = body.as_object().ok_or_else(endpoint_changed)?;
    if root.get("status") == Some(&Value::Bool(false))
        || root.get("code").and_then(Value::as_i64).is_some_and(|code| code != 0)
    {
        return Err(HarnessError::new(ErrorCode::BadResponse, false));
    }
    let data = root.get("data").and_then(Value::as_object).ok_or_else(endpoint_changed)?;
    let facts = facts_of(data);
    if facts.is_empty() {
        return Err(endpoint_changed());
    }
    Ok(facts)
}

fn facts_of(data: &Map<String, Value>) -> Vec<Fact> {
    [
        ("balance", "available_balance"),
        ("voucher", "voucher_balance"),
        // Negative is real here: the documentation says the cash balance can mean money owed.
        ("cash", "cash_balance"),
    ]
    .into_iter()
    .filter_map(|(id, field)| {
        let number = data.get(field)?.as_f64()?;
        (number.is_finite() && number.abs() <= MAX_AMOUNT).then(|| Fact {
            id: id.to_owned(),
            number: Some(number),
            text: None,
            unit: Some("usd".to_owned()),
        })
    })
    .collect()
}
