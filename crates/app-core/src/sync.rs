//! Lock helper shared by the core.

use std::sync::{Mutex, MutexGuard, PoisonError};

/// The state behind these mutexes is plain data, so a panic elsewhere must not wedge the app.
pub(crate) fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}
