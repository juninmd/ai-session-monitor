//! Turns the balance document into windows and facts. The shape varies by plan (monthly credits,
//! or session plus weekly percentages), so every field is typed defensively; what cannot be
//! trusted is unknown (`None`), never a made-up number.

use serde_json::{Map, Value};
use usage_model::{Amount, AmountUnit, ErrorCode, Fact, HarnessError, UsageWindow, WindowKind};
use usage_support::parse_rfc3339_secs;

/// A balance beyond this magnitude (in dollars) is not money.
const MAX_DOLLARS: f64 = 1e9;

#[derive(Debug, Default, PartialEq)]
pub(crate) struct Balance {
    pub windows: Vec<UsageWindow>,
    pub facts: Vec<Fact>,
}

fn endpoint_changed() -> HarnessError {
    HarnessError::new(ErrorCode::EndpointChanged, false)
}

/// `EndpointChanged` unless the document is an object whose `included` or `purchased` part yields
/// at least one window or fact.
pub(crate) fn parse_balance(body: &Value) -> Result<Balance, HarnessError> {
    let root = body.as_object().ok_or_else(endpoint_changed)?;
    let mut balance = Balance::default();
    if let Some(included) = root.get("included").and_then(Value::as_object) {
        read_included(included, &mut balance);
    }
    let purchased = root.get("purchased").and_then(Value::as_object);
    balance.facts.extend(
        purchased
            .and_then(|part| dollars(part, "balance_usd"))
            .map(|usd| fact("purchased_balance", usd)),
    );
    if balance.windows.is_empty() && balance.facts.is_empty() {
        return Err(endpoint_changed());
    }
    Ok(balance)
}

fn read_included(included: &Map<String, Value>, out: &mut Balance) {
    // Current plans: a monthly credit allowance with its own period.
    let left = dollars(included, "balance_usd");
    out.facts.extend(left.map(|usd| fact("included_balance", usd)));
    out.windows.extend(monthly_window(included, left));
    // Legacy plans: two percentages with a reset moment each.
    for (field, id, kind) in
        [("session", "session", WindowKind::Other), ("weekly", "weekly", WindowKind::Weekly)]
    {
        let part = included.get(field).and_then(Value::as_object);
        out.windows.extend(part.and_then(|part| percent_window(part, id, kind)));
    }
}

fn fact(id: &str, usd: f64) -> Fact {
    Fact { id: id.to_owned(), number: Some(usd), text: None, unit: Some("usd".to_owned()) }
}

fn dollars(part: &Map<String, Value>, field: &str) -> Option<f64> {
    let number = part.get(field)?.as_f64()?;
    (number.is_finite() && (0.0..=MAX_DOLLARS).contains(&number)).then_some(number)
}

fn monthly_window(included: &Map<String, Value>, left: Option<f64>) -> Option<UsageWindow> {
    let allowance = dollars(included, "allowance_usd").filter(|allowance| *allowance > 0.0)?;
    let used = (allowance - left?).max(0.0);
    let mut window = UsageWindow::new("monthly", WindowKind::Monthly);
    window.used_pct = Some((used * 100.0 / allowance).clamp(0.0, 100.0));
    window.amount = Some(Amount { used, limit: Some(allowance), unit: AmountUnit::Usd });
    let period = included.get("period").and_then(Value::as_object);
    let from =
        period.and_then(|p| p.get("from")).and_then(Value::as_str).and_then(parse_rfc3339_secs);
    let until =
        period.and_then(|p| p.get("until")).and_then(Value::as_str).and_then(parse_rfc3339_secs);
    window.resets_at = until;
    window.window_minutes = from
        .zip(until)
        .and_then(|(from, until)| u32::try_from((until - from) / 60).ok())
        .filter(|minutes| *minutes > 0);
    Some(window)
}

/// A window exists when it states a usage figure or a reset moment; a figure it does not state
/// stays unknown.
fn percent_window(part: &Map<String, Value>, id: &str, kind: WindowKind) -> Option<UsageWindow> {
    let mut window = UsageWindow::new(id, kind);
    window.used_pct = used_pct(part);
    window.resets_at = part.get("resets_at").and_then(Value::as_str).and_then(parse_rfc3339_secs);
    (window.used_pct.is_some() || window.resets_at.is_some()).then_some(window)
}

/// `remaining_percent` is the documented field (0..100). `used_percent` and a `usage` fraction
/// (0..1) are tolerated; every figure is clamped into range and a non-number is unknown.
fn used_pct(part: &Map<String, Value>) -> Option<f64> {
    let number = |field: &str| part.get(field)?.as_f64().filter(|n| n.is_finite());
    let pct = number("remaining_percent")
        .map(|remaining| 100.0 - remaining)
        .or_else(|| number("used_percent"))
        .or_else(|| number("usage").map(|fraction| fraction.clamp(0.0, 1.0) * 100.0))?;
    Some(pct.clamp(0.0, 100.0))
}
