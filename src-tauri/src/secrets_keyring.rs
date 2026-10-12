//! `SecretStore` over the OS credential store (the Windows Credential Manager) through
//! keyring-core. Every failure collapses to `store_error()`: the platform error text can carry
//! the secret itself (`BadEncoding` holds the raw bytes), so it is never formatted or logged.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use app_core::SecretStore;
use app_core::secrets::store_error;
use keyring_core::{CredentialStore, Entry, Error as KeyringError};
use usage_model::{ErrorCode, HarnessError, SecretKind};
use usage_support::Secret;

#[cfg(test)]
mod tests;

/// Credential Manager target prefix: entries are named `ai-session-monitor/<kind>`.
pub const SERVICE: &str = "ai-session-monitor";

pub struct KeyringSecretStore {
    store: Arc<CredentialStore>,
    service: String,
    /// The Windows store takes an explicit target name and persistence; the in-memory mock used by
    /// tests refuses modifiers.
    native_modifiers: bool,
    // The platform store does not order operations on one entry across threads.
    gate: Mutex<()>,
}

/// Credential Manager target name of a secret.
pub fn target_name(service: &str, kind: SecretKind) -> String {
    format!("{service}/{}", kind.as_str())
}

impl KeyringSecretStore {
    /// The Windows Credential Manager. Other platforms have no store here: callers fall back.
    pub fn open() -> Result<Self, HarnessError> {
        #[cfg(windows)]
        {
            let store = windows_native_keyring_store::Store::new().map_err(|_| store_error())?;
            Ok(Self::with_store(store, SERVICE, true))
        }
        #[cfg(not(windows))]
        {
            Err(store_error())
        }
    }

    pub fn with_store(store: Arc<CredentialStore>, service: &str, native_modifiers: bool) -> Self {
        Self { store, service: service.to_owned(), native_modifiers, gate: Mutex::new(()) }
    }

    fn entry(&self, kind: SecretKind) -> Result<Entry, HarnessError> {
        let target = target_name(&self.service, kind);
        let modifiers = HashMap::from([("target", target.as_str()), ("persistence", "Local")]);
        let modifiers = self.native_modifiers.then_some(&modifiers);
        self.store.build(&self.service, kind.as_str(), modifiers).map_err(|_| store_error())
    }

    fn locked(&self) -> Result<std::sync::MutexGuard<'_, ()>, HarnessError> {
        self.gate.lock().map_err(|_| store_error())
    }
}

/// Used when the OS store cannot be opened: every call fails closed, so a secret is never parked
/// somewhere unprotected as a fallback.
pub struct UnavailableSecretStore;

impl SecretStore for UnavailableSecretStore {
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

impl SecretStore for KeyringSecretStore {
    fn get(&self, kind: SecretKind) -> Result<Option<Secret>, HarnessError> {
        let _gate = self.locked()?;
        match self.entry(kind)?.get_password() {
            Ok(value) => Ok(Some(Secret::new(value))),
            Err(KeyringError::NoEntry) => Ok(None),
            Err(_) => Err(store_error()),
        }
    }

    fn set(&self, kind: SecretKind, value: &str) -> Result<(), HarnessError> {
        let _gate = self.locked()?;
        match self.entry(kind)?.set_password(value) {
            Ok(()) => Ok(()),
            // The credential blob has a size cap; say the value is unusable, not that storage broke.
            Err(KeyringError::TooLong(..)) => {
                Err(HarnessError::new(ErrorCode::InvalidInput, false).with_param("field", "value"))
            }
            Err(_) => Err(store_error()),
        }
    }

    fn delete(&self, kind: SecretKind) -> Result<(), HarnessError> {
        let _gate = self.locked()?;
        match self.entry(kind)?.delete_credential() {
            Ok(()) | Err(KeyringError::NoEntry) => Ok(()),
            Err(_) => Err(store_error()),
        }
    }
}
