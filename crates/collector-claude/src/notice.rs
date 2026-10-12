//! The notice codes this crate emits. The catalog and the pt-BR texts live in docs/ipc.md and
//! `src/i18n/messages/notices.ts`; no code here is new.

use std::collections::BTreeMap;

use usage_model::{ErrorCode, Notice, Severity};

pub(crate) const WAITING_FIRST_SESSION: &str = "claude_waiting_first_session";
pub(crate) const NO_RATE_LIMITS: &str = "claude_no_rate_limits";
pub(crate) const SHIM_OUTDATED: &str = "claude_shim_outdated";
pub(crate) const OAUTH_PARTIAL: &str = "claude_oauth_partial";

pub(crate) fn plain(code: &str, severity: Severity) -> Notice {
    Notice { code: code.to_owned(), severity, params: BTreeMap::new() }
}

/// The optional OAuth source failed; `reason` is the error code, which the UI turns into words.
pub(crate) fn oauth_partial(reason: ErrorCode) -> Notice {
    let name = serde_json::to_value(reason)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_else(|| "internal".to_owned());
    Notice {
        code: OAUTH_PARTIAL.to_owned(),
        severity: Severity::Info,
        params: BTreeMap::from([("reason".to_owned(), name)]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_reason_is_the_snake_case_error_code() {
        for code in ErrorCode::ALL {
            let notice = oauth_partial(code);
            let expected = serde_json::to_value(code).unwrap();
            assert_eq!(notice.params["reason"], expected.as_str().unwrap());
            assert_eq!((notice.code.as_str(), notice.severity), (OAUTH_PARTIAL, Severity::Info));
        }
    }
}
