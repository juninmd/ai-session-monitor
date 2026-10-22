//! Turns the body of `copilot_internal/user` into windows and facts. The shape is community
//! observed and unverified, so every field is optional and typed defensively; what cannot be
//! trusted becomes unknown, never a made-up number. Counts may be numbers or numeric strings.

use serde_json::{Map, Value};
use usage_model::{Amount, AmountUnit, ErrorCode, Fact, HarnessError, UsageWindow, WindowKind};

use crate::calendar::{next_month_start, parse_reset};

/// Counts above this are not request counts.
const MAX_COUNT: f64 = 1e12;
/// A percentage further out than this is garbage, not an overage.
const MAX_PERCENT_ABS: f64 = 1_000.0;
/// A monthly reset farther away than this (a month and a few days of slack) is a clock or format
/// fault: the derived first-of-next-month is used instead.
const MAX_RESET_AHEAD_SECS: i64 = 35 * 86_400;
const MAX_PLAN_LEN: usize = 32;

/// `(key in quota_snapshots, window id)`. Anything else in the object is ignored.
const QUOTAS: [(&str, &str); 3] = [
    ("premium_interactions", "premium_requests"),
    ("chat", "chat"),
    ("completions", "completions"),
];

#[derive(Debug)]
pub(crate) struct Parsed {
    pub plan: Option<String>,
    pub windows: Vec<UsageWindow>,
    pub facts: Vec<Fact>,
    /// True when at least one window carries the derived first-of-next-month reset.
    pub reset_derived: bool,
}

fn endpoint_changed() -> HarnessError {
    HarnessError::new(ErrorCode::EndpointChanged, false)
}

/// `EndpointChanged` unless the body has a `quota_snapshots` object that yields at least one
/// window or fact: any other body has the shape of a different API, and "unknown" says nothing.
pub(crate) fn parse_usage(body: &Value, now_secs: i64) -> Result<Parsed, HarnessError> {
    let root = body.as_object().ok_or_else(endpoint_changed)?;
    let snapshots =
        root.get("quota_snapshots").and_then(Value::as_object).ok_or_else(endpoint_changed)?;
    let reported = reported_reset(root, now_secs);
    let reset = reported.unwrap_or_else(|| next_month_start(now_secs));

    let (mut windows, mut facts) = (Vec::new(), Vec::new());
    for (key, id) in QUOTAS {
        let Some(raw) = snapshots.get(key).and_then(Value::as_object) else { continue };
        if flag(raw.get("unlimited")) {
            facts.push(Fact {
                id: format!("{id}_unlimited"),
                number: None,
                text: Some("unlimited".to_owned()),
                unit: None,
            });
            continue;
        }
        let quota = Quota::of(raw);
        windows.extend(quota.window(id, reset));
        facts.extend(quota.overage.filter(|count| *count > 0.0).map(|count| Fact {
            id: format!("{id}_overage"),
            number: Some(count),
            text: None,
            unit: Some("requests".to_owned()),
        }));
    }
    if windows.is_empty() && facts.is_empty() {
        return Err(endpoint_changed());
    }
    Ok(Parsed {
        plan: plan_of(root),
        reset_derived: reported.is_none() && !windows.is_empty(),
        windows,
        facts,
    })
}

/// One quota snapshot, every number optional.
struct Quota {
    entitlement: Option<f64>,
    remaining: Option<f64>,
    percent_remaining: Option<f64>,
    overage: Option<f64>,
}

impl Quota {
    fn of(raw: &Map<String, Value>) -> Self {
        Self {
            entitlement: number(raw.get("entitlement"), 0.0).filter(|limit| *limit > 0.0),
            // Negative when the quota is overdrawn; the used count keeps the real figure.
            remaining: number(raw.get("remaining"), -MAX_COUNT),
            percent_remaining: number(raw.get("percent_remaining"), -MAX_PERCENT_ABS)
                .filter(|percent| *percent <= MAX_PERCENT_ABS),
            overage: number(raw.get("overage_count"), 0.0),
        }
    }

    /// The reported share wins (it carries decimals); counts are the fallback. Clamped to 0..=100:
    /// an overdrawn quota is 100 %, while `amount.used` keeps the real count.
    fn used_pct(&self) -> Option<f64> {
        let share = match (self.percent_remaining, self.entitlement, self.remaining) {
            (Some(percent), _, _) => 100.0 - percent,
            (None, Some(limit), Some(left)) => (limit - left) / limit * 100.0,
            _ => return None,
        };
        Some(share.clamp(0.0, 100.0))
    }

    /// `None` when nothing usable is known: a missing or zero entitlement with no percentage is
    /// "no such quota", never 0 %.
    fn window(&self, id: &str, reset: i64) -> Option<UsageWindow> {
        let used_pct = self.used_pct()?;
        let mut window = UsageWindow::new(id, WindowKind::Monthly);
        window.used_pct = Some(used_pct);
        window.resets_at = Some(reset);
        if let (Some(limit), Some(left)) = (self.entitlement, self.remaining) {
            let used = (limit - left).max(0.0);
            window.amount = Some(Amount { used, limit: Some(limit), unit: AmountUnit::Requests });
        }
        Some(window)
    }
}

/// `quota_reset_date_utc` (ISO) then `quota_reset_date` ("2026-11-01"). A date already past, or
/// more than a month and a few days out, is not trusted: the caller derives the reset instead.
fn reported_reset(root: &Map<String, Value>, now_secs: i64) -> Option<i64> {
    ["quota_reset_date_utc", "quota_reset_date"].iter().find_map(|key| {
        let secs = parse_reset(root.get(*key)?.as_str()?)?;
        (now_secs < secs && secs <= now_secs.saturating_add(MAX_RESET_AHEAD_SECS)).then_some(secs)
    })
}

/// A finite number, or a string holding one, within `min..=MAX_COUNT`.
fn number(value: Option<&Value>, min: f64) -> Option<f64> {
    let number = match value? {
        Value::Number(number) => number.as_f64()?,
        Value::String(text) => text.trim().parse::<f64>().ok()?,
        _ => return None,
    };
    (number.is_finite() && (min..=MAX_COUNT).contains(&number)).then_some(number)
}

fn flag(value: Option<&Value>) -> bool {
    match value {
        Some(Value::Bool(flag)) => *flag,
        Some(Value::String(text)) => text.trim().eq_ignore_ascii_case("true"),
        _ => false,
    }
}

/// `copilot_plan` is a wire token such as `individual_pro`. Only plain tokens are shown
/// (letters, digits, `_`, `-`, space; no control or markup characters), capped and prettified;
/// anything else is unknown rather than sanitized into something believable.
fn plan_of(root: &Map<String, Value>) -> Option<String> {
    let raw = root.get("copilot_plan")?.as_str()?.trim();
    let plain = !raw.is_empty()
        && raw.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | ' '));
    if !plain {
        return None;
    }
    let words: Vec<String> = raw
        .chars()
        .take(MAX_PLAN_LEN)
        .collect::<String>()
        .split(['_', '-', ' '])
        .filter(|word| !word.is_empty())
        .map(|word| format!("{}{}", word[..1].to_ascii_uppercase(), word[1..].to_ascii_lowercase()))
        .collect();
    (!words.is_empty()).then(|| words.join(" "))
}
