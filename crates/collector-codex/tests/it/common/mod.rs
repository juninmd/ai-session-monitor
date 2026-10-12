//! Shared fixtures for the integration tests.

pub mod process;
pub mod rig;
pub mod rollout;

pub use rig::{FIVE_HOUR_RESET, NOW, PATIENT, Rig, WEEKLY_RESET};

use usage_model::{ErrorCode, HarnessError, HarnessView, ViewState};

/// The error of a view that failed, with the shape every failure view has.
pub fn failure(view: &HarnessView) -> &HarnessError {
    assert_eq!(view.state, ViewState::Error, "{view:?}");
    assert!(view.windows.is_empty() && view.facts.is_empty() && view.notices.is_empty());
    assert!(view.source.is_none() && view.plan.is_none() && view.read_at.is_none());
    view.error.as_ref().unwrap()
}

/// Asserts the failure's code and retry flag and returns its params.
pub fn assert_failure(view: &HarnessView, code: ErrorCode, retryable: bool) -> &HarnessError {
    let error = failure(view);
    assert_eq!((error.code, error.retryable), (code, retryable), "{error:?}");
    error
}
