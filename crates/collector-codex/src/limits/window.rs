//! One rate-limit window: bounds on what it may say, and what it is called.

use serde_json::Value;
use usage_model::{UsageWindow, WindowKind};

use super::value::{field, integer, number};

/// `used_pct` may exceed 100 (over budget); beyond this it is a broken number, not a measurement.
const MAX_USED_PCT: f64 = 1000.0;
const MAX_WINDOW_MINUTES: i64 = 525_600;
/// 9999-12-31T23:59:59Z: later than that a "seconds" value is really milliseconds.
const MAX_EPOCH_SECS: i64 = 253_402_300_799;

/// Windows Codex names by length, like its own status view (within 5 %).
const KNOWN: [(i64, &str, WindowKind); 4] = [
    (300, "five_hour", WindowKind::FiveHour),
    (1_440, "daily", WindowKind::Daily),
    (10_080, "weekly", WindowKind::Weekly),
    (43_200, "monthly", WindowKind::Monthly),
];

/// Which of the two slots of a snapshot a window came from.
#[derive(Clone, Copy)]
pub(super) enum Slot {
    Primary,
    Secondary,
}

impl Slot {
    const fn id(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Secondary => "secondary",
        }
    }
}

/// The fields of a window that passed the bounds; each may be missing.
pub(super) struct Raw {
    used_pct: Option<f64>,
    minutes: Option<u32>,
    resets_at: Option<i64>,
}

/// `None` when nothing in the window is usable.
pub(super) fn read(value: &Value) -> Option<Raw> {
    let window = value.as_object()?;
    let used_pct = number(field(window, &["usedPercent", "used_percent"]))
        .filter(|pct| (0.0..=MAX_USED_PCT).contains(pct));
    let minutes = integer(field(window, &["windowDurationMins", "window_minutes"]))
        .filter(|minutes| (1..=MAX_WINDOW_MINUTES).contains(minutes))
        .and_then(|minutes| u32::try_from(minutes).ok());
    let resets_at = integer(field(window, &["resetsAt", "resets_at"]))
        .filter(|secs| (1..=MAX_EPOCH_SECS).contains(secs));
    (used_pct.is_some() || minutes.is_some() || resets_at.is_some()).then_some(Raw {
        used_pct,
        minutes,
        resets_at,
    })
}

pub(super) fn build(raw: &Raw, slot: Slot, taken: &[UsageWindow]) -> UsageWindow {
    let (mut id, mut kind, minutes) = classify(raw.minutes, slot);
    if taken.iter().any(|window| window.id == id) {
        (id, kind) = (slot.id(), WindowKind::Other);
    }
    let mut window = UsageWindow::new(id, kind);
    window.window_minutes = minutes;
    window.used_pct = raw.used_pct;
    window.resets_at = raw.resets_at;
    window
}

/// By the stated length when there is one; otherwise by slot (primary is the 5 h session,
/// secondary the weekly one), whose length is then fixed by contract.
fn classify(stated: Option<u32>, slot: Slot) -> (&'static str, WindowKind, Option<u32>) {
    match (stated, slot) {
        (Some(minutes), _) => {
            let known = KNOWN.iter().find(|(length, ..)| within_five_percent(minutes, *length));
            match known {
                Some(&(_, id, kind)) => (id, kind, Some(minutes)),
                None => (slot.id(), WindowKind::Other, Some(minutes)),
            }
        }
        (None, Slot::Primary) => ("five_hour", WindowKind::FiveHour, Some(300)),
        (None, Slot::Secondary) => ("weekly", WindowKind::Weekly, Some(10_080)),
    }
}

fn within_five_percent(minutes: u32, target: i64) -> bool {
    (i64::from(minutes) - target).abs() * 100 <= target * 5
}
