//! The notice code this collector emits (catalog in docs/ipc.md) and its constructor.

use std::collections::BTreeMap;

use usage_model::{Notice, Severity};

/// The window resets at the first instant of the next UTC month because the response carried no
/// usable reset date: GitHub documents the cadence, the instant is derived, not reported.
pub(crate) const RESET_DERIVED: &str = "copilot_reset_derived";

pub(crate) fn reset_derived() -> Notice {
    Notice { code: RESET_DERIVED.to_owned(), severity: Severity::Info, params: BTreeMap::new() }
}
