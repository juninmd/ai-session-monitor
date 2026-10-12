//! Turns the two documents into plain numbers. Every field is typed defensively; what cannot be
//! trusted is unknown (`None`), never a made-up number.

use serde_json::Value;
use usage_model::{ErrorCode, HarnessError};

/// An amount above this is not money.
const MAX_AMOUNT: f64 = 1e12;
const CENTS_PER_DOLLAR: f64 = 100.0;

#[derive(Debug, PartialEq)]
pub(crate) struct Checklist {
    /// Negative: funds ready to spend. Positive: money owed.
    pub stripe_balance: f64,
    pub recent: Option<f64>,
    /// `None` is "no limit reported" (the documented `null`, or a figure that is not a plain amount).
    pub limit: Option<f64>,
}

fn endpoint_changed() -> HarnessError {
    HarnessError::new(ErrorCode::EndpointChanged, false)
}

fn plain(value: Option<&Value>) -> Option<f64> {
    let number = value?.as_f64()?;
    (number.is_finite() && number.abs() <= MAX_AMOUNT).then_some(number)
}

/// `EndpointChanged` unless the document carries a usable `stripe_balance`.
pub(crate) fn parse_checklist(body: &Value) -> Result<Checklist, HarnessError> {
    let root = body.as_object().ok_or_else(endpoint_changed)?;
    let stripe_balance = plain(root.get("stripe_balance")).ok_or_else(endpoint_changed)?;
    Ok(Checklist {
        stripe_balance,
        recent: plain(root.get("recent")).filter(|recent| *recent >= 0.0),
        limit: plain(root.get("limit")).filter(|limit| *limit > 0.0),
    })
}

/// The current month's spend in dollars; `None` when the document is not the one we know.
pub(crate) fn parse_month_spend(body: &Value) -> Option<f64> {
    let months = body.as_object()?.get("months")?.as_array()?;
    let cents = plain(months.first()?.as_object()?.get("total_cost"))?;
    (cents >= 0.0).then_some(cents / CENTS_PER_DOLLAR)
}
