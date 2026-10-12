//! Turns the body of the Go usage endpoint into windows. Every field is optional and typed
//! defensively; what cannot be trusted becomes unknown, never a clamped, valid-looking number.

use serde_json::{Map, Value};
use usage_model::{ErrorCode, HarnessError, Notice, Severity, UsageWindow, WindowKind};
use usage_support::parse_rfc3339_secs;

use super::month::month_minutes_ending_at;
use crate::notice;

/// Slack between our clock and the service's when deciding that a reset is a full window away.
const IDLE_TOLERANCE_SECS: i64 = 120;
/// No window resets more than a month out; a farther date is garbage, not a reset time.
const MAX_RESET_AHEAD_SECS: i64 = 366 * 86_400;

struct Spec {
    key: &'static str,
    kind: WindowKind,
    /// `None` when the length is not fixed by contract (monthly: computed from the reset).
    minutes: Option<u32>,
}

/// Order is display order; the key doubles as the window id.
const SPECS: [Spec; 3] = [
    Spec { key: "rolling", kind: WindowKind::FiveHour, minutes: Some(300) },
    Spec { key: "weekly", kind: WindowKind::Weekly, minutes: Some(10_080) },
    Spec { key: "monthly", kind: WindowKind::Monthly, minutes: None },
];

#[derive(Debug)]
pub(crate) struct Parsed {
    pub windows: Vec<UsageWindow>,
    pub notices: Vec<Notice>,
}

fn endpoint_changed() -> HarnessError {
    HarnessError::new(ErrorCode::EndpointChanged, false)
}

/// `EndpointChanged` unless at least one window carries a usable used percentage: a body whose
/// windows all lack one has the shape of a different API, and "unknown" three times says nothing.
pub(crate) fn parse_usage(body: &Value, now_secs: i64) -> Result<Parsed, HarnessError> {
    let usage = body.get("usage").and_then(Value::as_object).ok_or_else(endpoint_changed)?;
    let mut parsed = Parsed { windows: Vec::new(), notices: Vec::new() };
    let mut idle = Vec::new();
    for spec in &SPECS {
        let Some(raw) = usage.get(spec.key).and_then(Value::as_object) else { continue };
        let Some(read) = read_window(raw, spec, now_secs) else { continue };
        if read.rate_limited {
            parsed.notices.push(window_notice(notice::RATE_LIMITED, Severity::Warn, spec.key));
        }
        if read.idle {
            idle.push(window_notice(notice::WINDOW_IDLE, Severity::Info, spec.key));
        }
        parsed.windows.push(read.window);
    }
    parsed.notices.extend(idle);
    if parsed.windows.iter().all(|window| window.used_pct.is_none()) {
        return Err(endpoint_changed());
    }
    Ok(parsed)
}

fn window_notice(code: &str, severity: Severity, window: &str) -> Notice {
    notice::make(code, severity, &[("window", window)])
}

struct Read {
    window: UsageWindow,
    rate_limited: bool,
    idle: bool,
}

/// `None` when the object holds nothing recognizable: that window is unknown and is not emitted.
fn read_window(raw: &Map<String, Value>, spec: &Spec, now_secs: i64) -> Option<Read> {
    let rate_limited = raw
        .get("status")
        .and_then(Value::as_str)
        .is_some_and(|s| s.eq_ignore_ascii_case("rate-limited"));
    let used_pct = if rate_limited { Some(100.0) } else { percent(raw.get("percent")) };
    let resets_at = reset(raw.get("resetsAt"), now_secs);
    if used_pct.is_none() && resets_at.is_none() {
        return None;
    }
    let minutes = spec.minutes.or_else(|| resets_at.and_then(month_minutes_ending_at));
    let mut window = UsageWindow::new(spec.key, spec.kind);
    window.window_minutes = minutes;
    window.used_pct = used_pct;
    window.resets_at = resets_at;
    let idle = is_idle(used_pct, resets_at, minutes, now_secs);
    Some(Read { window, rate_limited, idle })
}

/// The service reports the used share as a floor-rounded 0..=100 number; anything else is unknown.
fn percent(value: Option<&Value>) -> Option<f64> {
    let number = value?.as_f64()?;
    // `abs` turns a JSON `-0.0` into a plain zero so it never serializes as "-0.0".
    (number.is_finite() && (0.0..=100.0).contains(&number)).then(|| number.floor().abs())
}

fn reset(value: Option<&Value>, now_secs: i64) -> Option<i64> {
    let secs = parse_rfc3339_secs(value?.as_str()?)?;
    (0..=now_secs.saturating_add(MAX_RESET_AHEAD_SECS)).contains(&secs).then_some(secs)
}

/// 0 % used with the reset a whole window away: the window has not started, it starts on the
/// next request.
fn is_idle(used: Option<f64>, resets_at: Option<i64>, minutes: Option<u32>, now_secs: i64) -> bool {
    let (Some(used), Some(resets_at), Some(minutes)) = (used, resets_at, minutes) else {
        return false;
    };
    let length_secs = i64::from(minutes) * 60;
    used <= 0.0 && (resets_at - now_secs - length_secs).abs() <= IDLE_TOLERANCE_SECS
}
