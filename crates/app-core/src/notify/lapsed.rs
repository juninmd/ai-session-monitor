//! Resets that arrive while the source is silent. A window shown as out whose `resetsAt` passes
//! with no newer reading is announced once, from the clock; a reading that later confirms the
//! reset finds it already told (the `Publisher` holds that state). Pure functions over snapshots.

use usage_model::status::available_pct;
use usage_model::{HarnessId, HarnessView, Settings, Snapshot, UsageWindow, ViewState};

use super::{Notification, NotificationKind, text};

/// Harness, window id and the reset moment: one reset of one window is told at most once.
pub(crate) type ResetKey = (HarnessId, String, i64);

/// The windows `shown` displayed as out with their reset still ahead of it, with that reset.
fn waiting(shown: &Snapshot) -> impl Iterator<Item = (HarnessId, &UsageWindow, i64)> {
    shown
        .views
        .iter()
        // LiteLLM's proxy applies a reset up to ~10 min late; its confirming poll tells it.
        .filter(|view| view.state == ViewState::Ready && view.id != HarnessId::Litellm)
        .flat_map(|view| view.windows.iter().map(move |window| (view.id, window)))
        .filter_map(|(id, window)| {
            let at = window.resets_at.filter(|&at| at > shown.generated_at)?;
            is_out(window).then_some((id, window, at))
        })
}

fn is_out(window: &UsageWindow) -> bool {
    available_pct(window).is_some_and(|available| available <= 0.0)
}

/// Whether `views` still describe that window as out with that reset: nothing newer replaced it.
fn still_out(views: &[HarnessView], harness: HarnessId, window_id: &str, at: i64) -> bool {
    let ready = views.iter().filter(|view| view.id == harness && view.state == ViewState::Ready);
    ready
        .flat_map(|view| &view.windows)
        .any(|window| window.id == window_id && window.resets_at == Some(at) && is_out(window))
}

/// Resets due at `now_secs` for windows `before` showed out, in `before`'s order, and that `now`
/// (the hub's current views) still shows out with the same reset. Stale data counts: a failing
/// source is exactly a silent one.
pub(crate) fn lapsed_resets(
    before: &Snapshot,
    now: &[HarnessView],
    settings: &Settings,
    now_secs: i64,
) -> Vec<(ResetKey, Notification)> {
    if !settings.general.notify_on_reset {
        return Vec::new();
    }
    waiting(before)
        .filter(|&(id, window, at)| at <= now_secs && still_out(now, id, &window.id, at))
        .map(|(id, window, at)| {
            let note = text::window_lapsed(settings.general.language, id, window);
            ((id, window.id.clone(), at), note)
        })
        .collect()
}

/// The earliest reset still ahead of `now_secs` among the windows `shown` has out: when the loop
/// must look again with no reading to prompt it.
pub(crate) fn next_lapse(shown: &Snapshot, settings: &Settings, now_secs: i64) -> Option<i64> {
    if !settings.general.notify_on_reset {
        return None;
    }
    waiting(shown).map(|(_, _, at)| at).filter(|&at| at > now_secs).min()
}

/// The reset a regular `WindowReset` confirms: the one `before` showed for that window.
pub(crate) fn key_of(before: &Snapshot, note: &Notification) -> Option<ResetKey> {
    if note.kind != NotificationKind::WindowReset {
        return None;
    }
    let view = before.views.iter().find(|view| view.id == note.harness)?;
    let window = view.windows.iter().find(|window| window.id == note.window_id)?;
    Some((note.harness, note.window_id.clone(), window.resets_at?))
}

/// Whether `shown` still has that window out with that reset, so its told mark is still needed.
pub(crate) fn still_waiting(shown: &Snapshot, key: &ResetKey) -> bool {
    still_out(&shown.views, key.0, &key.1, key.2)
}
