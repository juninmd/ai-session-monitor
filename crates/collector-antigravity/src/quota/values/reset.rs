//! When a window resets: an absolute time in any form seen so far, or prose such as "in 4 hours".

use serde_json::Value;
use usage_support::parse_rfc3339_secs;

use super::text::{Token, tokens};

/// 2000-01-01 and 2100-01-01 UTC: a reset outside this range is a changed meaning, not a date.
const MIN_EPOCH: i64 = 946_684_800;
const MAX_EPOCH: i64 = 4_102_444_800;
/// Epoch numbers from here up are milliseconds (as seconds this would be the year 5138).
const MILLIS_FROM: f64 = 100_000_000_000.0;
/// No plan resets further away than this; a longer "in N days" is something else.
const MAX_PROSE_SECS: i64 = 400 * 86_400;
const MAX_TIME_TEXT: usize = 64;

/// Epoch seconds of the reset of a bucket or quota object: `resetTime` when readable, else the
/// relative time its `description` states ("it will fully refresh in 5 days, 11 hours"), counted
/// from `now`, the moment the response was produced.
pub(crate) fn reset_at(object: &Value, now: i64) -> Option<i64> {
    // `reset_time` is how the `agy` CLI spells it.
    let stated = object.get("resetTime").or_else(|| object.get("reset_time"));
    stated.and_then(absolute).or_else(|| {
        object.get("description").and_then(Value::as_str).and_then(|text| relative(text, now))
    })
}

fn absolute(value: &Value) -> Option<i64> {
    let secs = match value {
        Value::String(text) if text.len() <= MAX_TIME_TEXT => {
            let text = text.trim();
            parse_rfc3339_secs(text).or_else(|| epoch(text.parse().ok()?))
        }
        Value::Number(number) => epoch(number.as_f64()?),
        _ => None,
    }?;
    (MIN_EPOCH..=MAX_EPOCH).contains(&secs).then_some(secs)
}

fn epoch(raw: f64) -> Option<i64> {
    if !raw.is_finite() || raw < 0.0 {
        return None;
    }
    let secs = if raw >= MILLIS_FROM { raw / 1000.0 } else { raw };
    // Saturating cast: a value this large is rejected by the range check.
    Some(secs.floor() as i64)
}

fn relative(text: &str, now: i64) -> Option<i64> {
    let found = tokens(text);
    let after_in = found.iter().enumerate().filter(|(_, token)| is_word(token, "in"));
    let total = after_in.into_iter().find_map(|(index, _)| duration_after(&found[index + 1..]));
    total.map(|secs| now.saturating_add(secs))
}

fn is_word(token: &Token, expected: &str) -> bool {
    matches!(token, Token::Word(word) if word == expected)
}

/// `5 days 11 hours`, `1 hour and 30 minutes`: the total, when at least one pair is there.
fn duration_after(tokens: &[Token]) -> Option<i64> {
    let (mut total, mut index) = (0_i64, 0);
    loop {
        if tokens.get(index).is_some_and(|token| is_word(token, "and")) {
            index += 1;
        }
        let (Some(Token::Number(count)), Some(Token::Word(unit))) =
            (tokens.get(index), tokens.get(index + 1))
        else {
            break;
        };
        let Some(unit_secs) = unit_seconds(unit) else { break };
        total = total.saturating_add(i64::try_from(*count).ok()?.saturating_mul(unit_secs));
        index += 2;
    }
    (1..=MAX_PROSE_SECS).contains(&total).then_some(total)
}

fn unit_seconds(unit: &str) -> Option<i64> {
    Some(match unit {
        "second" | "seconds" | "sec" | "secs" => 1,
        "minute" | "minutes" | "min" | "mins" => 60,
        "hour" | "hours" | "hr" | "hrs" => 3_600,
        "day" | "days" => 86_400,
        "week" | "weeks" => 604_800,
        _ => return None,
    })
}

#[cfg(test)]
mod tests;
