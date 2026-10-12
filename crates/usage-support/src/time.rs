//! RFC 3339 timestamps to UTC epoch seconds, for sources that report reset times as text.

use ::time::OffsetDateTime;
use ::time::format_description::well_known::Rfc3339;

/// Parses an RFC 3339 timestamp (`Z` or a numeric offset, optional fractional seconds) into UTC
/// epoch seconds, rounded down. Anything else, including out-of-range dates, is `None`.
pub fn parse_rfc3339_secs(input: &str) -> Option<i64> {
    OffsetDateTime::parse(input, &Rfc3339).ok().map(OffsetDateTime::unix_timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_zulu_and_numeric_offsets_to_the_same_instant() {
        let utc = parse_rfc3339_secs("2026-10-10T12:00:00Z");
        assert_eq!(utc, Some(1_791_633_600));
        assert_eq!(parse_rfc3339_secs("2026-10-10T15:00:00+03:00"), utc);
        assert_eq!(parse_rfc3339_secs("2026-10-10T08:30:00-03:30"), utc);
        assert_eq!(parse_rfc3339_secs("2026-10-10T12:00:00+00:00"), utc);
        assert_eq!(parse_rfc3339_secs("2026-10-10T12:00:00-00:00"), utc);
    }

    #[test]
    fn fractional_seconds_are_dropped_not_rounded() {
        let base = parse_rfc3339_secs("2026-10-10T12:00:00Z");
        assert_eq!(parse_rfc3339_secs("2026-10-10T12:00:00.123456789Z"), base);
        assert_eq!(parse_rfc3339_secs("2026-10-10T12:00:00.999Z"), base);
        assert_eq!(parse_rfc3339_secs("2026-10-10T12:00:00.5+02:00").map(|v| v + 7200), base);
    }

    #[test]
    fn handles_leap_days_and_rejects_impossible_ones() {
        assert_eq!(parse_rfc3339_secs("2024-02-29T12:00:00Z"), Some(1_709_208_000));
        assert_eq!(parse_rfc3339_secs("2023-02-29T12:00:00Z"), None);
        // 2100 is divisible by 100 but not by 400, so it has no leap day.
        assert_eq!(parse_rfc3339_secs("2100-02-29T00:00:00Z"), None);
        assert_eq!(parse_rfc3339_secs("2000-02-29T00:00:00Z"), Some(951_782_400));
    }

    #[test]
    fn handles_dates_around_the_epoch_and_far_future() {
        assert_eq!(parse_rfc3339_secs("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(parse_rfc3339_secs("1969-12-31T23:59:59Z"), Some(-1));
        // Fractions floor towards the past, never towards zero.
        assert_eq!(parse_rfc3339_secs("1969-12-31T23:59:59.5Z"), Some(-1));
        assert_eq!(parse_rfc3339_secs("1969-07-20T20:17:40Z"), Some(-14_182_940));
        assert_eq!(parse_rfc3339_secs("2100-01-01T00:00:00Z"), Some(4_102_444_800));
    }

    #[test]
    fn extreme_years_and_offsets_never_panic() {
        for text in [
            "0000-01-01T00:00:00Z",
            "0000-01-01T00:00:00+23:59",
            "9999-12-31T23:59:59Z",
            "9999-12-31T23:59:59-23:59",
            "9999-12-31T23:59:60Z",
            "10000-01-01T00:00:00Z",
            "99999999999999999999-01-01T00:00:00Z",
            "-0001-01-01T00:00:00Z",
            "2026-10-10T12:00:00+99:99",
            "2026-13-40T25:61:61Z",
        ] {
            let _ = parse_rfc3339_secs(text);
        }
        assert_eq!(parse_rfc3339_secs("0000-01-01T00:00:00Z"), Some(-62_167_219_200));
        assert_eq!(parse_rfc3339_secs("9999-12-31T23:59:59Z"), Some(253_402_300_799));
        assert_eq!(parse_rfc3339_secs("10000-01-01T00:00:00Z"), None);
        assert_eq!(parse_rfc3339_secs("2026-10-10T12:00:00+99:99"), None);
    }

    #[test]
    fn rejects_garbage_partial_and_trailing_input() {
        for text in [
            "",
            " ",
            "not a date",
            "2026-10-10",
            "2026-10-10T12:00:00",
            "2026-10-10T12:00Z",
            "2026-10-10T12:00:00Z junk",
            "2026-10-10T12:00:00Zjunk",
            " 2026-10-10T12:00:00Z",
            "2026-10-10T12:00:00Z ",
            "2026-10-10T12:00:00Z\n",
            "1791633600",
            "2026-10-10T12:00:00.Z",
            "2026-10-10T12:00:00+0300",
            "\0",
            "2026-10-10T12:00:00Z\0",
        ] {
            assert_eq!(parse_rfc3339_secs(text), None, "{text:?}");
        }
    }

    #[test]
    fn rejects_non_ascii_digits_and_lookalikes() {
        // Arabic-Indic digits and a full-width colon must not slip through as a valid date.
        assert_eq!(parse_rfc3339_secs("٢٠٢٦-١٠-١٠T١٢:٠٠:٠٠Z"), None);
        assert_eq!(parse_rfc3339_secs("2026-10-10T12\u{ff1a}00:00Z"), None);
    }
}
