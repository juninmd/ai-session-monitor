//! Turns the body of the quota endpoint into windows. Every field is optional and typed
//! defensively; what cannot be trusted becomes unknown, never a made-up number. The one clamp: a used share
//! computed from counts above the limit (overage) is shown as 100 %, while `amount.used` keeps
//! the real count.

use serde_json::{Map, Value};
use usage_model::{Amount, AmountUnit, ErrorCode, HarnessError, UsageWindow, WindowKind};

/// No window resets more than a year out; a farther date is garbage, not a reset time.
const MAX_RESET_AHEAD_SECS: i64 = 366 * 86_400;
/// A reset farther than the window's own length (plus this slack) is a clock or timezone fault.
const RESET_SLACK_SECS: i64 = 120;
/// Counts above this are not quota counts.
const MAX_COUNT: f64 = 1e15;
/// A body with more entries than this is not the document we know.
const MAX_LIMITS: usize = 16;
const MAX_MINUTES: u64 = 366 * 1_440;

#[derive(Debug)]
pub(crate) struct Parsed {
    pub plan: Option<String>,
    pub windows: Vec<UsageWindow>,
}

fn endpoint_changed() -> HarnessError {
    HarnessError::new(ErrorCode::EndpointChanged, false)
}

/// `EndpointChanged` unless at least one window carries a usable used percentage.
pub(crate) fn parse_quota(body: &Value, now_secs: i64) -> Result<Parsed, HarnessError> {
    let root = body.as_object().ok_or_else(endpoint_changed)?;
    envelope_check(root)?;
    let data = root.get("data").and_then(Value::as_object).ok_or_else(endpoint_changed)?;
    let limits = data.get("limits").and_then(Value::as_array).ok_or_else(endpoint_changed)?;
    // A key without a Coding Plan answers with an empty list: there is nothing to read.
    if limits.is_empty() {
        return Err(HarnessError::new(ErrorCode::UnsupportedPlan, false));
    }
    let mut windows: Vec<UsageWindow> = Vec::new();
    for entry in limits.iter().take(MAX_LIMITS).filter_map(Value::as_object) {
        let Some(window) = window_of(entry, now_secs) else { continue };
        // Two entries claiming one slot is ambiguity, not data: the first one stands.
        if !windows.iter().any(|seen| seen.id == window.id) {
            windows.push(window);
        }
    }
    if windows.iter().all(|window| window.used_pct.is_none()) {
        return Err(endpoint_changed());
    }
    Ok(Parsed { plan: plan_of(data), windows })
}

/// The service reports failure inside a 200: `success: false` and a `code`. Mapped like the HTTP
/// status of the same number; the `msg` text is vendor prose and is never read.
fn envelope_check(root: &Map<String, Value>) -> Result<(), HarnessError> {
    let code = root.get("code").and_then(Value::as_i64);
    let failed = root.get("success").and_then(Value::as_bool) == Some(false)
        || code.is_some_and(|code| code != 0 && code != 200);
    if !failed {
        return Ok(());
    }
    Err(match code {
        // 1000/1001/1003 are the documented auth failures (docs.z.ai/api-reference/api-code).
        Some(401 | 403 | 1000 | 1001 | 1003) => {
            HarnessError::new(ErrorCode::Unauthorized, false).with_param("ambiguous", "true")
        }
        Some(429) => HarnessError::new(ErrorCode::RateLimited, true),
        Some(500..=599) => HarnessError::new(ErrorCode::ServerError, true),
        _ => endpoint_changed(),
    })
}

/// A `TOKENS_LIMIT`/`CREDIT_LIMIT` entry is a rolling window whose length is `number * unit`; a
/// `TIME_LIMIT` entry is the monthly MCP tool allowance. Anything else is not read.
fn window_of(entry: &Map<String, Value>, now_secs: i64) -> Option<UsageWindow> {
    let kind = entry.get("type")?.as_str()?;
    let (mut window, minutes, unit) = match kind {
        "TOKENS_LIMIT" | "CREDIT_LIMIT" => {
            let minutes = minutes_of(entry)?;
            let (id, window_kind) = classify(minutes);
            let unit =
                if kind == "TOKENS_LIMIT" { AmountUnit::Tokens } else { AmountUnit::Credits };
            (UsageWindow::new(id, window_kind), Some(minutes), unit)
        }
        "TIME_LIMIT" => (UsageWindow::new("mcp", WindowKind::Monthly), None, AmountUnit::Requests),
        _ => return None,
    };
    window.window_minutes = minutes;
    window.resets_at = reset(entry.get("nextResetTime"), minutes, now_secs);
    let counts = counts_of(entry);
    window.used_pct = percent(entry.get("percentage"))
        .or_else(|| counts.map(|(used, limit)| (used / limit * 100.0).min(100.0)));
    window.amount = counts.map(|(used, limit)| Amount { used, limit: Some(limit), unit });
    (window.used_pct.is_some() || window.resets_at.is_some()).then_some(window)
}

fn classify(minutes: u32) -> (String, WindowKind) {
    match minutes {
        300 => ("five_hour".to_owned(), WindowKind::FiveHour),
        1_440 => ("daily".to_owned(), WindowKind::Daily),
        10_080 => ("weekly".to_owned(), WindowKind::Weekly),
        other => (format!("window_{other}m"), WindowKind::Other),
    }
}

/// Length in minutes from `number` and the unit code (1 day, 3 hour, 5 minute, 6 week). The
/// codes are undocumented; they are the ones the vendor's client matches on, and an unknown one is
/// no length at all.
fn minutes_of(entry: &Map<String, Value>) -> Option<u32> {
    let number = whole(entry.get("number"))?;
    let factor: u64 = match whole(entry.get("unit"))? {
        1 => 1_440,
        3 => 60,
        5 => 1,
        6 => 10_080,
        _ => return None,
    };
    let minutes =
        number.checked_mul(factor).filter(|minutes| (1..=MAX_MINUTES).contains(minutes))?;
    u32::try_from(minutes).ok()
}

fn whole(value: Option<&Value>) -> Option<u64> {
    value?.as_u64()
}

/// The service reports the used share as a 0..=100 number; anything else is unknown.
fn percent(value: Option<&Value>) -> Option<f64> {
    let number = value?.as_f64()?;
    (number.is_finite() && (0.0..=100.0).contains(&number)).then_some(number)
}

/// `(used, limit)` from `currentValue` and `usage` when the entry carries a positive cap.
fn counts_of(entry: &Map<String, Value>) -> Option<(f64, f64)> {
    let in_range = |value: f64| value.is_finite() && (0.0..=MAX_COUNT).contains(&value);
    let limit = entry.get("usage")?.as_f64().filter(|limit| in_range(*limit) && *limit > 0.0)?;
    let used = entry.get("currentValue")?.as_f64().filter(|used| in_range(*used))?;
    Some((used, limit))
}

/// `nextResetTime` is epoch milliseconds; `0` and `null` mean none. A reset farther away than the
/// window is long cannot be right, so it is unknown rather than shown.
fn reset(value: Option<&Value>, minutes: Option<u32>, now_secs: i64) -> Option<i64> {
    let millis = value?.as_i64().filter(|millis| *millis > 0)?;
    let secs = millis / 1_000;
    let ceiling = minutes.map_or(MAX_RESET_AHEAD_SECS, |m| i64::from(m) * 60 + RESET_SLACK_SECS);
    (0..=now_secs.saturating_add(ceiling)).contains(&secs).then_some(secs)
}

/// `data.level` (`lite`, `pro`, `max`) as a plain token; anything else is not shown.
fn plan_of(data: &Map<String, Value>) -> Option<String> {
    let level = data.get("level")?.as_str()?;
    let plain = !level.is_empty()
        && level.len() <= 16
        && level.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-');
    plain.then(|| format!("{}{}", level[..1].to_ascii_uppercase(), level[1..].to_ascii_lowercase()))
}
