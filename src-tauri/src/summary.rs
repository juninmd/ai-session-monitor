//! Which reading drives the tray ring and tooltip: the pure choice between harnesses and windows.

use usage_model::settings::GeneralSettings;
use usage_model::status::{Status, summarize};
use usage_model::{HarnessId, Snapshot, WindowKind};

#[cfg(test)]
mod tests;

/// The most constrained usable window of the followed harness (or of all of them).
#[derive(Clone, Debug, PartialEq)]
pub struct TrayReading {
    pub harness: HarnessId,
    pub kind: WindowKind,
    pub window_minutes: Option<u32>,
    pub resets_at: Option<i64>,
    pub available_pct: f64,
    pub status: Status,
}

/// `None` when nothing usable exists: no data yet, stale, expired, or the followed harness has
/// none. Stale and expired windows must not colour the tray (see `usage_model::status`).
pub fn pick(snapshot: &Snapshot, general: &GeneralSettings, now_secs: i64) -> Option<TrayReading> {
    let warn_at = f64::from(general.warn_at_pct);
    snapshot
        .views
        .iter()
        .filter(|view| general.tray_source.is_none_or(|followed| followed == view.id))
        .filter_map(|view| {
            let summary = summarize(view, now_secs, warn_at)?;
            let window = view.windows.iter().find(|w| w.id == summary.window_id)?;
            Some(TrayReading {
                harness: view.id,
                kind: window.kind,
                window_minutes: window.window_minutes,
                resets_at: window.resets_at,
                available_pct: summary.available_pct,
                status: summary.status,
            })
        })
        // `min_by` keeps the first of equals, so a tie goes to the earlier tab.
        .min_by(|a, b| a.available_pct.total_cmp(&b.available_pct))
}
