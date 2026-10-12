//! Turns the two documents into plain numbers. Every field is typed defensively; what cannot be
//! trusted is unknown (`None`), never a made-up number.

use serde_json::{Map, Value};
use usage_model::{ErrorCode, HarnessError};

/// An amount above this is not money.
const MAX_AMOUNT: f64 = 1e12;

#[derive(Debug, Default, PartialEq)]
pub(crate) struct KeyInfo {
    /// `None` is "no limit reported" (the documented `null` means unlimited).
    pub limit: Option<f64>,
    pub limit_remaining: Option<f64>,
    pub usage: Option<f64>,
    pub usage_daily: Option<f64>,
    pub usage_weekly: Option<f64>,
    pub usage_monthly: Option<f64>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct Credits {
    pub total: f64,
    pub used: f64,
}

fn endpoint_changed() -> HarnessError {
    HarnessError::new(ErrorCode::EndpointChanged, false)
}

fn data_of(body: &Value) -> Option<&Map<String, Value>> {
    body.as_object()?.get("data")?.as_object()
}

fn amount(data: &Map<String, Value>, field: &str) -> Option<f64> {
    let number = data.get(field)?.as_f64()?;
    (number.is_finite() && number.abs() <= MAX_AMOUNT).then_some(number)
}

fn non_negative(data: &Map<String, Value>, field: &str) -> Option<f64> {
    amount(data, field).filter(|number| *number >= 0.0)
}

/// `EndpointChanged` unless the document carries a positive limit or a usage figure.
pub(crate) fn parse_key(body: &Value) -> Result<KeyInfo, HarnessError> {
    let data = data_of(body).ok_or_else(endpoint_changed)?;
    let info = KeyInfo {
        limit: non_negative(data, "limit"),
        // Negative is real here: a key can overspend its cap.
        limit_remaining: amount(data, "limit_remaining"),
        usage: non_negative(data, "usage"),
        usage_daily: non_negative(data, "usage_daily"),
        usage_weekly: non_negative(data, "usage_weekly"),
        usage_monthly: non_negative(data, "usage_monthly"),
    };
    if info.limit.is_none_or(|limit| limit <= 0.0) && info.usage.is_none() {
        return Err(endpoint_changed());
    }
    Ok(info)
}

/// `None` when either figure is missing or not a plain amount: the balance stays unknown.
pub(crate) fn parse_credits(body: &Value) -> Option<Credits> {
    let data = data_of(body)?;
    Some(Credits {
        total: non_negative(data, "total_credits")?,
        used: non_negative(data, "total_usage")?,
    })
}
