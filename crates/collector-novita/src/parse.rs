//! Turns the balance document into facts. Every field is typed defensively; what cannot be trusted
//! is left out (unknown), never turned into a made-up number.

use serde_json::{Map, Value};
use usage_model::{ErrorCode, Fact, HarnessError};

/// The API states every amount in units of 1/10000 US dollar.
const UNITS_PER_DOLLAR: f64 = 10_000.0;
/// A balance beyond this magnitude (in dollars) is not money.
const MAX_DOLLARS: f64 = 1e9;

/// (fact id, response field, may be negative)
const FIELDS: [(&str, &str, bool); 5] = [
    ("balance", "availableBalance", true),
    ("cash", "cashBalance", true),
    ("credit_limit", "creditLimit", false),
    ("pending", "pendingCharges", false),
    ("owed", "outstandingInvoices", false),
];

fn endpoint_changed() -> HarnessError {
    HarnessError::new(ErrorCode::EndpointChanged, false)
}

/// `EndpointChanged` unless the document is an object with at least one usable figure.
pub(crate) fn parse_balance(body: &Value) -> Result<Vec<Fact>, HarnessError> {
    let root = body.as_object().ok_or_else(endpoint_changed)?;
    let facts = facts_of(root);
    if facts.is_empty() {
        return Err(endpoint_changed());
    }
    Ok(facts)
}

fn facts_of(root: &Map<String, Value>) -> Vec<Fact> {
    FIELDS
        .iter()
        .filter_map(|(id, field, signed)| {
            let dollars = units(root.get(*field)?)? / UNITS_PER_DOLLAR;
            let in_range = dollars.abs() <= MAX_DOLLARS && (*signed || dollars >= 0.0);
            in_range.then(|| Fact {
                id: (*id).to_owned(),
                number: Some(dollars),
                text: None,
                unit: Some("usd".to_owned()),
            })
        })
        .collect()
}

/// The documented form is a decimal string; a JSON number is tolerated, anything else is unknown.
fn units(value: &Value) -> Option<f64> {
    let number = match value {
        Value::String(text) => text.trim().parse::<f64>().ok()?,
        Value::Number(number) => number.as_f64()?,
        _ => return None,
    };
    number.is_finite().then_some(number)
}
