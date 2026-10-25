//! Quiet hours: whether a desktop notification falls in the window the user chose. Pure; the
//! publisher asks it before handing a notification to the shell, which then drops it.

use usage_model::settings::GeneralSettings;

const SECS_PER_DAY: i64 = 24 * 60 * 60;

/// True when `now_secs`, read on the local clock `utc_offset_secs` ahead of UTC, is inside the
/// quiet window. The window is `[quiet_from, quiet_to)`; when `from` is later than `to` it wraps
/// past midnight, and when they are equal there is no window at all.
pub(crate) fn is_quiet(general: &GeneralSettings, now_secs: i64, utc_offset_secs: i32) -> bool {
    if !general.quiet_hours_on {
        return false;
    }
    let (from, to) = (general.quiet_from, general.quiet_to);
    if from == to {
        return false;
    }
    let local = (now_secs + i64::from(utc_offset_secs)).rem_euclid(SECS_PER_DAY);
    let minute = u16::try_from(local / 60).unwrap_or(0);
    if from < to { (from..to).contains(&minute) } else { minute >= from || minute < to }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 2026-10-25T00:00:00Z, so the local minute of day is easy to read off in every test.
    const MIDNIGHT_UTC: i64 = 1_792_886_400;

    fn general(on: bool, from: u16, to: u16) -> GeneralSettings {
        GeneralSettings { quiet_hours_on: on, quiet_from: from, quiet_to: to, ..Default::default() }
    }

    /// The local time `hh:mm` on a zone `offset_hours` ahead of UTC, as a UTC epoch second.
    fn at(hh: i64, mm: i64, offset_hours: i64) -> i64 {
        MIDNIGHT_UTC + (hh * 60 + mm) * 60 - offset_hours * 3600
    }

    #[test]
    fn off_means_never_quiet() {
        let night = general(false, 22 * 60, 8 * 60);
        assert!(!is_quiet(&night, at(23, 0, 0), 0));
    }

    #[test]
    fn a_window_that_wraps_past_midnight_covers_both_sides_of_it() {
        let night = general(true, 22 * 60, 8 * 60);
        assert!(is_quiet(&night, at(22, 0, 0), 0), "the start is inside");
        assert!(is_quiet(&night, at(23, 59, 0), 0));
        assert!(is_quiet(&night, at(0, 0, 0), 0));
        assert!(is_quiet(&night, at(7, 59, 0), 0));
        assert!(!is_quiet(&night, at(8, 0, 0), 0), "the end is outside");
        assert!(!is_quiet(&night, at(12, 0, 0), 0));
        assert!(!is_quiet(&night, at(21, 59, 0), 0));
    }

    #[test]
    fn a_window_inside_one_day_does_not_wrap() {
        let lunch = general(true, 12 * 60, 13 * 60);
        assert!(is_quiet(&lunch, at(12, 30, 0), 0));
        assert!(!is_quiet(&lunch, at(13, 0, 0), 0));
        assert!(!is_quiet(&lunch, at(0, 30, 0), 0));
    }

    #[test]
    fn equal_bounds_mean_no_window() {
        let none = general(true, 9 * 60, 9 * 60);
        assert!(!is_quiet(&none, at(9, 0, 0), 0));
    }

    #[test]
    fn the_local_zone_decides_not_utc() {
        // 12:00 UTC is 14:00 at UTC+2 (outside 22:00-08:00); 23:30 UTC is 01:30 there (inside).
        // Reading UTC instead of the local zone would get both wrong.
        let night = general(true, 22 * 60, 8 * 60);
        assert!(!is_quiet(&night, at(12, 0, 0), 2 * 3600));
        assert!(is_quiet(&night, at(23, 30, 0), 2 * 3600));
        // A zone behind UTC: 03:00 UTC is 22:00 locally at UTC-5.
        assert!(is_quiet(&night, at(3, 0, 0), -5 * 3600));
    }

    #[test]
    fn a_negative_epoch_still_reads_the_right_minute() {
        let night = general(true, 22 * 60, 8 * 60);
        // Nineteen minutes before the epoch's midnight: 23:41 on the previous day.
        assert!(is_quiet(&night, -19 * 60, 0));
    }
}
