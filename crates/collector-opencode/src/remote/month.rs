//! UTC calendar arithmetic for the monthly window length (Hinnant's civil-date algorithms), so the
//! crate needs no date dependency.

const SECS_PER_DAY: i64 = 86_400;

/// Days since 1970-01-01 for a proleptic Gregorian date. `month` is 1..=12, `day` is 1..=31.
pub(crate) fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year.rem_euclid(400);
    let shifted_month = i64::from(if month > 2 { month - 3 } else { month + 9 });
    let day_of_year = (153 * shifted_month + 2) / 5 + i64::from(day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// Inverse of [`days_from_civil`]: `(year, month, day)`.
pub(crate) fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 { shifted_month + 3 } else { shifted_month - 9 };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    // Month and day are in range by construction; the fallback only avoids a denied unwrap.
    (year, u32::try_from(month).unwrap_or(1), u32::try_from(day).unwrap_or(1))
}

fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_in_month(year: i64, month: u32) -> u32 {
    match month {
        2 if is_leap(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Length in minutes of the calendar month that ends at `resets_at`: the span back to the same
/// day and time one month earlier, with the day clamped to that month's length (the service
/// clamps its anniversary the same way). When the real anniversary was itself clamped the true
/// window can differ by a few days; the source does not say, so this is the best stated estimate.
pub(crate) fn month_minutes_ending_at(resets_at: i64) -> Option<u32> {
    let days = resets_at.div_euclid(SECS_PER_DAY);
    let time_of_day = resets_at.rem_euclid(SECS_PER_DAY);
    let (year, month, day) = civil_from_days(days);
    let (prev_year, prev_month) = if month == 1 { (year - 1, 12) } else { (year, month - 1) };
    let prev_day = day.min(days_in_month(prev_year, prev_month));
    let start = days_from_civil(prev_year, prev_month, prev_day) * SECS_PER_DAY + time_of_day;
    u32::try_from((resets_at - start) / 60).ok().filter(|minutes| *minutes > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAY_MIN: u32 = 1_440;

    fn at(year: i64, month: u32, day: u32, hour: i64, minute: i64) -> i64 {
        days_from_civil(year, month, day) * SECS_PER_DAY + hour * 3_600 + minute * 60
    }

    #[test]
    fn civil_conversion_round_trips_across_centuries() {
        for days in (-800_000..800_000).step_by(997) {
            let (y, m, d) = civil_from_days(days);
            assert_eq!(days_from_civil(y, m, d), days, "{y}-{m}-{d}");
        }
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(civil_from_days(20_833), (2027, 1, 15));
    }

    #[test]
    fn month_length_follows_the_month_before_the_reset() {
        // Oct has 31 days, Feb 28 or 29, Dec 31 (year boundary), Apr 30.
        assert_eq!(month_minutes_ending_at(at(2026, 11, 3, 14, 20)), Some(31 * DAY_MIN));
        assert_eq!(month_minutes_ending_at(at(2026, 3, 1, 0, 0)), Some(28 * DAY_MIN));
        assert_eq!(month_minutes_ending_at(at(2028, 3, 1, 0, 0)), Some(29 * DAY_MIN));
        assert_eq!(month_minutes_ending_at(at(2027, 1, 10, 9, 5)), Some(31 * DAY_MIN));
        assert_eq!(month_minutes_ending_at(at(2026, 5, 15, 23, 59)), Some(30 * DAY_MIN));
    }

    #[test]
    fn century_years_follow_the_gregorian_leap_rule() {
        assert_eq!(month_minutes_ending_at(at(2100, 3, 1, 0, 0)), Some(28 * DAY_MIN));
        assert_eq!(month_minutes_ending_at(at(2000, 3, 1, 0, 0)), Some(29 * DAY_MIN));
    }

    #[test]
    fn anniversary_day_is_clamped_to_the_shorter_previous_month() {
        // March 31 back to Feb 28: the clamped anniversary, 31 days.
        assert_eq!(month_minutes_ending_at(at(2026, 3, 31, 10, 0)), Some(31 * DAY_MIN));
        // March 30 in a leap year back to Feb 29 is not clamped to 28.
        assert_eq!(month_minutes_ending_at(at(2028, 3, 30, 10, 0)), Some(30 * DAY_MIN));
    }

    #[test]
    fn pre_epoch_and_extreme_values_never_panic() {
        for secs in [i64::MIN / 4, -1, 0, 1, i64::MAX / 4] {
            let _ = month_minutes_ending_at(secs);
        }
    }
}
