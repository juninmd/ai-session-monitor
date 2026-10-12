//! Codex's `RateLimitSnapshot`, read from either spelling: the app-server speaks camelCase
//! (`usedPercent`), rollout files snake_case (`used_percent`). Every field is optional, unknown
//! ones are ignored, and a bad value becomes unknown instead of a clamped number that looks valid.

mod value;
mod window;

#[cfg(test)]
mod tests;

use serde_json::{Map, Value};
use usage_model::UsageWindow;

use crate::plan;
use value::{field, number};
use window::Slot;

const MAX_CREDITS: f64 = 1.0e12;

#[derive(Debug, PartialEq)]
pub(crate) struct Limits {
    pub windows: Vec<UsageWindow>,
    pub plan: Option<String>,
    /// Balance of extra credits, when the account has some.
    pub credits: Option<f64>,
}

/// `None` when `snapshot` is not an object at all; an object with nothing usable is valid and
/// yields no windows.
pub(crate) fn parse_snapshot(snapshot: &Value) -> Option<Limits> {
    let snapshot = snapshot.as_object()?;
    let mut windows: Vec<UsageWindow> = Vec::new();
    for (key, slot) in [("primary", Slot::Primary), ("secondary", Slot::Secondary)] {
        let Some(raw) = snapshot.get(key).and_then(window::read) else { continue };
        let built = window::build(&raw, slot, &windows);
        windows.push(built);
    }
    let plan_name =
        field(snapshot, &["planType", "plan_type"]).and_then(Value::as_str).and_then(plan::label);
    Some(Limits { windows, plan: plan_name, credits: credits(snapshot) })
}

/// The extra-credits balance, only when the account has a finite one to show.
fn credits(snapshot: &Map<String, Value>) -> Option<f64> {
    let credits = field(snapshot, &["credits"])?.as_object()?;
    let unlimited = field(credits, &["unlimited"]).and_then(Value::as_bool) == Some(true);
    let has_none =
        field(credits, &["hasCredits", "has_credits"]).and_then(Value::as_bool) == Some(false);
    if unlimited || has_none {
        return None;
    }
    let balance = match field(credits, &["balance"])? {
        Value::String(text) => text.trim().parse::<f64>().ok(),
        other => number(Some(other)),
    }?;
    (0.0..=MAX_CREDITS).contains(&balance).then_some(balance)
}
