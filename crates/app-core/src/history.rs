//! Quota burndown history: samples every window with a known used percentage into a small local
//! file so the UI can draw past windows. Stores percentages and times only, keyed by harness and
//! window id: no account, plan, path, label or error text. Kept 35 days, capped in points and
//! bytes, written at most once a minute and on shutdown.

use std::path::PathBuf;
use std::sync::Mutex;

use usage_model::history::MAX_EPOCHS_PER_WINDOW;
use usage_model::{ErrorCode, HarnessError, HarnessHistory, HarnessId, HarnessView, ViewState};

use crate::sync::lock;

mod disk;
mod ledger;
mod query;

use ledger::Ledger;

/// Disk writes are at least this far apart; a reading in between only marks the ledger dirty.
const FLUSH_GAP_SECS: i64 = 60;

struct State {
    ledger: Ledger,
    dirty: bool,
    last_flush: i64,
}

pub(crate) struct HistoryStore {
    path: PathBuf,
    state: Mutex<State>,
}

fn write_failed() -> HarnessError {
    HarnessError::new(ErrorCode::Internal, true).with_param("where", "history_write")
}

impl HistoryStore {
    pub(crate) fn load(path: PathBuf, now_secs: i64) -> Self {
        let ledger = disk::load(&path, now_secs);
        Self { path, state: Mutex::new(State { ledger, dirty: false, last_flush: now_secs }) }
    }

    /// Samples a fresh reading. Failed, stale and empty views carry no new measurement.
    pub(crate) fn record(&self, view: &HarnessView, now_secs: i64) {
        if view.state != ViewState::Ready || view.stale {
            return;
        }
        let mut state = lock(&self.state);
        if state.ledger.record(view.id, &view.windows, now_secs) {
            state.ledger.prune(now_secs);
            state.dirty = true;
        }
    }

    /// Writes when something changed and the last write is old enough. A failure keeps the data
    /// in memory and the dirty mark, so the next call retries.
    pub(crate) fn flush_if_due(&self, now_secs: i64) {
        let mut state = lock(&self.state);
        if state.dirty && now_secs - state.last_flush >= FLUSH_GAP_SECS {
            self.write(&mut state, now_secs);
        }
    }

    /// For shutdown: writes whatever is pending, regardless of the gap.
    pub(crate) fn flush(&self, now_secs: i64) {
        let mut state = lock(&self.state);
        if state.dirty {
            self.write(&mut state, now_secs);
        }
    }

    fn write(&self, state: &mut State, now_secs: i64) {
        if disk::save(&self.path, &mut state.ledger).is_ok() {
            state.dirty = false;
        }
        state.last_flush = now_secs;
    }

    /// The newest `limit` epochs of each window of `harness`.
    pub(crate) fn query(&self, harness: HarnessId, limit: u8) -> HarnessHistory {
        let limit = usize::from(limit.clamp(1, MAX_EPOCHS_PER_WINDOW));
        lock(&self.state).ledger.epochs(harness, limit)
    }

    /// Forgets everything, in memory and on disk (the set-aside copy included).
    pub(crate) fn clear(&self, now_secs: i64) -> Result<(), HarnessError> {
        let mut state = lock(&self.state);
        state.ledger = Ledger::default();
        state.dirty = false;
        state.last_flush = now_secs;
        let removed = remove_if_present(&self.path)
            .and_then(|()| remove_if_present(&self.path.with_file_name(disk::QUARANTINE_NAME)));
        removed.map_err(|_| write_failed())
    }
}

fn remove_if_present(path: &std::path::Path) -> std::io::Result<()> {
    match std::fs::remove_file(path) {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => Err(error),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod ledger_tests;
#[cfg(test)]
mod tests;
