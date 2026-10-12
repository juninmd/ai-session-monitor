//! Turns the `model_remains` document into quota windows. The schema is unpublished, so every
//! field is typed defensively; what cannot be trusted is unknown, never a made-up number.

use serde_json::{Map, Value};
use usage_model::{ErrorCode, HarnessError, UsageWindow, WindowKind};

/// A document with more buckets than this is not the one we know.
const MAX_MODELS: usize = 16;
const MAX_NAME_CHARS: usize = 48;
/// A count beyond this is not a request quota.
const MAX_COUNT: f64 = 1e12;
const MS_PER_SEC: i64 = 1000;
const MS_PER_MIN: i64 = 60_000;
/// No window of this plan lasts longer than a week, so a longer span is a garbled time.
const MAX_SPAN_MS: i64 = 8 * 24 * 3_600 * MS_PER_SEC;

fn endpoint_changed() -> HarnessError {
    HarnessError::new(ErrorCode::EndpointChanged, false)
}

/// `EndpointChanged` unless `model_remains` is a list and, when it has entries, one is readable.
/// A model whose totals are zero has no such quota in the plan and yields no window.
pub(crate) fn parse_remains(body: &Value, now_secs: i64) -> Result<Vec<UsageWindow>, HarnessError> {
    let root = body.as_object().ok_or_else(endpoint_changed)?;
    // MiniMax answers failures inside a 200 as a non-zero base_resp.status_code.
    let status = root.get("base_resp").and_then(|b| b.get("status_code")).and_then(Value::as_i64);
    if status.is_some_and(|code| code != 0) {
        return Err(HarnessError::new(ErrorCode::BadResponse, false));
    }
    let models =
        root.get("model_remains").and_then(Value::as_array).ok_or_else(endpoint_changed)?;
    let mut windows: Vec<UsageWindow> = Vec::new();
    let mut readable = 0;
    for entry in models.iter().take(MAX_MODELS).filter_map(Value::as_object) {
        let Some(name) = entry.get("model_name").and_then(Value::as_str).and_then(clean_name)
        else {
            continue;
        };
        readable += 1;
        for window in model_windows(entry, name, now_secs) {
            // Two entries claiming one id is ambiguity, not data: the first one stands.
            if !windows.iter().any(|known| known.id == window.id) {
                windows.push(window);
            }
        }
    }
    if !models.is_empty() && readable == 0 {
        return Err(endpoint_changed());
    }
    Ok(windows)
}

/// A name the UI can show as a pool title: short, printable, no markup.
fn clean_name(raw: &str) -> Option<&str> {
    let name = raw.trim();
    let allowed = |c: char| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | ' ' | '*');
    (!name.is_empty() && name.chars().count() <= MAX_NAME_CHARS && name.chars().all(allowed))
        .then_some(name)
}

fn slug(name: &str) -> String {
    let mapped: String = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '_' })
        .collect();
    mapped.trim_matches('_').to_owned()
}

fn model_windows(entry: &Map<String, Value>, name: &str, now_secs: i64) -> Vec<UsageWindow> {
    let slug = slug(name);
    let interval = bucket(entry, "current_interval").map(|used_pct| {
        let start = millis(entry, "start_time");
        let end = millis(entry, "end_time");
        let minutes = start
            .zip(end)
            .map(|(start, end)| end - start)
            .filter(|span| (1..=MAX_SPAN_MS).contains(span))
            .and_then(|span| u32::try_from(span / MS_PER_MIN).ok());
        let mut window = UsageWindow::new(format!("{slug}:interval"), kind_for(minutes));
        window.window_minutes = minutes;
        window.used_pct = Some(used_pct);
        window.resets_at = reset_at(entry, "remains_time", "end_time", now_secs);
        window
    });
    let weekly = bucket(entry, "current_weekly").map(|used_pct| {
        let mut window = UsageWindow::new(format!("{slug}:weekly"), WindowKind::Weekly);
        window.used_pct = Some(used_pct);
        window.resets_at = reset_at(entry, "weekly_remains_time", "weekly_end_time", now_secs);
        window
    });
    [interval, weekly]
        .into_iter()
        .flatten()
        .map(|mut window| {
            window.group = Some(name.to_owned());
            window
        })
        .collect()
}

/// The length decides the name; a length that matches no cadence is just a window.
fn kind_for(minutes: Option<u32>) -> WindowKind {
    match minutes {
        Some(300) => WindowKind::FiveHour,
        Some(1440) => WindowKind::Daily,
        Some(10_080) => WindowKind::Weekly,
        _ => WindowKind::Other,
    }
}

fn count(entry: &Map<String, Value>, field: &str) -> Option<f64> {
    let number = entry.get(field)?.as_f64()?;
    (number.is_finite() && (0.0..=MAX_COUNT).contains(&number)).then_some(number)
}

/// Used share of one bucket, 0..=100, or `None` when the plan has no such quota or the figures are
/// unreadable. `*_usage_count` holds what is LEFT (community reports; unverified), unless an
/// explicit remaining field is present, which wins.
fn bucket(entry: &Map<String, Value>, prefix: &str) -> Option<f64> {
    let total = count(entry, &format!("{prefix}_total_count")).filter(|total| *total > 0.0)?;
    let left = ["remaining_count", "remains_count", "usage_count"]
        .iter()
        .find_map(|suffix| count(entry, &format!("{prefix}_{suffix}")))?;
    Some(((total - left.min(total)) * 100.0 / total).clamp(0.0, 100.0))
}

fn millis(entry: &Map<String, Value>, field: &str) -> Option<i64> {
    let number = entry.get(field)?.as_i64()?;
    (number > 0).then_some(number)
}

/// The reset moment from a time-left field in milliseconds, else from an end time in epoch
/// milliseconds; anything outside a sane span is unknown.
fn reset_at(entry: &Map<String, Value>, left: &str, end: &str, now_secs: i64) -> Option<i64> {
    let sane = |span_ms: i64| (1..=MAX_SPAN_MS).contains(&span_ms);
    if let Some(left_ms) = millis(entry, left).filter(|ms| sane(*ms)) {
        return Some(now_secs + left_ms / MS_PER_SEC);
    }
    let end_ms = millis(entry, end)?;
    sane(end_ms - now_secs * MS_PER_SEC).then_some(end_ms / MS_PER_SEC)
}
