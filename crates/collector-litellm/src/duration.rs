//! `budget_duration` strings (`30d`, `24h`, `weekly`) as a fixed number of minutes.

/// Words the proxy accepts instead of a duration (`duration_parser._BUDGET_DURATION_WORD_ALIASES`).
const WORD_ALIASES: [(&str, &str); 4] =
    [("hourly", "1h"), ("daily", "24h"), ("weekly", "7d"), ("monthly", "30d")];

/// Length of a budget window in whole minutes. `None` when the string does not state a fixed
/// length: months (28 to 31 days), zero, sub-minute or fractional spans, and anything unparseable.
pub(crate) fn duration_minutes(raw: &str) -> Option<u32> {
    let text = raw.trim();
    let lowered = text.to_ascii_lowercase();
    let spec =
        WORD_ALIASES.iter().find(|(word, _)| *word == lowered).map_or(text, |(_, spec)| *spec);
    let (digits, unit) = spec.split_at(spec.find(|c: char| !c.is_ascii_digit())?);
    let unit_secs: u64 = match unit {
        "s" => 1,
        "m" => 60,
        "h" => 3_600,
        "d" => 86_400,
        "w" => 604_800,
        _ => return None,
    };
    let secs = digits.parse::<u64>().ok()?.checked_mul(unit_secs)?;
    if secs == 0 || !secs.is_multiple_of(60) {
        return None;
    }
    u32::try_from(secs / 60).ok()
}

#[cfg(test)]
mod tests {
    use super::duration_minutes;

    #[test]
    fn fixed_length_spans_are_converted_to_minutes() {
        for (raw, minutes) in [
            ("1h", 60),
            ("24h", 1_440),
            ("30d", 43_200),
            ("7d", 10_080),
            ("1w", 10_080),
            ("2w", 20_160),
            ("90m", 90),
            ("120s", 2),
            ("1d", 1_440),
            ("  30d  ", 43_200),
        ] {
            assert_eq!(duration_minutes(raw), Some(minutes), "{raw:?}");
        }
    }

    #[test]
    fn word_aliases_follow_the_proxy_table_in_any_case() {
        assert_eq!(duration_minutes("hourly"), Some(60));
        assert_eq!(duration_minutes("Daily"), Some(1_440));
        assert_eq!(duration_minutes("WEEKLY"), Some(10_080));
        assert_eq!(duration_minutes(" monthly "), Some(43_200));
    }

    #[test]
    fn months_are_ambiguous_so_the_length_stays_unknown() {
        for raw in ["1mo", "2mo", "12mo", "1MO"] {
            assert_eq!(duration_minutes(raw), None, "{raw:?}");
        }
    }

    #[test]
    fn zero_sub_minute_and_fractional_spans_are_unknown() {
        for raw in ["0d", "0h", "0s", "30s", "59s", "90s", "1.5d", "0.5h", "-1d", "+1d"] {
            assert_eq!(duration_minutes(raw), None, "{raw:?}");
        }
    }

    #[test]
    fn malformed_strings_are_unknown() {
        for raw in [
            "",
            " ",
            "d",
            "30",
            "30x",
            "30 d",
            "3 0d",
            "30dd",
            "30d2",
            "d30",
            "thirty days",
            "30D",
            "1H",
            "٣٠d",
            "\0",
            "1h30m",
            "yearly",
        ] {
            assert_eq!(duration_minutes(raw), None, "{raw:?}");
        }
        assert_eq!(duration_minutes("30d\n"), Some(43_200), "edge whitespace is trimmed");
    }

    #[test]
    fn overflowing_values_are_unknown_instead_of_wrapping() {
        assert_eq!(duration_minutes("99999999999999999999999d"), None);
        assert_eq!(duration_minutes("18446744073709551615w"), None);
        // 4,294,967,295 minutes is the largest span that fits the contract field.
        assert_eq!(duration_minutes("4294967295m"), Some(u32::MAX));
        assert_eq!(duration_minutes("4294967296m"), None);
    }
}
