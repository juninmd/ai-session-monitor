//! Defensive readers for numbers and text taken from files and responses we do not control.
//! Anything out of range is unknown (`None`), never clamped into a valid-looking value.

use serde_json::Value;

/// 2000-01-01 and 2100-01-01 in epoch seconds: outside this range a timestamp is garbage.
const MIN_EPOCH: i64 = 946_684_800;
const MAX_EPOCH: i64 = 4_102_444_800;

/// A percentage in `0..=max`. NaN, infinity, negatives and larger values are unknown.
pub(crate) fn percent(value: Option<&Value>, max: f64) -> Option<f64> {
    let n = value?.as_f64()?;
    (n.is_finite() && (0.0..=max).contains(&n)).then_some(n)
}

/// Epoch seconds as an integer (a whole float is accepted) inside a plausible range.
pub(crate) fn epoch_secs(value: Option<&Value>) -> Option<i64> {
    let value = value?;
    let secs = value.as_i64().or_else(|| {
        let n = value.as_f64()?;
        (n.is_finite() && n.fract() == 0.0 && n.abs() < 1e12).then_some(n as i64)
    })?;
    plausible(secs)
}

pub(crate) fn plausible(secs: i64) -> Option<i64> {
    (MIN_EPOCH..=MAX_EPOCH).contains(&secs).then_some(secs)
}

/// A non-negative, finite amount in a unit the source names.
pub(crate) fn amount(value: Option<&Value>) -> Option<f64> {
    let n = value?.as_f64()?;
    (n.is_finite() && (0.0..=1e12).contains(&n)).then_some(n)
}

/// Display text without control characters, trimmed and cut to `max_chars`; empty is `None`.
pub(crate) fn clean_text(raw: &str, max_chars: usize) -> Option<String> {
    let cleaned: String = raw.chars().filter(|c| !c.is_control()).collect();
    let trimmed = cleaned.trim();
    (!trimmed.is_empty()).then(|| trimmed.chars().take(max_chars).collect())
}

/// Lowercase ASCII words joined by `_`, usable as a window id or group; empty input gives `model`.
pub(crate) fn slug(raw: &str) -> String {
    let mut out = String::new();
    for c in raw.chars().take(48) {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.is_empty() && !out.ends_with('_') {
            out.push('_');
        }
    }
    let trimmed = out.trim_end_matches('_');
    if trimmed.is_empty() { "model".to_owned() } else { trimmed.to_owned() }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn percent_rejects_everything_outside_the_range_instead_of_clamping() {
        assert_eq!(percent(Some(&json!(0)), 100.0), Some(0.0));
        assert_eq!(percent(Some(&json!(100)), 100.0), Some(100.0));
        assert_eq!(percent(Some(&json!(23.5)), 100.0), Some(23.5));
        for bad in [json!(-0.1), json!(100.01), json!("12"), json!(null), json!([1]), json!(1e308)]
        {
            assert_eq!(percent(Some(&bad), 100.0), None, "{bad}");
        }
        assert_eq!(percent(None, 100.0), None);
        assert_eq!(percent(Some(&json!(250)), 1000.0), Some(250.0));
    }

    #[test]
    fn epoch_secs_wants_a_plausible_whole_number_of_seconds() {
        assert_eq!(epoch_secs(Some(&json!(1_800_000_000))), Some(1_800_000_000));
        assert_eq!(epoch_secs(Some(&json!(1_800_000_000.0))), Some(1_800_000_000));
        for bad in [
            json!(1_800_000_000.5),
            json!(0),
            json!(-5),
            json!(1_800_000_000_000_i64),
            json!("1800000000"),
            json!(4_102_444_801_i64),
            json!(946_684_799),
            json!(null),
        ] {
            assert_eq!(epoch_secs(Some(&bad)), None, "{bad}");
        }
    }

    #[test]
    fn amount_is_finite_and_not_negative() {
        assert_eq!(amount(Some(&json!(314.12))), Some(314.12));
        assert_eq!(amount(Some(&json!(0))), Some(0.0));
        for bad in [json!(-1), json!(1e13), json!("5"), json!(null)] {
            assert_eq!(amount(Some(&bad)), None, "{bad}");
        }
    }

    #[test]
    fn clean_text_strips_controls_trims_and_truncates_by_characters() {
        assert_eq!(clean_text("  Opus\u{0}\n ", 40).as_deref(), Some("Opus"));
        assert_eq!(clean_text("ção ✓ ação longa", 5).as_deref(), Some("ção ✓"));
        assert_eq!(clean_text(" \t\r\n", 40), None);
        assert_eq!(clean_text("", 40), None);
    }

    #[test]
    fn slug_is_ascii_lowercase_words() {
        assert_eq!(slug("Opus 4.1"), "opus_4_1");
        assert_eq!(slug("  --Fable!! "), "fable");
        assert_eq!(slug("ção"), "o");
        assert_eq!(slug("✓✓"), "model");
        assert_eq!(slug(""), "model");
        assert!(slug(&"a".repeat(500)).len() <= 48);
    }
}
