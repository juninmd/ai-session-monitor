//! Calendar arithmetic for reset dates: a date-only `quota_reset_date` and the first instant of
//! the next UTC month. Civil-date conversions follow Howard Hinnant's `days_from_civil` family.

use usage_support::parse_rfc3339_secs;

const DAY_SECS: i64 = 86_400;

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let month_index = (month + 9) % 12;
    let day_of_year = (153 * month_index + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// `(year, month, day)` of a day count since 1970-01-01.
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 { month_index + 3 } else { month_index - 9 };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

/// Midnight UTC of a strict `YYYY-MM-DD` date; any other text, or an impossible date, is `None`.
fn parse_date(text: &str) -> Option<i64> {
    let bytes = text.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return None;
    }
    let digits = |range: std::ops::Range<usize>| -> Option<i64> {
        let part = &bytes[range];
        part.iter()
            .all(u8::is_ascii_digit)
            .then(|| part.iter().fold(0_i64, |acc, digit| acc * 10 + i64::from(digit - b'0')))
    };
    let (year, month, day) = (digits(0..4)?, digits(5..7)?, digits(8..10)?);
    let days = days_from_civil(year, month, day);
    // A round trip rejects 2027-02-30 and month 13: they normalize to a different date.
    ((1..=12).contains(&month) && civil_from_days(days) == (year, month, day))
        .then_some(days * DAY_SECS)
}

/// A reset as the endpoint writes it: a full RFC 3339 instant or a bare UTC date.
pub(crate) fn parse_reset(text: &str) -> Option<i64> {
    let text = text.trim();
    parse_rfc3339_secs(text).or_else(|| parse_date(text))
}

/// The first instant of the UTC month after the one `now_secs` falls in. GitHub documents that
/// premium requests reset monthly on the 1st at 00:00:00 UTC.
pub(crate) fn next_month_start(now_secs: i64) -> i64 {
    let (year, month, _) = civil_from_days(now_secs.div_euclid(DAY_SECS));
    let (year, month) = if month == 12 { (year + 1, 1) } else { (year, month + 1) };
    days_from_civil(year, month, 1) * DAY_SECS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_only_resets_are_midnight_utc() {
        assert_eq!(parse_reset("2027-02-01"), Some(1_801_440_000));
        assert_eq!(parse_reset(" 2027-02-01 "), Some(1_801_440_000));
        assert_eq!(parse_reset("1970-01-01"), Some(0));
        assert_eq!(parse_reset("2024-02-29"), Some(1_709_164_800));
    }

    #[test]
    fn full_instants_are_read_with_their_offset() {
        assert_eq!(parse_reset("2027-02-01T00:00:00.000Z"), Some(1_801_440_000));
        assert_eq!(parse_reset("2027-02-01T02:00:00+02:00"), Some(1_801_440_000));
    }

    #[test]
    fn impossible_or_malformed_dates_are_unknown() {
        for text in [
            "",
            "2027-02-30",
            "2027-13-01",
            "2027-00-10",
            "2027-1-1",
            "27-02-01",
            "2027/02/01",
            "2027-02-0a",
            "2023-02-29",
            "tomorrow",
            "２０２７-02-01",
            "2027-02-01x",
        ] {
            assert_eq!(parse_reset(text), None, "{text:?}");
        }
    }

    #[test]
    fn the_next_month_starts_on_the_first_at_midnight_utc() {
        // 2027-01-15T08:00:00Z, 2027-01-31T23:59:59Z, 2027-02-01T00:00:00Z
        assert_eq!(next_month_start(1_800_000_000), 1_801_440_000);
        assert_eq!(next_month_start(1_801_439_999), 1_801_440_000);
        assert_eq!(next_month_start(1_801_440_000), 1_803_859_200);
    }

    #[test]
    fn december_rolls_into_january_and_leap_february_has_29_days() {
        // 2026-12-31T23:59:59Z -> 2027-01-01T00:00:00Z
        assert_eq!(next_month_start(1_798_761_599), 1_798_761_600);
        // 2028-02-10T00:00:00Z -> 2028-03-01T00:00:00Z (leap year)
        assert_eq!(next_month_start(1_833_753_600), 1_835_481_600);
    }
}
