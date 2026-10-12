//! Turns the body of the usages endpoint into windows. Every field is optional and typed
//! defensively; what cannot be trusted becomes unknown, never a made-up number. The one clamp: a used share
//! above the limit (overage) is shown as 100 %, while `amount.used` keeps the real count.

use serde_json::{Map, Value};
use usage_model::{Amount, AmountUnit, ErrorCode, HarnessError, UsageWindow, WindowKind};
use usage_support::parse_rfc3339_secs;

/// No window resets more than a year out; a farther date is garbage, not a reset time.
const MAX_RESET_AHEAD_SECS: i64 = 366 * 86_400;
/// A reset farther than the window's own length (plus this slack) is a clock or timezone fault.
const RESET_SLACK_SECS: i64 = 120;
/// Counts above this are not request counts.
const MAX_COUNT: f64 = 1e12;
/// A body with more rolling windows than this is not the document we know.
const MAX_LIMITS: usize = 8;
const MAX_MINUTES: u64 = 366 * 1_440;
const RESET_KEYS: [&str; 4] = ["resetTime", "reset_time", "resetAt", "reset_at"];

#[derive(Debug)]
pub(crate) struct Parsed {
    pub plan: Option<String>,
    pub windows: Vec<UsageWindow>,
}

fn endpoint_changed() -> HarnessError {
    HarnessError::new(ErrorCode::EndpointChanged, false)
}

/// `EndpointChanged` unless at least one window carries a usable used percentage: a body whose
/// windows all lack one has the shape of a different API, and "unknown" says nothing.
pub(crate) fn parse_usage(body: &Value, now_secs: i64) -> Result<Parsed, HarnessError> {
    let root = body.as_object().ok_or_else(endpoint_changed)?;
    let ratios = root.get("usages").and_then(Value::as_object);
    let mut windows = rolling_windows(root.get("limits"), now_secs);
    fill_five_hour_from_ratio(&mut windows, ratios, now_secs);
    let weekly = root.get("usage").and_then(Value::as_object).and_then(|raw| {
        counted(raw, UsageWindow::new("weekly", WindowKind::Weekly), None, now_secs)
    });
    fold_weekly(&mut windows, weekly);
    let monthly = ratios.and_then(|map| ratio_window(map, "limit_month_total", None, now_secs));
    windows.extend(monthly.map(|window| with_kind(window, "monthly", WindowKind::Monthly)));
    if windows.iter().all(|window| window.used_pct.is_none()) {
        return Err(endpoint_changed());
    }
    Ok(Parsed { plan: plan_of(root), windows })
}

/// `limits[]`: rolling windows whose length the body states as `{duration, timeUnit}`.
fn rolling_windows(limits: Option<&Value>, now_secs: i64) -> Vec<UsageWindow> {
    let mut windows: Vec<UsageWindow> = Vec::new();
    for item in limits.and_then(Value::as_array).into_iter().flatten().take(MAX_LIMITS) {
        let Some(item) = item.as_object() else { continue };
        let Some(minutes) = item.get("window").and_then(Value::as_object).and_then(minutes_of)
        else {
            continue;
        };
        let (id, kind) = classify(minutes);
        if windows.iter().any(|window| window.id == id) {
            continue;
        }
        let detail = item.get("detail").and_then(Value::as_object);
        let base = UsageWindow::new(id, kind);
        windows.extend(detail.and_then(|raw| counted(raw, base, Some(minutes), now_secs)));
    }
    windows
}

/// A 10080-minute `limits[]` entry and the top-level block describe one week: one window, the
/// one that states a reset (the rolling entry when both or neither do).
fn fold_weekly(windows: &mut Vec<UsageWindow>, top: Option<UsageWindow>) {
    let Some(top) = top else { return };
    match windows.iter_mut().find(|window| window.kind == WindowKind::Weekly) {
        Some(existing) if existing.resets_at.is_none() && top.resets_at.is_some() => {
            *existing = top
        }
        Some(_) => {}
        None => windows.push(top),
    }
}

fn classify(minutes: u32) -> (String, WindowKind) {
    match minutes {
        300 => ("five_hour".to_owned(), WindowKind::FiveHour),
        1_440 => ("daily".to_owned(), WindowKind::Daily),
        10_080 => ("limit_weekly".to_owned(), WindowKind::Weekly),
        other => (format!("window_{other}m"), WindowKind::Other),
    }
}

/// Window length in minutes from `{duration, timeUnit}`; an unknown unit is no length at all.
fn minutes_of(window: &Map<String, Value>) -> Option<u32> {
    let duration = count(window.get("duration"))?;
    if duration.fract() != 0.0 || duration < 1.0 {
        return None;
    }
    let unit = window
        .get("timeUnit")
        .or_else(|| window.get("time_unit"))
        .and_then(Value::as_str)?
        .to_ascii_uppercase();
    let unit = unit.strip_prefix("TIME_UNIT_").unwrap_or(&unit);
    let factor: u64 = match unit.strip_suffix('S').unwrap_or(unit) {
        "MINUTE" => 1,
        "HOUR" => 60,
        "DAY" => 1_440,
        _ => return None,
    };
    // `duration` is a validated whole number below MAX_COUNT.
    let minutes = (duration as u64).checked_mul(factor)?;
    if minutes > MAX_MINUTES { None } else { u32::try_from(minutes).ok() }
}

/// A `{limit, used, remaining, resetTime}` block as a window. `None` when it holds nothing usable.
fn counted(
    raw: &Map<String, Value>,
    mut window: UsageWindow,
    minutes: Option<u32>,
    now_secs: i64,
) -> Option<UsageWindow> {
    let limit = count(raw.get("limit")).filter(|limit| *limit > 0.0);
    let used = count(raw.get("used")).or_else(|| {
        let left = count(raw.get("remaining"))?;
        Some(limit? - left).filter(|used| *used >= 0.0)
    });
    window.window_minutes = minutes;
    window.resets_at = RESET_KEYS.iter().find_map(|key| reset(raw.get(*key), minutes, now_secs));
    if let (Some(limit), Some(used)) = (limit, used) {
        window.used_pct = Some((used / limit * 100.0).min(100.0));
        window.amount = Some(Amount { used, limit: Some(limit), unit: AmountUnit::Requests });
    }
    (window.used_pct.is_some() || window.resets_at.is_some()).then_some(window)
}

/// `usages.<key>`: `{used_ratio, reset_time}` with the ratio a 0..=1 share of the pool.
fn ratio_window(
    map: &Map<String, Value>,
    key: &str,
    minutes: Option<u32>,
    now_secs: i64,
) -> Option<UsageWindow> {
    let raw = map.get(key)?.as_object()?;
    let mut window = UsageWindow::new(key, WindowKind::Other);
    window.used_pct = raw
        .get("used_ratio")
        .and_then(Value::as_f64)
        .filter(|ratio| ratio.is_finite() && (0.0..=1.0).contains(ratio))
        .map(|ratio| ratio * 100.0);
    window.resets_at = RESET_KEYS.iter().find_map(|name| reset(raw.get(*name), minutes, now_secs));
    (window.used_pct.is_some() || window.resets_at.is_some()).then_some(window)
}

fn with_kind(mut window: UsageWindow, id: &str, kind: WindowKind) -> UsageWindow {
    id.clone_into(&mut window.id);
    window.kind = kind;
    window
}

/// The newer shape reports the 5 h pool as a ratio too. `limits[]` stays the source when it
/// gives a percentage; the ratio only completes a rolling window that came back without one.
fn fill_five_hour_from_ratio(
    windows: &mut Vec<UsageWindow>,
    ratios: Option<&Map<String, Value>>,
    now_secs: i64,
) {
    let Some(ratio) = ratios.and_then(|map| ratio_window(map, "limit_5h", Some(300), now_secs))
    else {
        return;
    };
    let mut fallback = with_kind(ratio, "five_hour", WindowKind::FiveHour);
    fallback.window_minutes = Some(300);
    match windows.iter_mut().find(|window| window.id == "five_hour") {
        Some(existing) if existing.used_pct.is_none() => {
            existing.used_pct = fallback.used_pct;
            existing.resets_at = existing.resets_at.or(fallback.resets_at);
        }
        Some(_) => {}
        None => windows.push(fallback),
    }
}

/// A non-negative number or a string holding one: the service sends counts as strings.
fn count(value: Option<&Value>) -> Option<f64> {
    let number = match value? {
        Value::Number(number) => number.as_f64()?,
        Value::String(text) => text.trim().parse::<f64>().ok()?,
        _ => return None,
    };
    (number.is_finite() && (0.0..=MAX_COUNT).contains(&number)).then_some(number)
}

fn reset(value: Option<&Value>, minutes: Option<u32>, now_secs: i64) -> Option<i64> {
    let secs = parse_rfc3339_secs(value?.as_str()?)?;
    let ceiling = minutes.map_or(MAX_RESET_AHEAD_SECS, |m| i64::from(m) * 60 + RESET_SLACK_SECS);
    (0..=now_secs.saturating_add(ceiling)).contains(&secs).then_some(secs)
}

/// `user.membership.level` is a wire enum such as `LEVEL_INTERMEDIATE`. Only a plain upper-case
/// token is shown, prettified; the tier names behind the levels are not published, so none is made up.
fn plan_of(root: &Map<String, Value>) -> Option<String> {
    let level = root.get("user")?.get("membership")?.get("level")?.as_str()?;
    let body = level.strip_prefix("LEVEL_").unwrap_or(level);
    let plain = !body.is_empty()
        && body.len() <= 24
        && body.bytes().all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_');
    if !plain {
        return None;
    }
    let words: Vec<String> = body
        .split('_')
        .filter(|word| !word.is_empty())
        .map(|word| format!("{}{}", &word[..1], word[1..].to_ascii_lowercase()))
        .collect();
    (!words.is_empty()).then(|| words.join(" "))
}
