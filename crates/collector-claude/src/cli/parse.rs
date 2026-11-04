//! The `/usage` answer: the JSON envelope, then only the limit lines of its `result` text.
//!
//! Accepted lines, nothing else (everything after the blank line that ends the limits block, such
//! as the contributing-factors section with plugin and MCP names, is never looked at):
//!
//! ```text
//! Current session: 1% used · resets Oct 12, 1:40pm (America/Sao_Paulo)
//! Current week (all models): 78% used · resets Oct 17, 8pm (America/Sao_Paulo)
//! Current week (Fable): 0% used · resets Oct 17, 8pm (America/Sao_Paulo)
//! ```
//!
//! The reset text is localized and carries an IANA zone name, so it is never turned into a time.

use serde_json::Value;
use usage_model::{ErrorCode, HarnessError};

use super::{CliReport, ModelRow};
use crate::lenient::{clean_text, percent};

const MAX_ROWS: usize = 8;
const MAX_LABEL_CHARS: usize = 40;
const SESSION: &str = "Current session";
const WEEK: &str = "Current week (";
const ALL_MODELS: &str = "all models";
const USED_AND_RESETS: &str = "% used · resets ";

enum Line<'a> {
    Session { used_pct: Option<f64> },
    Week { label: &'a str, used_pct: Option<f64>, reset: &'a str },
}

/// The parsed `/usage` report; `EndpointChanged` when it is not the `/usage` answer.
pub(super) fn report(body: &Value) -> Result<CliReport, HarnessError> {
    let changed = || HarnessError::new(ErrorCode::EndpointChanged, false);
    let object = body.as_object().ok_or_else(changed)?;
    if object.get("local_command").and_then(Value::as_str) != Some("usage") {
        return Err(changed());
    }
    let text = object.get("result").and_then(Value::as_str).ok_or_else(changed)?;
    parse_block(text).ok_or_else(changed)
}

#[cfg(test)]
pub(super) fn envelope(body: &Value) -> Result<Vec<ModelRow>, HarnessError> {
    report(body).map(|r| r.rows)
}

/// `None` when the text has no limit line at all (a prompt answer, a changed layout).
fn parse_block(text: &str) -> Option<CliReport> {
    let mut lines = Vec::new();
    let mut in_block = false;
    for raw in text.lines().map(str::trim) {
        if in_block && raw.is_empty() {
            break;
        }
        in_block |= raw.starts_with(SESSION) || raw.starts_with("Current week");
        if in_block && let Some(line) = line(raw) {
            lines.push(line);
        }
    }
    if lines.is_empty() {
        return None;
    }
    let session_pct = lines.iter().find_map(|l| match l {
        Line::Session { used_pct } => percent(used_pct.map(Value::from).as_ref(), 100.0),
        _ => None,
    });
    let all_models = lines.iter().find_map(|l| match l {
        Line::Week { label, used_pct, reset } if is_all_models(label) => {
            Some((percent(used_pct.map(Value::from).as_ref(), 100.0), *reset))
        }
        _ => None,
    });
    let (weekly_pct, all_reset) = match all_models {
        Some((pct, reset)) => (pct, Some(reset)),
        None => (None, None),
    };
    let rows = lines.iter().filter_map(|line| match line {
        Line::Week { label, used_pct, reset } if !is_all_models(label) => Some(ModelRow {
            label: clean_text(label, MAX_LABEL_CHARS)?,
            used_pct: percent(used_pct.map(Value::from).as_ref(), 100.0),
            same_reset: all_reset.is_some_and(|all| same_reset_text(all, reset)),
        }),
        _ => None,
    });
    Some(CliReport { session_pct, weekly_pct, rows: rows.take(MAX_ROWS).collect() })
}

fn is_all_models(label: &str) -> bool {
    label.trim().eq_ignore_ascii_case(ALL_MODELS)
}

/// Sub-second jitter on the API (`22:59:59.861Z` vs `23:00:00Z`) prints as `<h>:59pm` on one
/// weekly row and `<h+1>pm` on another; rounding `:59` up to the hour keeps them matched.
fn same_reset_text(a: &str, b: &str) -> bool {
    normalize_reset(a) == normalize_reset(b)
}

fn normalize_reset(text: &str) -> String {
    text.split_whitespace()
        .map(|word| round_minute_59(word).unwrap_or_else(|| word.to_owned()))
        .collect::<Vec<_>>()
        .join(" ")
}

fn round_minute_59(word: &str) -> Option<String> {
    let (hour, meridiem) = word.split_once(":59")?;
    if !matches!(meridiem, "am" | "pm") {
        return None;
    }
    let h: u8 = hour.parse().ok().filter(|h| (1..=12).contains(h))?;
    if h == 11 && meridiem == "pm" {
        return None;
    }
    let next_h = if h == 12 { 1 } else { h + 1 };
    let next_m = if h == 11 { "pm" } else { meridiem };
    Some(format!("{next_h}{next_m}"))
}

fn line(raw: &str) -> Option<Line<'_>> {
    if let Some(tail) = raw.strip_prefix(SESSION).and_then(|t| t.strip_prefix(": ")) {
        return used_and_reset(tail).map(|(used_pct, _)| Line::Session { used_pct });
    }
    let rest = raw.strip_prefix(WEEK)?;
    let (label, tail) = rest.split_once("): ")?;
    let (used_pct, reset) = used_and_reset(tail)?;
    Some(Line::Week { label, used_pct, reset })
}

/// `78% used · resets <text>`; the number may be decimal, anything else is not this line.
fn used_and_reset(tail: &str) -> Option<(Option<f64>, &str)> {
    let (number, reset) = tail.split_once(USED_AND_RESETS)?;
    let digits = !number.is_empty() && number.chars().all(|c| c.is_ascii_digit() || c == '.');
    let reset = reset.trim();
    (digits && !reset.is_empty()).then(|| (number.parse::<f64>().ok(), reset))
}
