//! Secret storage behind a trait: the shell plugs in the Windows Credential Manager, tests and the
//! browser dev mock use the in-memory store. Secrets are never part of `Settings` or any IPC payload.

use std::collections::HashMap;
use std::sync::Mutex;

use usage_model::{ErrorCode, HarnessError, SecretKind};
use usage_support::Secret;

pub trait SecretStore: Send + Sync {
    fn get(&self, kind: SecretKind) -> Result<Option<Secret>, HarnessError>;
    fn set(&self, kind: SecretKind, value: &str) -> Result<(), HarnessError>;
    /// Deleting a missing secret is not an error.
    fn delete(&self, kind: SecretKind) -> Result<(), HarnessError>;
}

/// Failure of the backing store, without any detail that could carry a secret.
pub fn store_error() -> HarnessError {
    HarnessError::new(ErrorCode::Internal, false).with_param("where", "secret_store")
}

#[derive(Default)]
pub struct MemorySecretStore(Mutex<HashMap<SecretKind, Secret>>);

impl MemorySecretStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl SecretStore for MemorySecretStore {
    fn get(&self, kind: SecretKind) -> Result<Option<Secret>, HarnessError> {
        let map = self.0.lock().map_err(|_| store_error())?;
        Ok(map.get(&kind).cloned())
    }

    fn set(&self, kind: SecretKind, value: &str) -> Result<(), HarnessError> {
        let mut map = self.0.lock().map_err(|_| store_error())?;
        map.insert(kind, Secret::new(value));
        Ok(())
    }

    fn delete(&self, kind: SecretKind) -> Result<(), HarnessError> {
        let mut map = self.0.lock().map_err(|_| store_error())?;
        map.remove(&kind);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_and_deletes_idempotently() {
        let store = MemorySecretStore::new();
        assert!(store.get(SecretKind::Litellm).expect("get").is_none());
        store.set(SecretKind::Litellm, "sk-test").expect("set");
        assert_eq!(store.get(SecretKind::Litellm).expect("get").expect("some").expose(), "sk-test");
        assert!(store.get(SecretKind::Opencode).expect("get").is_none());
        store.delete(SecretKind::Litellm).expect("delete");
        store.delete(SecretKind::Litellm).expect("delete again");
        assert!(store.get(SecretKind::Litellm).expect("get").is_none());
    }
}
