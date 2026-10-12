//! Per-harness scheduling state and its transitions. Pure: every method takes `now`, so the rules
//! are testable with plain instants and no runtime.

use std::sync::Arc;
use std::time::Duration;

use tokio::task::AbortHandle;
use tokio::time::Instant;
use usage_model::{ErrorCode, HarnessId, HarnessView, ViewState};

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
    /// While set and ahead, the source answered "too many requests" and is left alone.
    pub(super) cooldown_until: Option<Instant>,
    /// Healthy reads in a row that changed nothing; drives the adaptive stretch.
    pub(super) idle_ticks: u32,
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
    /// The source said "too many requests": a cool-down, honouring its `Retry-After`.
    Limited {
        retry_after_secs: Option<u64>,
    },
}

pub(super) fn judge(view: &HarnessView) -> Verdict {
    match view.state {
        ViewState::Error
            if view.error.as_ref().is_some_and(|e| e.code == ErrorCode::RateLimited) =>
        {
            let asked = view.error.as_ref().and_then(|e| e.params.get("retryAfterSecs"));
            Verdict::Limited { retry_after_secs: asked.and_then(|secs| secs.parse().ok()) }
        }
        ViewState::Error => {
            Verdict::Failed { retryable: view.error.as_ref().is_none_or(|e| e.retryable) }
        }
        ViewState::NeedsConsent | ViewState::NeedsSetup | ViewState::Disabled => Verdict::Parked,
        ViewState::Loading | ViewState::Ready | ViewState::NoData => Verdict::Healthy,
    }
}

impl Slot {
    pub(super) fn new(id: HarnessId, collector: Option<Arc<dyn Collector>>, enabled: bool) -> Self {
        Self {
            id,
            collector,
            enabled,
            due: None,
            failures: 0,
            last_start: None,
            running: None,
            cooldown_until: None,
            idle_ticks: 0,
        }
    }

    pub(super) fn is_due(&self, now: Instant) -> bool {
        self.enabled && self.running.is_none() && self.due.is_some_and(|due| due <= now)
    }

    /// A manual refresh is dropped for a harness that is off, busy or was collected moments ago.
    pub(super) fn may_refresh(&self, now: Instant) -> bool {
        self.enabled
            && self.collector.is_some()
            && self.running.is_none()
            && !self.cooling(now)
            && self.last_start.is_none_or(|at| now.duration_since(at) >= REFRESH_MIN_GAP)
    }

    /// A rate-limit cool-down still holds this source back.
    pub(super) fn cooling(&self, now: Instant) -> bool {
        self.cooldown_until.is_some_and(|until| now < until)
    }

    /// Forgets failures and any cool-down: something the user did makes the next call a fresh try.
    pub(super) fn reset_failures(&mut self) {
        self.failures = 0;
        self.cooldown_until = None;
    }

    /// A collection ended: schedule the next one from how it went.
    pub(super) fn settle(&mut self, verdict: Verdict, cadence: Duration, now: Instant) {
        self.running = None;
        self.cooldown_until = None;
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
            Verdict::Limited { retry_after_secs } => {
                self.failures = self.failures.saturating_add(1);
                let wait = timing::cooldown(cadence, self.failures, retry_after_secs);
                self.due = Some(now + wait);
                self.cooldown_until = Some(now + wait);
            }
        }
    }

    /// Counts a healthy read that changed nothing (or restarts the count when it did).
    pub(super) fn note_read(&mut self, changed: bool) {
        self.idle_ticks = if changed { 0 } else { self.idle_ticks.saturating_add(1) };
    }

    /// Replaces the wait after a healthy read with a longer, adaptive one.
    pub(super) fn stretch_to(&mut self, wait: Duration, now: Instant) {
        if self.failures == 0 && self.due.is_some() {
            self.due = Some(now + wait);
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
