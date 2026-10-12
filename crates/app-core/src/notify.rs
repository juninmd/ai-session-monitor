//! Decides which desktop notifications a snapshot change deserves. Pure: the shell displays them.
//! `plan` covers transitions between readings; the `lapsed` helpers cover a reset that arrives
//! with no reading at all (see `Publisher`, which holds the state that makes it a single toast).

use usage_model::status::{available_pct, is_expired};
use usage_model::{HarnessId, Settings, Snapshot, ViewState};

mod lapsed;
mod text;
mod threshold;

pub(crate) use lapsed::{ResetKey, key_of, lapsed_resets, next_lapse, still_waiting};
pub(crate) use text::settings_recovered;
pub(crate) use threshold::ThresholdState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotificationKind {
    /// A window just ran out of quota.
    WindowOut,
    /// A window that was out is available again: seen in a reading, or, when the source stays
    /// silent, announced once by the clock at the window's reset time.
    WindowReset,
    /// A window's used percentage crossed one of `general.alertThresholds`.
    ThresholdCrossed,
    /// The settings file was unusable and was set aside; defaults are in effect. `harness` and
    /// `window_id` carry no meaning for this kind (`Claude` and empty).
    SettingsRecovered,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Notification {
    pub kind: NotificationKind,
    pub harness: HarnessId,
    pub window_id: String,
    /// Ready-to-show pt-BR text.
    pub title: String,
    pub body: String,
}

/// Notifications for the transition `prev` -> `next`, honouring `notify_on_out` and
/// `notify_on_reset`. Fires once per transition, never for the first snapshot.
///
/// A window only counts once it exists in both snapshots, so a first reading never notifies.
/// The new reading must be usable (fresh view, window not expired, percentage known); the old
/// one is taken as it was shown, stale or expired included, because a window that ran out and
/// reset while nobody was reading is exactly the transition the user wants to hear about.
pub fn plan(
    prev: &Snapshot,
    next: &Snapshot,
    settings: &Settings,
    now_secs: i64,
) -> Vec<Notification> {
    let general = &settings.general;
    let mut planned = Vec::new();
    if !(general.notify_on_out || general.notify_on_reset) {
        return planned;
    }
    for view in next.views.iter().filter(|v| v.state == ViewState::Ready && !v.stale) {
        let Some(before) = prev.views.iter().find(|p| p.id == view.id) else { continue };
        for window in view.windows.iter().filter(|w| !is_expired(w, now_secs)) {
            let was = before.windows.iter().find(|w| w.id == window.id).and_then(available_pct);
            let (Some(was), Some(now)) = (was, available_pct(window)) else { continue };
            match (was > 0.0, now > 0.0) {
                (true, false) if general.notify_on_out => {
                    planned.push(text::window_out(view.id, window, now_secs));
                }
                (false, true) if general.notify_on_reset => {
                    planned.push(text::window_reset(view.id, window, now, now_secs));
                }
                _ => {}
            }
        }
    }
    planned
}

#[cfg(test)]
mod lapsed_tests;
#[cfg(test)]
mod tests;
