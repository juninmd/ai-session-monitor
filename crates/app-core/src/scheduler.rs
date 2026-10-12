//! The polling loop: one task per collector call, a cadence and a backoff per harness, debounced
//! manual refresh, live reaction to settings changes and a prompt, clean shutdown.
//!
//! Only this task touches the slots. The API reaches it through `Shared` (requests and the
//! settings watch), so no lock is ever held while a collector runs or a sink is called.

mod slot;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::watch;
use tokio::task::{Id, JoinError, JoinSet};
use tokio::time::{Instant, sleep_until, timeout};
use usage_model::{Ctx, ErrorCode, HarnessError, HarnessId, HarnessView, Settings, ViewState};

use self::slot::{Slot, judge};
use crate::collectors::Collector;
use crate::hub::Counts;
use crate::notify;
use crate::publish::Publisher;
use crate::secrets::SecretStore;
use crate::shared::{Requests, Shared};
use crate::timing::{self, COLLECT_TIMEOUT};

/// A far-away reset time must not overflow `Instant`; the loop re-arms at every wake anyway.
const MAX_EXPIRY_WAKE: Duration = Duration::from_secs(24 * 3600);
/// How long `run` waits for cancelled collections to unwind before it stops caring.
const STOP_GRACE: Duration = Duration::from_secs(2);

pub(crate) struct Scheduler {
    shared: Arc<Shared>,
    settings: watch::Receiver<Arc<Settings>>,
    /// The settings the slots were last reconciled with.
    applied: Arc<Settings>,
    slots: Vec<Slot>,
    tasks: JoinSet<HarnessView>,
    /// Task id to slot index; a task missing here was cancelled and its result is stale.
    running: HashMap<Id, usize>,
    publisher: Publisher,
    /// The hub's switch-on counts at the last reconcile (see `Hub::switch_ons`).
    seen_switch_ons: Counts,
}

impl Scheduler {
    pub(crate) fn new(shared: Arc<Shared>) -> Self {
        let settings = shared.subscribe_settings();
        let applied = shared.settings();
        let slots = HarnessId::ALL
            .iter()
            .map(|&id| Slot::new(id, shared.collector(id), applied.is_enabled(id)))
            .collect();
        let seen_switch_ons = shared.with_hub(|hub| hub.switch_ons());
        Self {
            shared,
            settings,
            applied,
            slots,
            tasks: JoinSet::new(),
            running: HashMap::new(),
            publisher: Publisher::default(),
            seen_switch_ons,
        }
    }

    pub(crate) async fn run(mut self) {
        let mut stop = self.shared.subscribe_stop();
        if *stop.borrow() {
            return;
        }
        self.announce_recovery();
        for slot in self.slots.iter_mut().filter(|slot| slot.enabled) {
            slot.due = Some(Instant::now());
        }
        loop {
            self.publisher.publish(&self.shared);
            let deadline = self.next_deadline();
            let alarm = async move {
                match deadline {
                    Some(at) => sleep_until(at).await,
                    None => std::future::pending().await,
                }
            };
            tokio::select! {
                biased;
                _ = stop.changed() => break,
                Some(joined) = self.tasks.join_next_with_id() => self.finished(joined),
                Ok(()) = self.settings.changed() => self.reconcile(),
                () = self.shared.wake.notified() => self.serve_requests(),
                () = alarm => self.tick(),
            }
        }
        // Dropping the tasks is not enough to say they are gone; wait, but only briefly.
        let _ = timeout(STOP_GRACE, self.tasks.shutdown()).await;
    }

    fn announce_recovery(&self) {
        if let Some(file_name) = self.shared.pending_recovery() {
            self.shared.sink.notify(&notify::settings_recovered(&file_name));
            self.shared.announced_recovery();
        }
    }

    fn next_deadline(&self) -> Option<Instant> {
        let due = self.slots.iter().filter_map(|slot| slot.due);
        due.chain(self.publisher.flush_at()).chain(self.clock_deadline()).min()
    }

    /// The moment the views change on their own, so the UI hears about it without a poll: a ready
    /// view going stale, or a window that ran out coming back (see `Publisher::next_lapse`).
    fn clock_deadline(&self) -> Option<Instant> {
        let now_secs = self.shared.clock.now_secs();
        let expiry = self.shared.with_hub(|hub| hub.next_expiry(now_secs));
        let lapse = self.publisher.next_lapse(&self.shared.settings(), now_secs);
        let at = expiry.into_iter().chain(lapse).min()?;
        let wait = Duration::from_secs(u64::try_from(at.saturating_sub(now_secs)).unwrap_or(1));
        Some(Instant::now() + wait.min(MAX_EXPIRY_WAKE))
    }

    fn tick(&mut self) {
        let now = Instant::now();
        for idx in 0..self.slots.len() {
            if self.slots[idx].is_due(now) {
                self.start(idx, now);
            }
        }
    }

    fn start(&mut self, idx: usize, now: Instant) {
        let id = self.slots[idx].id;
        let Some(collector) = self.slots[idx].collector.clone() else {
            // Built without a collector for this harness: nothing to read, nothing to poll.
            self.slots[idx].due = None;
            self.shared.with_hub(|hub| hub.apply(HarnessView::with_state(id, ViewState::NoData)));
            return;
        };
        let call = collect(
            collector,
            self.shared.settings(),
            Arc::clone(&self.shared.secrets),
            self.shared.ctx(),
            id,
        );
        let handle = self.tasks.spawn(call);
        self.running.insert(handle.id(), idx);
        let slot = &mut self.slots[idx];
        slot.running = Some(handle);
        slot.due = None;
        slot.last_start = Some(now);
    }

    fn cancel(&mut self, idx: usize) {
        if let Some(handle) = self.slots[idx].running.take() {
            self.running.remove(&handle.id());
            handle.abort();
        }
    }

    /// Collects now, abandoning a call that started with outdated settings or secrets.
    fn restart(&mut self, idx: usize, now: Instant) {
        self.cancel(idx);
        if self.slots[idx].enabled {
            self.start(idx, now);
        }
    }

    fn finished(&mut self, joined: Result<(Id, HarnessView), JoinError>) {
        let (task, result) = match joined {
            Ok((task, view)) => (task, Some(view)),
            Err(error) => (error.id(), None),
        };
        let Some(idx) = self.running.remove(&task) else { return };
        let id = self.slots[idx].id;
        // No result for a live task means the collector panicked.
        let view = result.unwrap_or_else(|| {
            let error =
                HarnessError::new(ErrorCode::Internal, true).with_param("where", "collector");
            HarnessView::error(id, error)
        });
        let verdict = judge(&view);
        self.shared.with_hub(|hub| hub.apply(view));
        let cadence = timing::cadence(id, &self.shared.settings());
        self.slots[idx].settle(verdict, cadence, Instant::now());
    }

    fn serve_requests(&mut self) {
        if self.settings.has_changed().unwrap_or(false) {
            self.reconcile();
        }
        let Requests { manual, forced } = self.shared.take_requests();
        let now = Instant::now();
        for idx in 0..self.slots.len() {
            if forced[idx] {
                self.slots[idx].failures = 0;
                self.restart(idx, now);
            } else if manual[idx] && self.slots[idx].may_refresh(now) {
                self.start(idx, now);
            }
        }
    }

    /// Applies new settings: switched-off harnesses stop, switched-on ones start, a changed
    /// section of a running harness is read again, and a changed interval re-times the next call.
    fn reconcile(&mut self) {
        let new = Arc::clone(&self.settings.borrow_and_update());
        let old = std::mem::replace(&mut self.applied, Arc::clone(&new));
        let switch_ons = self.shared.with_hub(|hub| {
            hub.reconcile_enabled(&new);
            hub.switch_ons()
        });
        let now = Instant::now();
        let switched_on = std::mem::replace(&mut self.seen_switch_ons, switch_ons);
        for (idx, seen) in switched_on.into_iter().enumerate() {
            let id = self.slots[idx].id;
            let enabled = new.is_enabled(id);
            self.slots[idx].enabled = enabled;
            if !enabled {
                self.cancel(idx);
                let slot = &mut self.slots[idx];
                slot.due = None;
                slot.failures = 0;
            } else if seen != switch_ons[idx]
                || !old.is_enabled(id)
                || section_changed(id, &old, &new)
            {
                self.slots[idx].failures = 0;
                self.restart(idx, now);
            } else {
                let (before, after) = (timing::cadence(id, &old), timing::cadence(id, &new));
                self.slots[idx].retime(before, after, now);
            }
        }
    }
}

fn section_changed(id: HarnessId, old: &Settings, new: &Settings) -> bool {
    match id {
        HarnessId::Claude => old.claude != new.claude,
        HarnessId::Codex => old.codex != new.codex,
        HarnessId::Antigravity => old.antigravity != new.antigravity,
        HarnessId::Opencode => old.opencode != new.opencode,
        HarnessId::Litellm => old.litellm != new.litellm,
    }
}

/// One collector call with its own deadline, so a hung source never blocks the others.
async fn collect(
    collector: Arc<dyn Collector>,
    settings: Arc<Settings>,
    secrets: Arc<dyn SecretStore>,
    ctx: Ctx,
    id: HarnessId,
) -> HarnessView {
    let call = collector.collect(&settings, secrets.as_ref(), &ctx);
    match timeout(COLLECT_TIMEOUT, call).await {
        // A view for another tab would overwrite that tab's data: a collector bug, never retried.
        Ok(view) if view.id != id => {
            let error =
                HarnessError::new(ErrorCode::Internal, false).with_param("where", "collector_id");
            HarnessView::error(id, error)
        }
        Ok(view) => view,
        Err(_elapsed) => HarnessView::error(id, HarnessError::new(ErrorCode::Timeout, true)),
    }
}
