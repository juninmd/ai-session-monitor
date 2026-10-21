//! Turns hub changes into `snapshot_changed` events: only when the content really changed, at
//! most one per `EMIT_MIN_GAP`, with a trailing emit so the last change is never lost. Also the
//! one place that tells a reset the clock announced, so a reading confirming it stays quiet.

use std::collections::HashSet;

use tokio::time::Instant;
use usage_model::{HarnessView, Settings, Snapshot};

use crate::notify::{self, Notification, PaceState, ResetKey, ThresholdState};
use crate::shared::Shared;
use crate::timing::EMIT_MIN_GAP;

#[derive(Default)]
pub(crate) struct Publisher {
    published: Option<Snapshot>,
    last_emit: Option<Instant>,
    /// When a change held back by the rate limit becomes due.
    flush_at: Option<Instant>,
    /// Resets announced by the clock, kept while the window is still shown out with that reset.
    told: HashSet<ResetKey>,
    thresholds: ThresholdState,
    pace: PaceState,
}

impl Publisher {
    pub(crate) fn flush_at(&self) -> Option<Instant> {
        self.flush_at
    }

    /// The earliest reset of a window shown as out, which only the clock will report.
    pub(crate) fn next_lapse(&self, settings: &Settings, now_secs: i64) -> Option<i64> {
        notify::next_lapse(self.published.as_ref()?, settings, now_secs)
    }

    /// Emits the current snapshot if it differs from the last one emitted and the rate limit
    /// allows; otherwise schedules the emit. The first call always emits (the initial state).
    /// Then tells the resets that came due with no reading, after the snapshot they belong to.
    pub(crate) fn publish(&mut self, shared: &Shared) {
        let now_secs = shared.clock.now_secs();
        let views = shared.with_hub(|hub| hub.views_at(now_secs));
        // Judged against the snapshot the user saw, before the emit below replaces it.
        let lapsed = self.collect_lapsed(shared, &views, now_secs);
        self.emit_changes(shared, views, now_secs);
        for note in &lapsed {
            shared.sink.notify(note);
        }
        if let Some(shown) = &self.published {
            self.told.retain(|key| notify::still_waiting(shown, key));
        }
    }

    /// Resets that came due with no reading behind them, marked told as they are taken.
    fn collect_lapsed(
        &mut self,
        shared: &Shared,
        views: &[HarnessView],
        now_secs: i64,
    ) -> Vec<Notification> {
        let Some(shown) = &self.published else { return Vec::new() };
        let due = notify::lapsed_resets(shown, views, &shared.settings(), now_secs);
        due.into_iter().filter_map(|(key, note)| self.told.insert(key).then_some(note)).collect()
    }

    fn emit_changes(&mut self, shared: &Shared, views: Vec<HarnessView>, now_secs: i64) {
        if self.published.as_ref().is_some_and(|last| last.views == views) {
            self.flush_at = None;
            return;
        }
        let now = Instant::now();
        if let Some(due) = self.last_emit.map(|at| at + EMIT_MIN_GAP)
            && now < due
        {
            self.flush_at = Some(due);
            return;
        }
        let snapshot = Snapshot { generated_at: now_secs, views };
        let settings = shared.settings();
        let mut notes = match &self.published {
            Some(previous) => {
                let planned = notify::plan(previous, &snapshot, &settings, now_secs);
                without_told(&mut self.told, previous, planned)
            }
            None => Vec::new(),
        };
        notes.extend(self.thresholds.plan(self.published.as_ref(), &snapshot, &settings, now_secs));
        notes.extend(self.pace.plan(self.published.as_ref(), &snapshot, &settings, now_secs));
        self.published = Some(snapshot.clone());
        self.last_emit = Some(now);
        self.flush_at = None;
        shared.sink.snapshot_changed(&snapshot);
        for note in &notes {
            shared.sink.notify(note);
        }
    }
}

/// Drops a reset notice for a window whose reset the clock already announced; that mark is spent.
fn without_told(
    told: &mut HashSet<ResetKey>,
    previous: &Snapshot,
    notes: Vec<Notification>,
) -> Vec<Notification> {
    let mut spent =
        |note: &Notification| notify::key_of(previous, note).is_some_and(|key| told.remove(&key));
    notes.into_iter().filter(|note| !spent(note)).collect()
}
