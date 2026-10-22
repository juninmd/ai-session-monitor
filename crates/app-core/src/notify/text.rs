//! Wording of the notifications, in the language the user chose (see `crate::i18n`). Window labels
//! come from remote services, so they are stripped of control characters and cut short before they
//! reach a desktop toast.

use usage_model::settings::Language;
use usage_model::{HarnessId, UsageWindow, WindowKind};

use super::{Notification, NotificationKind};
use crate::i18n::{Messages, fill, harness_name, messages};

const MAX_LABEL_CHARS: usize = 40;

/// The sentence that says when the window resets, or that the time is unknown.
fn reset_sentence(m: &Messages, window: &UsageWindow, now_secs: i64) -> String {
    match window.resets_at {
        Some(at) => fill(m.reset_in, &[("span", &span(m, at - now_secs))]),
        None => m.reset_unknown.to_owned(),
    }
}

pub(super) fn window_out(
    language: Language,
    harness: HarnessId,
    window: &UsageWindow,
    now_secs: i64,
) -> Notification {
    let m = messages(language);
    let values = [("window", subject(m, window)), ("reset", reset_sentence(m, window, now_secs))];
    Notification {
        kind: NotificationKind::WindowOut,
        harness,
        window_id: window.id.clone(),
        title: fill(m.window_out_title, &[("harness", harness_name(harness))]),
        body: fill(m.window_out_body, &borrowed(&values)),
    }
}

pub(super) fn window_reset(
    language: Language,
    harness: HarnessId,
    window: &UsageWindow,
    available_pct: f64,
    show_used: bool,
    now_secs: i64,
) -> Notification {
    let m = messages(language);
    let free = available_pct.round().clamp(0.0, 100.0) as u8;
    let share = if show_used {
        fill(m.share_used, &[("pct", &(100 - free).to_string())])
    } else {
        fill(m.share_free, &[("pct", &free.to_string())])
    };
    let window_name = subject(m, window);
    let body = match window.resets_at {
        Some(at) => fill(
            m.window_reset_body_next,
            &[("window", &window_name), ("share", &share), ("span", &span(m, at - now_secs))],
        ),
        None => fill(m.window_reset_body, &[("window", &window_name), ("share", &share)]),
    };
    Notification {
        kind: NotificationKind::WindowReset,
        harness,
        window_id: window.id.clone(),
        title: fill(m.window_reset_title, &[("harness", harness_name(harness))]),
        body,
    }
}

/// A usage threshold was crossed; `level` is the threshold, `used` the reading that crossed it.
pub(super) fn threshold(
    language: Language,
    harness: HarnessId,
    window: &UsageWindow,
    level: u8,
    used: f64,
    now_secs: i64,
) -> Notification {
    let m = messages(language);
    let shown = used.round().clamp(0.0, 999.0) as u16;
    let values = [
        ("window", subject(m, window)),
        ("pct", shown.to_string()),
        ("reset", reset_sentence(m, window, now_secs)),
    ];
    Notification {
        kind: NotificationKind::ThresholdCrossed,
        harness,
        window_id: window.id.clone(),
        title: fill(
            m.threshold_title,
            &[("harness", harness_name(harness)), ("level", &level.to_string())],
        ),
        body: fill(m.threshold_body, &borrowed(&values)),
    }
}

/// The pace projects the window running out `early_min` minutes before it renews.
pub(super) fn pace_risk(
    language: Language,
    harness: HarnessId,
    window: &UsageWindow,
    early_min: f64,
) -> Notification {
    let m = messages(language);
    let early = span(m, (early_min.max(0.0) * 60.0) as i64);
    Notification {
        kind: NotificationKind::PaceRisk,
        harness,
        window_id: window.id.clone(),
        title: fill(m.pace_title, &[("harness", harness_name(harness))]),
        body: fill(m.pace_body, &[("window", &subject(m, window)), ("span", &early)]),
    }
}

/// The reset came by the clock with no reading behind it, so the text claims no free percentage.
pub(super) fn window_lapsed(
    language: Language,
    harness: HarnessId,
    window: &UsageWindow,
) -> Notification {
    let m = messages(language);
    Notification {
        kind: NotificationKind::WindowReset,
        harness,
        window_id: window.id.clone(),
        title: fill(m.window_reset_title, &[("harness", harness_name(harness))]),
        body: fill(m.window_lapsed_body, &[("window", &subject(m, window))]),
    }
}

/// Shown when the settings file had to be set aside; `file_name` is the copy's name, no path.
pub(crate) fn settings_recovered(language: Language, file_name: &str) -> Notification {
    let m = messages(language);
    Notification {
        kind: NotificationKind::SettingsRecovered,
        harness: HarnessId::Claude,
        window_id: String::new(),
        title: m.settings_recovered_title.to_owned(),
        body: fill(m.settings_recovered_body, &[("file_name", file_name)]),
    }
}

fn borrowed<'a>(values: &'a [(&'a str, String)]) -> Vec<(&'a str, &'a str)> {
    values.iter().map(|(key, value)| (*key, value.as_str())).collect()
}

/// The window as a phrase, for example `Weekly window` or `Budget (team-a)`.
fn subject(m: &Messages, window: &UsageWindow) -> String {
    let base = match window.kind {
        WindowKind::FiveHour => m.window_five_hour,
        WindowKind::Weekly => m.window_weekly,
        WindowKind::Monthly => m.window_monthly,
        WindowKind::Daily => m.window_daily,
        WindowKind::Budget => m.window_budget,
        WindowKind::Other => m.window_other,
    };
    let detail = window.label.as_deref().or(window.group.as_deref()).map(clean);
    match detail.filter(|detail| !detail.is_empty()) {
        Some(detail) => fill(m.window_with_detail, &[("name", base), ("detail", &detail)]),
        None => base.to_owned(),
    }
}

fn clean(raw: &str) -> String {
    let printable: String = raw.chars().filter(|c| !c.is_control()).collect();
    let printable = printable.trim();
    if printable.chars().count() > MAX_LABEL_CHARS {
        let cut: String = printable.chars().take(MAX_LABEL_CHARS).collect();
        format!("{}…", cut.trim_end())
    } else {
        printable.to_owned()
    }
}

/// `45 min`, `1 h 53 min`, `2 h`, `4 d 11 h`; rounded up to the minute so it never reads "0 min".
fn span(m: &Messages, secs: i64) -> String {
    let minutes = secs.max(0).saturating_add(59) / 60;
    let n = |value: i64| value.to_string();
    match minutes {
        0 => m.span_less_than_minute.to_owned(),
        1..=59 => fill(m.span_minutes, &[("min", &n(minutes))]),
        60..=1439 => match minutes % 60 {
            0 => fill(m.span_hours, &[("hours", &n(minutes / 60))]),
            rest => fill(m.span_hours_minutes, &[("hours", &n(minutes / 60)), ("min", &n(rest))]),
        },
        _ => match (minutes / 60) % 24 {
            0 => fill(m.span_days, &[("days", &n(minutes / 1440))]),
            hours => fill(m.span_days_hours, &[("days", &n(minutes / 1440)), ("hours", &n(hours))]),
        },
    }
}

#[cfg(test)]
mod tests;
