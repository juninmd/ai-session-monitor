//! Turns the balance document into facts. Every field is typed defensively; what cannot be trusted
//! is left out (unknown), never turned into a made-up number.

use serde_json::{Map, Value};
use usage_model::{ErrorCode, Fact, HarnessError};

/// Currencies the documentation lists; any other code is not read.
const CURRENCIES: [&str; 2] = ["CNY", "USD"];
/// A balance above this is not money.
const MAX_AMOUNT: f64 = 1e12;
/// A body with more entries than this is not the document we know.
const MAX_INFOS: usize = 16;
const FIELDS: [(&str, &str); 3] = [
    ("total_balance", "balance"),
    ("granted_balance", "granted"),
    ("topped_up_balance", "topped_up"),
];

fn endpoint_changed() -> HarnessError {
    HarnessError::new(ErrorCode::EndpointChanged, false)
}

/// `EndpointChanged` unless `balance_infos` is a list and, when it has entries, one is usable.
pub(crate) fn parse_balance(body: &Value) -> Result<Vec<Fact>, HarnessError> {
    let infos = body
        .as_object()
        .and_then(|root| root.get("balance_infos"))
        .and_then(Value::as_array)
        .ok_or_else(endpoint_changed)?;
    let mut facts = Vec::new();
    let mut seen: Vec<&str> = Vec::new();
    for info in infos.iter().take(MAX_INFOS).filter_map(Value::as_object) {
        let Some(currency) = currency_of(info) else { continue };
        // Two entries claiming one currency is ambiguity, not data: the first one stands.
        if seen.contains(&currency) {
            continue;
        }
        seen.push(currency);
        facts.extend(facts_of(info, currency));
    }
    if !infos.is_empty() && facts.is_empty() {
        return Err(endpoint_changed());
    }
    Ok(facts)
}

fn currency_of(info: &Map<String, Value>) -> Option<&'static str> {
    let code = info.get("currency")?.as_str()?;
    CURRENCIES.iter().copied().find(|known| *known == code)
}

fn facts_of(info: &Map<String, Value>, currency: &str) -> Vec<Fact> {
    let unit = currency.to_ascii_lowercase();
    FIELDS
        .iter()
        .filter_map(|(field, id)| Some((id, amount(info.get(*field)?)?)))
        .map(|(id, number)| Fact {
            id: format!("{id}_{unit}"),
            number: Some(number),
            text: None,
            unit: Some(unit.clone()),
        })
        .collect()
}

/// The documented form is a decimal string; a JSON number is tolerated, anything else is unknown.
fn amount(value: &Value) -> Option<f64> {
    let number = match value {
        Value::String(text) => text.trim().parse::<f64>().ok()?,
        Value::Number(number) => number.as_f64()?,
        _ => return None,
    };
    (number.is_finite() && (0.0..=MAX_AMOUNT).contains(&number)).then_some(number)
}
