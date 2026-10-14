//! State shared by the service API and the scheduler loop. Every lock here is held for a few
//! field updates and never across an `await` or a sink call.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use tokio::sync::{Notify, watch};
use usage_model::{Ctx, HarnessError, HarnessId, Settings};

use crate::clock::Clock;
use crate::collectors::{Adapters, ClaudeShim, Collector};
use crate::history::HistoryStore;
use crate::hub::Hub;
use crate::paths::AppPaths;
use crate::relay::Relay;
use crate::secrets::SecretStore;
use crate::service::EventSink;
use crate::store::SettingsStore;
use crate::sync::lock;

pub(crate) const HARNESSES: usize = HarnessId::ALL.len();

/// Proof that this caller owns the scheduler loop. Released on drop, so a normal end, a panic
/// unwinding through `run` and a cancelled `run` future all free it for a restart.
pub(crate) struct RunClaim<'a>(&'a AtomicBool);

impl Drop for RunClaim<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

/// Collections the API asked for since the scheduler last looked, per harness.
#[derive(Clone, Copy, Default)]
pub(crate) struct Requests {
    /// From `refresh`: dropped when the harness was collected moments ago.
    pub(crate) manual: [bool; HARNESSES],
    /// From a secret or settings change: the data in flight is outdated, so always honoured.
    pub(crate) forced: [bool; HARNESSES],
}

pub(crate) struct Shared {
    pub(crate) paths: AppPaths,
    pub(crate) secrets: Arc<dyn SecretStore>,
    pub(crate) sink: Arc<dyn EventSink>,
    pub(crate) shim: Arc<dyn ClaudeShim>,
    pub(crate) clock: Arc<dyn Clock>,
    pub(crate) store: SettingsStore,
    pub(crate) history: HistoryStore,
    /// Orders `settings_changed` events across threads (see `Relay`).
    pub(crate) relay: Relay,
    /// Wakes the scheduler: a request is pending or the hub changed.
    pub(crate) wake: Notify,
    collectors: Vec<Option<Arc<dyn Collector>>>,
    settings: watch::Sender<Arc<Settings>>,
    hub: Mutex<Hub>,
    requests: Mutex<Requests>,
    stop: watch::Sender<bool>,
    running: AtomicBool,
    /// File name of the set-aside settings copy, until the scheduler has announced it.
    recovered: Mutex<Option<String>>,
}

impl Shared {
    pub(crate) fn new(
        paths: AppPaths,
        secrets: Arc<dyn SecretStore>,
        sink: Arc<dyn EventSink>,
        adapters: Adapters,
        clock: Arc<dyn Clock>,
    ) -> Result<Self, HarnessError> {
        let store = SettingsStore::new(paths.settings_file());
        let loaded = store.load(clock.now_secs())?;
        let history = HistoryStore::load(paths.history_file(), clock.now_secs());
        let hub = Hub::new(&loaded.settings);
        let Adapters { collectors, claude_shim } = adapters;
        let mut by_harness: Vec<Option<Arc<dyn Collector>>> = vec![None; HARNESSES];
        for collector in collectors {
            let slot = &mut by_harness[collector.id() as usize];
            if slot.is_none() {
                *slot = Some(collector);
            }
        }
        Ok(Self {
            paths,
            secrets,
            sink,
            shim: claude_shim,
            clock,
            store,
            history,
            relay: Relay::default(),
            wake: Notify::new(),
            collectors: by_harness,
            settings: watch::channel(Arc::new(loaded.settings)).0,
            hub: Mutex::new(hub),
            requests: Mutex::new(Requests::default()),
            stop: watch::channel(false).0,
            running: AtomicBool::new(false),
            recovered: Mutex::new(loaded.recovered),
        })
    }

    /// The settings every collector call and every decision uses; always the latest saved.
    pub(crate) fn settings(&self) -> Arc<Settings> {
        Arc::clone(&self.settings.borrow())
    }

    pub(crate) fn subscribe_settings(&self) -> watch::Receiver<Arc<Settings>> {
        self.settings.subscribe()
    }

    /// Publishes saved settings to the scheduler and brings the enabled/disabled views in line.
    pub(crate) fn apply_settings(&self, saved: Arc<Settings>) {
        lock(&self.hub).reconcile_enabled(&saved);
        self.settings.send_replace(saved);
        self.wake.notify_one();
    }

    pub(crate) fn collector(&self, id: HarnessId) -> Option<Arc<dyn Collector>> {
        self.collectors[id as usize].clone()
    }

    pub(crate) fn with_hub<R>(&self, work: impl FnOnce(&mut Hub) -> R) -> R {
        work(&mut lock(&self.hub))
    }

    pub(crate) fn ctx(&self) -> Ctx {
        Ctx::new(self.clock.now_secs(), self.paths.data_dir.clone(), self.paths.home_dir.clone())
    }

    /// `None` asks for every harness.
    pub(crate) fn request_refresh(&self, harness: Option<HarnessId>) {
        {
            let mut requests = lock(&self.requests);
            match harness {
                Some(id) => requests.manual[id as usize] = true,
                None => requests.manual = [true; HARNESSES],
            }
        }
        self.wake.notify_one();
    }

    pub(crate) fn request_forced(&self, harness: HarnessId) {
        lock(&self.requests).forced[harness as usize] = true;
        self.wake.notify_one();
    }

    pub(crate) fn take_requests(&self) -> Requests {
        std::mem::take(&mut *lock(&self.requests))
    }

    /// The set-aside copy still to be announced; stays until `announced_recovery`, so a loop that
    /// dies while delivering the notice leaves it for the next one.
    pub(crate) fn pending_recovery(&self) -> Option<String> {
        lock(&self.recovered).clone()
    }

    pub(crate) fn announced_recovery(&self) {
        lock(&self.recovered).take();
    }

    /// `Some` for the one caller that may run the scheduler loop; dropping the claim, however
    /// the loop ended, lets the next `run` take over.
    pub(crate) fn claim_run(&self) -> Option<RunClaim<'_>> {
        (!self.running.swap(true, Ordering::SeqCst)).then_some(RunClaim(&self.running))
    }

    pub(crate) fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub(crate) fn request_stop(&self) {
        self.stop.send_replace(true);
    }

    pub(crate) fn resume_after_failed_update(&self) -> bool {
        if self.is_running() {
            return false;
        }
        self.stop.send_replace(false);
        true
    }

    pub(crate) fn subscribe_stop(&self) -> watch::Receiver<bool> {
        self.stop.subscribe()
    }
}
