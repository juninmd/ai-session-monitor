//! Per-harness scheduling state and its transitions. Pure: every method takes `now`, so the rules
//! are testable with plain instants and no runtime.

use std::sync::Arc;
use std::time::Duration;

use tokio::task::AbortHandle;
use tokio::time::Instant;
use usage_model::{HarnessId, HarnessView, ViewState};

use crate::collectors::Collector;
use crate::timing::{self, REFRESH_MIN_GAP};

pub(super) struct Slot {
    pub(super) id: HarnessId,
    pub(super) collector: Option<Arc<dyn Collector>>,
    pub(super) enabled: bool,
    /// When the next scheduled collection starts. `None` while one runs, when the source waits
    /// for consent or setup (nothing to poll until a setting or secret changes), or when disabled.
    pub(super) due: Option<Instant>,
    pub(super) failures: u32,
    pub(super) last_start: Option<Instant>,
    pub(super) running: Option<AbortHandle>,
}

/// What a finished collection means for the schedule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Verdict {
    Healthy,
    /// Needs consent or setup: only the user can change that, so polling would be noise.
    Parked,
    Failed {
        retryable: bool,
    },
}

pub(super) fn judge(view: &HarnessView) -> Verdict {
    match view.state {
        ViewState::Error => {
            Verdict::Failed { retryable: view.error.as_ref().is_none_or(|e| e.retryable) }
        }
        ViewState::NeedsConsent | ViewState::NeedsSetup | ViewState::Disabled => Verdict::Parked,
        ViewState::Loading | ViewState::Ready | ViewState::NoData => Verdict::Healthy,
    }
}

impl Slot {
    pub(super) fn new(id: HarnessId, collector: Option<Arc<dyn Collector>>, enabled: bool) -> Self {
        Self { id, collector, enabled, due: None, failures: 0, last_start: None, running: None }
    }

    pub(super) fn is_due(&self, now: Instant) -> bool {
        self.enabled && self.running.is_none() && self.due.is_some_and(|due| due <= now)
    }

    /// A manual refresh is dropped for a harness that is off, busy or was collected moments ago.
    pub(super) fn may_refresh(&self, now: Instant) -> bool {
        self.enabled
            && self.collector.is_some()
            && self.running.is_none()
            && self.last_start.is_none_or(|at| now.duration_since(at) >= REFRESH_MIN_GAP)
    }

    /// A collection ended: schedule the next one from how it went.
    pub(super) fn settle(&mut self, verdict: Verdict, cadence: Duration, now: Instant) {
        self.running = None;
        match verdict {
            Verdict::Healthy => {
                self.failures = 0;
                self.due = Some(now + cadence);
            }
            Verdict::Parked => {
                self.failures = 0;
                self.due = None;
            }
            Verdict::Failed { retryable } => {
                self.failures = self.failures.saturating_add(1);
                self.due = Some(now + timing::backoff(cadence, self.failures, retryable));
            }
        }
    }

    /// A new interval applies to the next call of a healthy harness, counted from its last start.
    pub(super) fn retime(&mut self, before: Duration, after: Duration, now: Instant) {
        if before == after || self.failures > 0 {
            return;
        }
        if let (Some(_), Some(started)) = (self.due, self.last_start) {
            self.due = Some((started + after).max(now));
        }
    }
}

#[cfg(test)]
mod tests;
