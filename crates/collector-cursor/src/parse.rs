//! Turns the body of `usage-summary` into windows and facts. The shape is community-observed and
//! unverified, so every field is optional, typed defensively and unknown fields are ignored; what
//! cannot be trusted becomes unknown, never a made-up number. The one clamp: a used share above
//! 100 % (overage) is shown as 100 %.

use serde_json::{Map, Value};
use usage_model::{Amount, AmountUnit, ErrorCode, Fact, HarnessError, UsageWindow, WindowKind};
use usage_support::parse_rfc3339_secs;

/// No cycle resets more than a year out; a farther date is garbage, not a reset time.
const MAX_RESET_AHEAD_SECS: i64 = 366 * 86_400;
/// A reset farther than the cycle's own length (plus this slack) is a clock or timezone fault.
const RESET_SLACK_SECS: i64 = 120;
const MAX_CYCLE_MINUTES: i64 = 366 * 1_440;
/// Amounts above this (cents, requests, percent) are not what the field means.
const MAX_COUNT: f64 = 1e12;
const MAX_PLAN_CHARS: usize = 24;

#[derive(Debug, PartialEq)]
pub(crate) struct Parsed {
    pub plan: Option<String>,
    pub windows: Vec<UsageWindow>,
    pub facts: Vec<Fact>,
}

fn endpoint_changed() -> HarnessError {
    HarnessError::new(ErrorCode::EndpointChanged, false)
}

/// `EndpointChanged` unless the body yields at least one window or fact: a body with none has the
/// shape of a different API, and "unknown" says nothing.
pub(crate) fn parse_summary(body: &Value, now_secs: i64) -> Result<Parsed, HarnessError> {
    let root = body.as_object().ok_or_else(endpoint_changed)?;
    let cycle = Cycle::of(root, now_secs);
    let individual = root.get("individualUsage").and_then(Value::as_object);
    let mut windows = Vec::new();
    let mut facts = Vec::new();

    if root.get("isUnlimited").and_then(Value::as_bool) == Some(true) {
        facts.push(Fact {
            id: "unlimited".to_owned(),
            number: None,
            text: Some("true".to_owned()),
            unit: None,
        });
    } else {
        let plan = individual.and_then(|usage| usage.get("plan")).and_then(Value::as_object);
        windows.extend(plan.and_then(|raw| included(raw, &cycle)));
    }
    let on_demand = individual.and_then(|usage| usage.get("onDemand")).and_then(Value::as_object);
    if let Some(raw) = on_demand {
        let (window, fact) = spend(raw, &cycle);
        windows.extend(window);
        facts.extend(fact);
    }
    if windows.is_empty() && facts.is_empty() {
        return Err(endpoint_changed());
    }
    Ok(Parsed {
        plan: root.get("membershipType").and_then(Value::as_str).and_then(plan_label),
        windows,
        facts,
    })
}

/// The billing cycle both windows share.
struct Cycle {
    resets_at: Option<i64>,
    minutes: Option<u32>,
}

impl Cycle {
    fn of(root: &Map<String, Value>, now_secs: i64) -> Self {
        let date = |key: &str| root.get(key).and_then(Value::as_str).and_then(parse_rfc3339_secs);
        let (start, end) = (date("billingCycleStart"), date("billingCycleEnd"));
        let minutes = start
            .zip(end)
            .map(|(start, end)| (end - start) / 60)
            .filter(|minutes| (1..=MAX_CYCLE_MINUTES).contains(minutes))
            .and_then(|minutes| u32::try_from(minutes).ok());
        let ceiling =
            minutes.map_or(MAX_RESET_AHEAD_SECS, |m| i64::from(m) * 60 + RESET_SLACK_SECS);
        let resets_at = end.filter(|end| (0..=now_secs.saturating_add(ceiling)).contains(end));
        Self { resets_at, minutes }
    }

    fn window(&self, id: &str, kind: WindowKind) -> UsageWindow {
        let mut window = UsageWindow::new(id, kind);
        window.window_minutes = self.minutes;
        window.resets_at = self.resets_at;
        window
    }
}

/// `individualUsage.plan`: the included allowance. The server's own `totalPercentUsed` wins; else
/// `used / limit`. A share nothing states is unknown, and a window with neither a share nor a reset
/// time holds nothing, so it is not made.
fn included(raw: &Map<String, Value>, cycle: &Cycle) -> Option<UsageWindow> {
    let limit = count(raw.get("limit")).filter(|limit| *limit > 0.0);
    let from_ratio = || Some(count(raw.get("used"))? / limit?);
    let pct = count(raw.get("totalPercentUsed"))
        .or_else(|| from_ratio().map(|ratio| ratio * 100.0))
        .filter(|pct| pct.is_finite())
        .map(|pct| pct.min(100.0));
    let mut window = cycle.window("monthly", WindowKind::Monthly);
    window.used_pct = pct;
    (window.used_pct.is_some() || window.resets_at.is_some()).then_some(window)
}

/// `individualUsage.onDemand`: spend beyond the plan, in cents. A window only when there is a
/// real cap; spend without a cap is a fact, and a switched-off or empty block is nothing.
fn spend(raw: &Map<String, Value>, cycle: &Cycle) -> (Option<UsageWindow>, Option<Fact>) {
    if raw.get("enabled").and_then(Value::as_bool) == Some(false) {
        return (None, None);
    }
    let limit = count(raw.get("limit")).filter(|limit| *limit > 0.0);
    let used = count(raw.get("used")).or_else(|| {
        let left = count(raw.get("remaining"))?;
        Some(limit? - left).filter(|used| *used >= 0.0)
    });
    let usd = |cents: f64| cents / 100.0;
    let Some(limit) = limit else {
        let fact = used.filter(|used| *used > 0.0).map(|used| Fact {
            id: "on_demand_spend".to_owned(),
            number: Some(usd(used)),
            text: None,
            unit: Some("usd".to_owned()),
        });
        return (None, fact);
    };
    let mut window = cycle.window("on_demand", WindowKind::Budget);
    if let Some(used) = used {
        window.used_pct = Some((used / limit * 100.0).min(100.0));
        window.amount =
            Some(Amount { used: usd(used), limit: Some(usd(limit)), unit: AmountUnit::Usd });
    }
    (Some(window), None)
}

/// A non-negative number or a string holding one.
fn count(value: Option<&Value>) -> Option<f64> {
    let number = match value? {
        Value::Number(number) => number.as_f64()?,
        Value::String(text) => text.trim().parse::<f64>().ok()?,
        _ => return None,
    };
    (number.is_finite() && (0.0..=MAX_COUNT).contains(&number)).then_some(number)
}

/// `membershipType` (`pro_plus`) as a display label (`Pro Plus`). Control characters are stripped;
/// then only a short plain token (letters, digits, `_`, `-`, space) is shown, title-cased. Anything
/// else is no label at all: no other server text reaches the view.
fn plan_label(raw: &str) -> Option<String> {
    let clean: String = raw.chars().filter(|c| !c.is_control()).collect();
    let clean = clean.trim();
    let plain = !clean.is_empty()
        && clean.chars().count() <= MAX_PLAN_CHARS
        && clean.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b' '));
    if !plain {
        return None;
    }
    let words: Vec<String> = clean
        .split(['_', '-', ' '])
        .filter(|word| !word.is_empty())
        .map(|word| {
            let (head, tail) = word.split_at(1);
            format!("{}{}", head.to_ascii_uppercase(), tail.to_ascii_lowercase())
        })
        .collect();
    (!words.is_empty()).then(|| words.join(" "))
}
