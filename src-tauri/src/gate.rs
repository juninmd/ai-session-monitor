//! Click and blur bookkeeping for the flyout. On Windows a tray click first takes the focus away
//! from the open flyout (which hides it on blur) and only then delivers the click, so a plain
//! toggle would re-open what that very click just closed.

use std::time::{Duration, Instant};

#[cfg(test)]
mod tests;

/// Blurs this soon after opening are the shell settling focus, and a click this soon after a
/// blur-hide is the click that caused it.
pub const GUARD: Duration = Duration::from_millis(250);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClickAction {
    Show,
    Hide,
    /// The click that just closed the flyout by taking its focus: nothing left to do.
    Ignore,
}

#[derive(Debug, Default)]
pub struct Gate {
    shown_at: Option<Instant>,
    blur_hid_at: Option<Instant>,
    visible_at_press: bool,
}

impl Gate {
    pub fn shown(&mut self, now: Instant) {
        self.shown_at = Some(now);
        self.blur_hid_at = None;
    }

    /// The flyout lost focus. `true` when it must hide now.
    pub fn blurred(&mut self, now: Instant, visible: bool) -> bool {
        if !visible || self.within_guard(self.shown_at, now) {
            return false;
        }
        self.blur_hid_at = Some(now);
        true
    }

    /// Mouse button went down on the tray icon.
    pub fn press(&mut self, now: Instant, visible: bool) {
        self.visible_at_press = visible || self.within_guard(self.blur_hid_at, now);
    }

    /// Mouse button went up on the tray icon: what the click should do.
    pub fn release(&mut self, now: Instant, visible: bool) -> ClickAction {
        let closing =
            std::mem::take(&mut self.visible_at_press) || self.within_guard(self.blur_hid_at, now);
        if visible {
            return ClickAction::Hide;
        }
        if closing {
            // Consumed: a second click right after opens the flyout again.
            self.blur_hid_at = None;
            return ClickAction::Ignore;
        }
        ClickAction::Show
    }

    fn within_guard(&self, since: Option<Instant>, now: Instant) -> bool {
        since.is_some_and(|at| now.saturating_duration_since(at) < GUARD)
    }
}
