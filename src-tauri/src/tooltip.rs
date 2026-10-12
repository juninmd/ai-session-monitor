//! Tray tooltip text in pt-BR, worded and formatted like the flyout. Windows cuts a tooltip at 127
//! UTF-16 units, so the text is the first of a few shrinking candidates that fits.

use usage_model::settings::{GeneralSettings, TimeFormat, WeekFormat};
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
    match id {
        HarnessId::Claude => "Claude Code",
        HarnessId::Codex => "Codex",
        HarnessId::Antigravity => "Antigravity",
        HarnessId::Opencode => "OpenCode",
        HarnessId::Litellm => "LiteLLM",
    }
}

fn window_name(kind: WindowKind) -> &'static str {
    match kind {
        WindowKind::FiveHour => "5 horas",
        WindowKind::Weekly => "Semana",
        WindowKind::Monthly => "Mês",
        WindowKind::Daily => "Dia",
        WindowKind::Budget => "Orçamento",
        WindowKind::Other => "Janela",
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
pub fn format_duration(minutes: f64, time: TimeFormat, week: WeekFormat, long: bool) -> String {
    let total = if minutes.is_finite() { minutes.max(0.0).round() as u32 } else { 0 };
    let days = total / MIN_PER_DAY;
    if long && days > 0 && (week == WeekFormat::DaysHours || total > MIN_PER_WEEK) {
        return format!("{days}d {}h", total % MIN_PER_DAY / 60);
    }
    let (hours, mins) = (total / 60, total % 60);
    match time {
        TimeFormat::Colon => format!("{hours}:{mins:02}"),
        TimeFormat::Decimal => {
            // `{:.1}` rounds the exact tie x.25 to even (0,2); the flyout's toFixed rounds it up.
            let tie = if total % 60 == 15 { 1.0 } else { 0.0 };
            format!("{:.1} h", (f64::from(total) + tie) / 60.0).replace('.', ",")
        }
        TimeFormat::Hm if hours == 0 => format!("{mins} min"),
        TimeFormat::Hm => format!("{hours}h {mins:02}m"),
    }
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

/// `followed` is the harness the user pinned the tray to, shown when it has no reading.
pub fn text(
    reading: Option<&TrayReading>,
    followed: Option<HarnessId>,
    general: &GeneralSettings,
    now_secs: i64,
) -> String {
    let Some(reading) = reading else {
        return format!("{}: sem dados", followed.map_or(APP_NAME, harness_name));
    };
    let name = harness_name(reading.harness);
    let head = format!("{name} · {}", window_name(reading.kind));
    let long = reading.window_minutes.map_or(
        matches!(reading.kind, WindowKind::Weekly | WindowKind::Monthly | WindowKind::Budget),
        |m| m > MIN_PER_DAY,
    );
    let left = reading.resets_at.map(|at| (at - now_secs).max(0) as f64 / 60.0);
    let dur = left.map(|m| format_duration(m, general.time_format, general.week_format, long));
    let out = reading.status == Status::Out;
    // Used is the complement of the available share the tray reading carries.
    let (shown, word) = if general.show_used {
        (100.0 - reading.available_pct, "usado")
    } else {
        (reading.available_pct, "disponível")
    };
    let pct = format_percent(shown);
    let state = if out { "esgotado".to_owned() } else { format!("{pct} {word}") };
    let when = |verb: &str| dur.as_ref().map(|d| format!(" · {verb} em {d}")).unwrap_or_default();
    let verb = if out { "volta" } else { "reseta" };
    fit(&[
        format!("{head}: {state}{}", when(verb)),
        format!("{name}: {state}{}", when(verb)),
        format!("{name}: {pct}"),
    ])
}
