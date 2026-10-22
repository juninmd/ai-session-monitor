//! Tray tooltip text, in the user's language, worded and formatted like the flyout. Windows cuts a tooltip at 127
//! UTF-16 units, so the text is the first of a few shrinking candidates that fits.

use app_core::i18n::{self, Messages, fill, messages};
use usage_model::settings::{GeneralSettings, Language, TimeFormat, WeekFormat};
use usage_model::status::Status;
use usage_model::{HarnessId, WindowKind};

use crate::summary::TrayReading;

#[cfg(test)]
mod tests;

pub const MAX_UTF16: usize = 127;
const APP_NAME: &str = "AI Session Monitor";
const MIN_PER_DAY: u32 = 1440;
const MIN_PER_WEEK: u32 = 10_080;

pub fn harness_name(id: HarnessId) -> &'static str {
    i18n::harness_name(id)
}

fn window_name(m: &Messages, kind: WindowKind) -> &'static str {
    match kind {
        WindowKind::FiveHour => m.tip_window_five_hour,
        WindowKind::Weekly => m.tip_window_weekly,
        WindowKind::Monthly => m.tip_window_monthly,
        WindowKind::Daily => m.tip_window_daily,
        WindowKind::Budget => m.tip_window_budget,
        WindowKind::Other => m.tip_window_other,
    }
}

pub fn utf16_len(text: &str) -> usize {
    text.chars().map(char::len_utf16).sum()
}

fn format_percent(pct: f64) -> String {
    if pct > 0.0 && pct < 1.0 { "<1%".to_owned() } else { format!("{}%", pct.round() as i64) }
}

/// Same rules as `durationParts` in `src/i18n/format.ts`. `long` marks multi-day windows, which
/// read as days and hours when asked to, and always beyond one week.
pub fn format_duration(
    minutes: f64,
    time: TimeFormat,
    week: WeekFormat,
    long: bool,
    language: Language,
) -> String {
    let m = messages(language);
    let total = if minutes.is_finite() { minutes.max(0.0).round() as u32 } else { 0 };
    let days = total / MIN_PER_DAY;
    if long && days > 0 && (week == WeekFormat::DaysHours || total > MIN_PER_WEEK) {
        let values =
            [("days", days.to_string()), ("hours", (total % MIN_PER_DAY / 60).to_string())];
        return fill(m.duration_days_hours, &borrowed(&values));
    }
    let (hours, mins) = (total / 60, total % 60);
    match time {
        TimeFormat::Colon => format!("{hours}:{mins:02}"),
        TimeFormat::Decimal => {
            // `{:.1}` rounds the exact tie x.25 to even (0,2); the flyout's toFixed rounds it up.
            let tie = if total % 60 == 15 { 1.0 } else { 0.0 };
            let value =
                format!("{:.1}", (f64::from(total) + tie) / 60.0).replace('.', m.decimal_separator);
            fill(m.duration_decimal_hours, &[("value", &value)])
        }
        TimeFormat::Hm if hours == 0 => fill(m.duration_minutes, &[("min", &mins.to_string())]),
        TimeFormat::Hm => {
            let values = [("hours", hours.to_string()), ("min", format!("{mins:02}"))];
            fill(m.duration_hours_minutes, &borrowed(&values))
        }
    }
}

fn borrowed<'a>(values: &'a [(&'static str, String)]) -> Vec<(&'static str, &'a str)> {
    values.iter().map(|(key, value)| (*key, value.as_str())).collect()
}

/// First candidate within the limit; if none fits, the first one cut short with an ellipsis.
fn fit(candidates: &[String]) -> String {
    if let Some(found) = candidates.iter().find(|c| utf16_len(c) <= MAX_UTF16) {
        return found.clone();
    }
    let mut cut = String::new();
    for c in candidates.first().map(String::as_str).unwrap_or_default().chars() {
        if utf16_len(&cut) + c.len_utf16() >= MAX_UTF16 {
            break;
        }
        cut.push(c);
    }
    cut.push('…');
    cut
}

/// `followed` is the harness the user pinned the tray to, shown when it has no reading. The words
/// follow `general.language`.
pub fn text(
    reading: Option<&TrayReading>,
    followed: Option<HarnessId>,
    general: &GeneralSettings,
    now_secs: i64,
) -> String {
    let m = messages(general.language);
    let Some(reading) = reading else {
        let name = followed.map_or(APP_NAME, harness_name);
        return fill(m.tip_no_data, &[("name", name)]);
    };
    let name = harness_name(reading.harness);
    let head = fill(m.tip_head, &[("harness", name), ("window", window_name(m, reading.kind))]);
    let long = reading.window_minutes.map_or(
        matches!(reading.kind, WindowKind::Weekly | WindowKind::Monthly | WindowKind::Budget),
        |m| m > MIN_PER_DAY,
    );
    let left = reading.resets_at.map(|at| (at - now_secs).max(0) as f64 / 60.0);
    let dur = left.map(|mins| {
        format_duration(mins, general.time_format, general.week_format, long, general.language)
    });
    let out = reading.status == Status::Out;
    // Used is the complement of the available share the tray reading carries.
    let (shown, template) = if general.show_used {
        (100.0 - reading.available_pct, m.tip_used)
    } else {
        (reading.available_pct, m.tip_available)
    };
    let pct = format_percent(shown);
    let state = if out { m.tip_out.to_owned() } else { fill(template, &[("pct", &pct)]) };
    let when = dur.as_deref().map(|duration| {
        fill(if out { m.tip_back_in } else { m.tip_resets_in }, &[("duration", duration)])
    });
    let line = |head: &str, state: &str| match &when {
        Some(when) => fill(m.tip_line_when, &[("head", head), ("state", state), ("when", when)]),
        None => fill(m.tip_line, &[("head", head), ("state", state)]),
    };
    fit(&[
        line(&head, &state),
        line(name, &state),
        fill(m.tip_line, &[("head", name), ("state", &pct)]),
    ])
}
