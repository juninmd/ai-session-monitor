//! The shim's output file: `{"v":1,"capturedAt":<epoch s>,"rateLimits":<object|null>}`.
//!
//! `rateLimits` is the `rate_limits` object Claude Code pipes to its status line. Per the
//! statusLine docs it holds up to three independent windows, each dropped once it has reset:
//! `five_hour` and `seven_day` (`used_percentage` 0..=100, `resets_at` epoch seconds) and, behind
//! a Claude apps gateway, `spend_limit` (the same two fields, a percentage that may pass 100, and
//! optionally `used_usd`, `limit_usd` and `period`).

use std::io::ErrorKind;
use std::path::Path;

use serde_json::{Map, Value};
use usage_model::{Amount, AmountUnit, ErrorCode, HarnessError, UsageWindow, WindowKind};

use crate::lenient::{amount, epoch_secs, percent};

pub(crate) const MAX_FILE_BYTES: usize = 64 * 1024;
pub(crate) const FIVE_HOUR_MINUTES: u32 = 300;
pub(crate) const WEEKLY_MINUTES: u32 = 10_080;
const DAILY_MINUTES: u32 = 1_440;
const MAX_WINDOW_PCT: f64 = 100.0;
/// A spend limit may be exceeded, so its percentage is allowed past 100 (up to an absurd bound).
const MAX_SPEND_PCT: f64 = 100_000.0;
const UTF8_BOM: &[u8] = b"\xEF\xBB\xBF";

#[derive(Debug, PartialEq)]
pub(crate) struct DataFile {
    pub captured_at: i64,
    /// `None`: Claude Code reported no `rate_limits` at all.
    pub windows: Option<Vec<UsageWindow>>,
}

/// `Ok(None)` while the shim has not written the file yet.
pub(crate) fn read(path: &Path) -> Result<Option<DataFile>, HarnessError> {
    match usage_support::read_capped(path, MAX_FILE_BYTES) {
        Ok(bytes) => parse(&bytes).map(Some),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) if e.kind() == ErrorKind::FileTooLarge => {
            Err(HarnessError::new(ErrorCode::ResponseTooLarge, false))
        }
        Err(_) => Err(HarnessError::new(ErrorCode::Internal, true)),
    }
}

pub(crate) fn parse(bytes: &[u8]) -> Result<DataFile, HarnessError> {
    let bytes = bytes.strip_prefix(UTF8_BOM).unwrap_or(bytes);
    // Not JSON at all (a torn or hand-edited file): the shim rewrites it on its next run.
    let doc: Value = serde_json::from_slice(bytes)
        .map_err(|_| HarnessError::new(ErrorCode::BadResponse, true))?;
    let changed = || HarnessError::new(ErrorCode::EndpointChanged, false);
    if doc.get("v").and_then(Value::as_u64) != Some(1) {
        return Err(changed());
    }
    let captured_at = epoch_secs(doc.get("capturedAt")).ok_or_else(changed)?;
    let windows = match doc.get("rateLimits") {
        None | Some(Value::Null) => None,
        Some(Value::Object(limits)) if limits.is_empty() => None,
        Some(Value::Object(limits)) => Some(windows_of(limits).ok_or_else(changed)?),
        Some(_) => return Err(changed()),
    };
    Ok(DataFile { captured_at, windows })
}

/// `None` when the object holds none of the windows this reader knows.
fn windows_of(limits: &Map<String, Value>) -> Option<Vec<UsageWindow>> {
    let mut windows = Vec::new();
    if let Some(w) = limits
        .get("five_hour")
        .and_then(|raw| rolling("five_hour", WindowKind::FiveHour, FIVE_HOUR_MINUTES, raw))
    {
        windows.push(w);
    }
    if let Some(w) = limits
        .get("seven_day")
        .and_then(|raw| rolling("weekly", WindowKind::Weekly, WEEKLY_MINUTES, raw))
    {
        windows.push(w);
    }
    if let Some(w) = limits.get("spend_limit").and_then(spend) {
        windows.push(w);
    }
    (!windows.is_empty()).then_some(windows)
}

fn rolling(id: &str, kind: WindowKind, minutes: u32, raw: &Value) -> Option<UsageWindow> {
    let fields = raw.as_object()?;
    let mut window = UsageWindow::new(id, kind);
    window.window_minutes = Some(minutes);
    window.used_pct = percent(fields.get("used_percentage"), MAX_WINDOW_PCT);
    window.resets_at = epoch_secs(fields.get("resets_at"));
    Some(window)
}

fn spend(raw: &Value) -> Option<UsageWindow> {
    let fields = raw.as_object()?;
    let mut window = UsageWindow::new("spend_limit", WindowKind::Budget);
    window.used_pct = percent(fields.get("used_percentage"), MAX_SPEND_PCT);
    window.resets_at = epoch_secs(fields.get("resets_at"));
    window.window_minutes = match fields.get("period").and_then(Value::as_str) {
        Some("daily") => Some(DAILY_MINUTES),
        Some("weekly") => Some(WEEKLY_MINUTES),
        // A month has no fixed length, so no pace is derived from it.
        _ => None,
    };
    // A cap of zero (or a missing one) is not an amount worth showing.
    if let (Some(used), Some(limit)) =
        (amount(fields.get("used_usd")), amount(fields.get("limit_usd")))
        && limit > 0.0
    {
        window.amount = Some(Amount { used, limit: Some(limit), unit: AmountUnit::Usd });
    }
    Some(window)
}

#[cfg(test)]
mod tests;
