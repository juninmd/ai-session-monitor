//! Pace alerts: one toast per window epoch when the current rate of use would run the window out
//! before it renews. Like the threshold alerts, the told state lives in memory and a window seen
//! for the first time only sets the baseline, so a restart never repeats a toast.

use std::collections::HashSet;

use usage_model::status::{is_expired, pace_of};
use usage_model::{HarnessId, Pace, Settings, Snapshot, ViewState};

use super::{Notification, text};

/// Harness, window id and reset moment: a reset gives the window a new key, which re-arms it.
type Key = (HarnessId, String, Option<i64>);

#[derive(Default)]
pub(crate) struct PaceState {
    told: HashSet<Key>,
}

impl PaceState {
    /// Alerts for windows that newly project past their renewal. Stale views, expired windows and
    /// readings too early to extrapolate (`Pace::Early`) never alert.
    pub(crate) fn plan(
        &mut self,
        prev: Option<&Snapshot>,
        next: &Snapshot,
        settings: &Settings,
        now_secs: i64,
    ) -> Vec<Notification> {
        if !settings.general.notify_on_risk {
            self.told.clear();
            return Vec::new();
        }
        let mut planned = Vec::new();
        for view in next.views.iter().filter(|v| v.state == ViewState::Ready && !v.stale) {
            let before = prev.and_then(|p| p.views.iter().find(|b| b.id == view.id));
            let before = before.filter(|b| b.state == ViewState::Ready);
            for window in view.windows.iter().filter(|w| !is_expired(w, now_secs)) {
                let Pace::Risk { early_min, .. } = pace_of(window, now_secs) else { continue };
                let key = (view.id, window.id.clone(), window.resets_at);
                let known = before.is_some_and(|b| b.windows.iter().any(|w| w.id == window.id));
                if self.told.insert(key) && known {
                    planned.push(text::pace_risk(view.id, window, early_min));
                }
            }
        }
        self.told.retain(|key| shown(next, key));
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
