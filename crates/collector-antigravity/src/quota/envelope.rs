//! The `code` some responses carry next to their payload (`{"code": 16}` is an answer, not data).

use serde_json::Value;
use usage_model::{ErrorCode, HarnessError};

use super::values::slug;
use crate::errors::endpoint_changed;

const MAX_CODE_TEXT: usize = 32;
/// The status names gRPC and Connect define; any other text a server sends is not repeated.
const STATUS_NAMES: [&str; 16] = [
    "canceled",
    "unknown",
    "invalid_argument",
    "deadline_exceeded",
    "not_found",
    "already_exists",
    "permission_denied",
    "resource_exhausted",
    "failed_precondition",
    "aborted",
    "out_of_range",
    "unimplemented",
    "internal",
    "unavailable",
    "data_loss",
    "unauthenticated",
];

/// Ok when there is no `code` or it says success; otherwise the error it stands for. Numbers are
/// gRPC status codes and strings their names, as Connect writes them.
pub(super) fn check_code(body: &Value) -> Result<(), HarnessError> {
    match body.get("code") {
        None | Some(Value::Null) => Ok(()),
        Some(Value::Number(number)) => match number.as_i64() {
            Some(0) => Ok(()),
            Some(4) => Err(HarnessError::new(ErrorCode::Timeout, true)),
            Some(7 | 16) => Err(unauthorized()),
            Some(14) => Err(HarnessError::new(ErrorCode::ServerError, true)),
            Some(other) => Err(unexpected(Some(&other.to_string()))),
            None => Err(endpoint_changed()),
        },
        Some(Value::String(name)) => match slug(name, MAX_CODE_TEXT).as_deref() {
            None | Some("0" | "ok" | "success") => Ok(()),
            Some("unauthenticated" | "permission_denied" | "unauthorized") => Err(unauthorized()),
            Some("unavailable") => Err(HarnessError::new(ErrorCode::ServerError, true)),
            Some("deadline_exceeded") => Err(HarnessError::new(ErrorCode::Timeout, true)),
            Some(other) => Err(unexpected(STATUS_NAMES.contains(&other).then_some(other))),
        },
        Some(_) => Err(endpoint_changed()),
    }
}

fn unauthorized() -> HarnessError {
    HarnessError::new(ErrorCode::Unauthorized, false)
}

/// A number or a defined status name is safe to show; anything else the server wrote is not.
fn unexpected(code: Option<&str>) -> HarnessError {
    code.map_or_else(endpoint_changed, |code| endpoint_changed().with_param("code", code))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn code_of(body: &Value) -> Option<(ErrorCode, bool)> {
        check_code(body).err().map(|error| (error.code, error.retryable))
    }

    #[test]
    fn success_and_absence_pass() {
        for body in [
            json!({}),
            json!({ "code": null }),
            json!({ "code": 0 }),
            json!({ "code": "0" }),
            json!({ "code": "OK" }),
            json!({ "code": "success" }),
            json!({ "code": "" }),
            json!([]),
            json!(null),
        ] {
            assert_eq!(code_of(&body), None, "{body}");
        }
    }

    #[test]
    fn refusals_are_unauthorized_and_final() {
        for body in [
            json!({ "code": 16 }),
            json!({ "code": 7 }),
            json!({ "code": "unauthenticated" }),
            json!({ "code": "PERMISSION_DENIED" }),
        ] {
            assert_eq!(code_of(&body), Some((ErrorCode::Unauthorized, false)), "{body}");
        }
    }

    #[test]
    fn transient_statuses_may_heal_by_themselves() {
        assert_eq!(code_of(&json!({ "code": 14 })), Some((ErrorCode::ServerError, true)));
        assert_eq!(
            code_of(&json!({ "code": "unavailable" })),
            Some((ErrorCode::ServerError, true))
        );
        assert_eq!(code_of(&json!({ "code": 4 })), Some((ErrorCode::Timeout, true)));
        assert_eq!(
            code_of(&json!({ "code": "deadline_exceeded" })),
            Some((ErrorCode::Timeout, true))
        );
    }

    #[test]
    fn any_other_code_means_the_endpoint_does_not_work_as_known() {
        for (body, shown) in [
            (json!({ "code": 5 }), "5"),
            (json!({ "code": -1 }), "-1"),
            (json!({ "code": "not_found" }), "not_found"),
            (json!({ "code": "UNIMPLEMENTED" }), "unimplemented"),
        ] {
            let error = check_code(&body).unwrap_err();
            assert_eq!(error.code, ErrorCode::EndpointChanged, "{body}");
            assert!(!error.retryable);
            assert_eq!(error.params.get("code").map(String::as_str), Some(shown));
        }
        for body in [json!({ "code": true }), json!({ "code": 1.5 }), json!({ "code": {} })] {
            assert_eq!(code_of(&body), Some((ErrorCode::EndpointChanged, false)), "{body}");
        }
    }

    #[test]
    fn a_code_name_nobody_defined_is_not_repeated() {
        for name in [
            "x".repeat(500) + " <script> token=abc",
            "fake-csrf-token-0001".to_owned(),
            "teapot".to_owned(),
        ] {
            let error = check_code(&json!({ "code": name })).unwrap_err();
            assert_eq!((error.code, error.retryable), (ErrorCode::EndpointChanged, false));
            assert!(error.params.is_empty());
        }
    }

    #[test]
    fn every_defined_status_name_is_short_enough_to_survive_the_bound() {
        for name in STATUS_NAMES {
            assert!(name.len() <= MAX_CODE_TEXT, "{name}");
        }
    }
}
