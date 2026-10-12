//! The notice codes this collector emits (catalog in docs/ipc.md) and their constructor.

use std::collections::BTreeMap;

use usage_model::{Notice, Severity};

pub(crate) const WINDOW_IDLE: &str = "opencode_window_idle";
pub(crate) const RATE_LIMITED: &str = "opencode_rate_limited";
pub(crate) const NO_KEY: &str = "opencode_no_key";
pub(crate) const LOCAL_ESTIMATE: &str = "opencode_local_estimate";
pub(crate) const DB_UNREADABLE: &str = "opencode_db_unreadable";

/// Params are fixed identifiers (`rolling`, `locked`, ...), never database or response text.
pub(crate) fn make(code: &str, severity: Severity, params: &[(&str, &str)]) -> Notice {
    let params: BTreeMap<String, String> =
        params.iter().map(|&(key, value)| (key.to_owned(), value.to_owned())).collect();
    Notice { code: code.to_owned(), severity, params }
}
