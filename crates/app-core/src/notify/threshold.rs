//! Usage-threshold alerts: one toast per window per crossing, per reset epoch. The fired state
//! lives in memory like the other notification state; a reading seen for the first time after a
//! start only sets the baseline, so a restart never repeats a toast.

use std::collections::HashMap;

use usage_model::status::is_expired;
use usage_model::{HarnessId, Settings, Snapshot, ViewState};

use super::{Notification, text};

/// Harness, window id and reset moment: a reset gives the window a new key, which re-arms it.
type Key = (HarnessId, String, Option<i64>);

#[derive(Default)]
pub(crate) struct ThresholdState {
    /// Highest threshold already told (or baselined) for each window epoch.
    fired: HashMap<Key, u8>,
}

impl ThresholdState {
    /// Alerts for the crossings between `prev` and `next`; a jump over several thresholds tells
    /// only the highest. Unknown percentages, stale views and expired windows never alert.
    pub(crate) fn plan(
        &mut self,
        prev: Option<&Snapshot>,
        next: &Snapshot,
        settings: &Settings,
        now_secs: i64,
    ) -> Vec<Notification> {
        let levels = &settings.general.alert_thresholds;
        if levels.is_empty() {
            self.fired.clear();
            return Vec::new();
        }
        let mut planned = Vec::new();
        for view in next.views.iter().filter(|v| v.state == ViewState::Ready && !v.stale) {
            let before = prev.and_then(|p| p.views.iter().find(|b| b.id == view.id));
            let before = before.filter(|b| b.state == ViewState::Ready);
            for window in view.windows.iter().filter(|w| !is_expired(w, now_secs)) {
                let Some(used) = window.used_pct else { continue };
                let Some(&level) = levels.iter().rev().find(|&&t| f64::from(t) <= used) else {
                    continue;
                };
                let key = (view.id, window.id.clone(), window.resets_at);
                let known = before.is_some_and(|b| b.windows.iter().any(|w| w.id == window.id));
                let told = self.fired.get(&key).copied();
                if told.is_none_or(|t| level > t) {
                    self.fired.insert(key, level);
                    if known {
                        planned.push(text::threshold(
                            settings.general.language,
                            view.id,
                            window,
                            level,
                            used,
                            now_secs,
                        ));
                    }
                }
            }
        }
        self.fired.retain(|key, _| shown(next, key));
        planned
    }
}

/// Whether `snapshot` still has that window epoch, so its state is still needed.
fn shown(snapshot: &Snapshot, key: &Key) -> bool {
    let ready = snapshot.views.iter().filter(|v| v.id == key.0 && v.state == ViewState::Ready);
    ready.flat_map(|v| &v.windows).any(|w| w.id == key.1 && w.resets_at == key.2)
}

#[cfg(test)]
mod tests;
