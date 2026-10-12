//! What a finished collection means for when the next one runs: adaptive stretch and the
//! rate-limit cool-down mark the UI shows.

use std::time::Duration;

use tokio::time::Instant;
use usage_model::Settings;

use super::Scheduler;
use crate::timing;

impl Scheduler {
    /// Adaptive cadence: a calm source is read less often (see `timing::adaptive_wait`).
    pub(super) fn stretch(
        &mut self,
        idx: usize,
        settings: &Settings,
        base: Duration,
        changed: bool,
        now: Instant,
    ) {
        let slot = &mut self.slots[idx];
        slot.note_read(changed);
        if !settings.general.adaptive {
            return;
        }
        let now_secs = self.shared.clock.now_secs();
        let view = self.shared.with_hub(|hub| hub.view_at(slot.id, now_secs));
        let hot = timing::hot_pct(settings);
        let wait = timing::adaptive_wait(base, &view.windows, slot.idle_ticks, hot, now_secs);
        slot.stretch_to(wait, now);
    }

    /// Tells the UI when the cool-down of a rate-limited source ends.
    pub(super) fn announce_cooldown(&self, idx: usize, now: Instant) {
        let slot = &self.slots[idx];
        let Some(until) = slot.cooldown_until else { return };
        let left =
            i64::try_from(until.saturating_duration_since(now).as_secs()).unwrap_or(i64::MAX);
        let at = self.shared.clock.now_secs().saturating_add(left);
        self.shared.with_hub(|hub| hub.set_next_attempt(slot.id, Some(at)));
    }
}
