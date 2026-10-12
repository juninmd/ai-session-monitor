//! Turns one parsed record into the windows and facts the UI shows.

use std::cmp::Ordering;
use std::collections::HashSet;

use usage_model::{Amount, AmountUnit, Fact, UsageWindow, WindowKind};

use crate::duration::duration_minutes;
use crate::record::Level;
use crate::sanitize::display_text;

/// Windows beyond the headline budget; the panel body scrolls but a hundred rows help nobody.
const MAX_EXTRA_WINDOWS: usize = 16;
/// Free-label unit of a timestamp fact; the UI formats it as a date.
pub(crate) const EPOCH_UNIT: &str = "epoch_s";

pub(crate) struct Windows {
    pub list: Vec<UsageWindow>,
    /// Extra windows left out because of the cap above.
    pub dropped: usize,
}

/// The headline `budget` window first (the UI treats the first ungrouped window as the hero),
/// then concurrent windows and per-model caps, most consumed first.
pub(crate) fn windows(level: &Level, secret: &str) -> Windows {
    let mut list: Vec<UsageWindow> = headline(level, secret).into_iter().collect();
    let mut extras = limit_windows(level, secret);
    extras.extend(model_windows(level, secret));
    extras.sort_by(most_used_first);
    let mut seen = HashSet::new();
    extras.retain(|window| seen.insert(window.id.clone()));
    let dropped = extras.len().saturating_sub(MAX_EXTRA_WINDOWS);
    extras.truncate(MAX_EXTRA_WINDOWS);
    list.extend(extras);
    Windows { list, dropped }
}

pub(crate) fn facts(level: &Level) -> Vec<Fact> {
    let caps = &level.caps;
    let counts = [
        ("rpm_limit", caps.rpm, "requests"),
        ("tpm_limit", caps.tpm, "tokens"),
        ("max_parallel_requests", caps.parallel, "requests"),
    ];
    let expires = caps.expires.map(|secs| ("expires", secs as f64, EPOCH_UNIT));
    counts
        .into_iter()
        .filter_map(|(id, number, unit)| Some((id, number?, unit)))
        .chain(expires)
        .map(|(id, number, unit)| Fact {
            id: id.to_owned(),
            number: Some(number),
            text: None,
            unit: Some(unit.to_owned()),
        })
        .collect()
}

/// `None` unless the spend ratio is a finite number: a tiny cap must not turn into infinity.
fn percent(spend: f64, limit: f64) -> Option<f64> {
    Some(spend / limit * 100.0).filter(|pct| pct.is_finite())
}

fn budget_window(id: String, spend: Option<f64>, limit: f64) -> UsageWindow {
    let mut window = UsageWindow::new(id, WindowKind::Budget);
    if let Some(spend) = spend {
        window.used_pct = percent(spend, limit);
        window.amount = Some(Amount { used: spend, limit: Some(limit), unit: AmountUnit::Usd });
    }
    window
}

fn headline(level: &Level, secret: &str) -> Option<UsageWindow> {
    let mut window = budget_window("budget".to_owned(), level.spend, level.max_budget?);
    window.label = level.alias.as_deref().and_then(|alias| display_text(alias, secret));
    window.window_minutes = level.duration.as_deref().and_then(duration_minutes);
    window.resets_at = level.resets_at;
    Some(window)
}

fn limit_windows(level: &Level, secret: &str) -> Vec<UsageWindow> {
    level
        .limits
        .iter()
        .filter_map(|limit| {
            let name = display_text(&limit.duration, secret)?;
            let mut window = budget_window(format!("limit:{name}"), Some(limit.spend), limit.limit);
            window.label = Some(name);
            window.window_minutes = duration_minutes(&limit.duration);
            window.resets_at = limit.resets_at;
            Some(window)
        })
        .collect()
}

fn model_windows(level: &Level, secret: &str) -> Vec<UsageWindow> {
    level
        .models
        .iter()
        .filter_map(|model| {
            let name = display_text(&model.name, secret)?;
            let mut window = budget_window(format!("model:{name}"), Some(model.spend), model.limit);
            window.group = Some(name.clone());
            window.label = Some(name);
            window.window_minutes = model.period.as_deref().and_then(duration_minutes);
            Some(window)
        })
        .collect()
}

fn most_used_first(a: &UsageWindow, b: &UsageWindow) -> Ordering {
    let by_use = match (a.used_pct, b.used_pct) {
        (Some(x), Some(y)) => y.total_cmp(&x),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    };
    by_use.then_with(|| a.id.cmp(&b.id))
}

#[cfg(test)]
mod tests;
