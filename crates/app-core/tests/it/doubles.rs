//! Small stand-ins for the service's other seams: two clocks and broken secret stores.

use std::sync::atomic::{AtomicUsize, Ordering};

use app_core::secrets::store_error;
use app_core::{Clock, MemorySecretStore, SecretStore};
use tokio::time::Instant;
use usage_model::{HarnessError, SecretKind};
use usage_support::Secret;

use crate::views::BASE;

/// Wall clock that moves with tokio's virtual time.
pub struct VirtualClock {
    origin: Instant,
}

impl VirtualClock {
    pub fn new() -> Self {
        Self { origin: Instant::now() }
    }
}

impl Clock for VirtualClock {
    fn now_secs(&self) -> i64 {
        BASE + i64::try_from(self.origin.elapsed().as_secs()).unwrap_or(0)
    }
}

/// For tests on plain threads, where nothing needs time to pass.
pub struct FixedClock;

impl Clock for FixedClock {
    fn now_secs(&self) -> i64 {
        BASE
    }
}

/// A secret store that always fails, like a locked-down Credential Manager.
pub struct FailingStore;

impl SecretStore for FailingStore {
    fn get(&self, _kind: SecretKind) -> Result<Option<Secret>, HarnessError> {
        Err(store_error())
    }

    fn set(&self, _kind: SecretKind, _value: &str) -> Result<(), HarnessError> {
        Err(store_error())
    }

    fn delete(&self, _kind: SecretKind) -> Result<(), HarnessError> {
        Err(store_error())
    }
}

/// A store that works except for reading one kind, like a credential with a bad encoding.
pub struct OneUnreadable {
    pub unreadable: SecretKind,
    inner: MemorySecretStore,
}

impl OneUnreadable {
    pub fn new(unreadable: SecretKind) -> Self {
        Self { unreadable, inner: MemorySecretStore::new() }
    }
}

impl SecretStore for OneUnreadable {
    fn get(&self, kind: SecretKind) -> Result<Option<Secret>, HarnessError> {
        if kind == self.unreadable { Err(store_error()) } else { self.inner.get(kind) }
    }

    fn set(&self, kind: SecretKind, value: &str) -> Result<(), HarnessError> {
        self.inner.set(kind, value)
    }

    fn delete(&self, kind: SecretKind) -> Result<(), HarnessError> {
        self.inner.delete(kind)
    }
}

/// Reads and writes work; deleting does not, like a credential another process holds open.
#[derive(Default)]
pub struct StuckDelete {
    inner: MemorySecretStore,
    deletes: AtomicUsize,
}

impl StuckDelete {
    /// How many times a delete was tried.
    pub fn attempts(&self) -> usize {
        self.deletes.load(Ordering::SeqCst)
    }
}

impl SecretStore for StuckDelete {
    fn get(&self, kind: SecretKind) -> Result<Option<Secret>, HarnessError> {
        self.inner.get(kind)
    }

    fn set(&self, kind: SecretKind, value: &str) -> Result<(), HarnessError> {
        self.inner.set(kind, value)
    }

    fn delete(&self, _kind: SecretKind) -> Result<(), HarnessError> {
        self.deletes.fetch_add(1, Ordering::SeqCst);
        Err(store_error())
    }
}
