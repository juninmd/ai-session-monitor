//! Failures as `HarnessError`s. Params are non-secret by construction: a phase name, a JSON-RPC
//! code, an HTTP status. The server's own error text can carry URLs and response bodies, so it is
//! read to classify the failure and never copied.

use usage_model::{ErrorCode, HarnessError};

/// JSON-RPC 2.0 "invalid request": what the app-server uses for auth problems too.
const INVALID_REQUEST: i64 = -32600;
const OVERLOADED: i64 = -32001;

/// Where in the conversation something went wrong.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Phase {
    Initialize,
    Account,
    RateLimits,
}

impl Phase {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Initialize => "initialize",
            Self::Account => "account",
            Self::RateLimits => "rate_limits",
        }
    }
}

fn error(code: ErrorCode, retryable: bool, phase: Phase) -> HarnessError {
    HarnessError::new(code, retryable).with_param("phase", phase.as_str())
}

pub(crate) fn timeout() -> HarnessError {
    HarnessError::new(ErrorCode::Timeout, true)
}

pub(crate) fn not_signed_in() -> HarnessError {
    HarnessError::new(ErrorCode::NotSignedIn, false)
}

pub(crate) fn endpoint_changed(phase: Phase) -> HarnessError {
    error(ErrorCode::EndpointChanged, false, phase)
}

/// The server answered `initialize` with an error or with something that is not a result object.
pub(crate) fn handshake_refused() -> HarnessError {
    error(ErrorCode::HandshakeFailed, false, Phase::Initialize)
}

/// A conversation that broke: the clock ran out, the pipe closed, or a line was not JSON-RPC.
pub(crate) fn from_transport(source: &HarnessError, phase: Phase) -> HarnessError {
    match source.code {
        ErrorCode::Timeout => error(ErrorCode::Timeout, true, phase),
        ErrorCode::ProtocolError => error(ErrorCode::ProtocolError, false, phase),
        _ => error(ErrorCode::HandshakeFailed, true, phase),
    }
}

/// Failures that need the user to act on the account never fall back to old rollout data, which
/// would hide the real problem behind plausible numbers.
pub(crate) fn allows_fallback(code: ErrorCode) -> bool {
    !matches!(code, ErrorCode::NotSignedIn | ErrorCode::Unauthorized)
}

/// What a JSON-RPC error from `account/rateLimits/read` means.
#[derive(Debug, PartialEq)]
pub(crate) enum Verdict {
    /// Not a failure: the login has no plan windows (API key).
    NoLimits,
    Failed(HarnessError),
}

pub(crate) fn from_rate_limits_error(code: i64, message: &str) -> Verdict {
    let text = message.to_ascii_lowercase();
    if code == INVALID_REQUEST && text.contains("authentication required") {
        // "chatgpt authentication required" is an API-key login; "codex account authentication
        // required" is no login at all.
        return if text.contains("chatgpt authentication") {
            Verdict::NoLimits
        } else {
            Verdict::Failed(not_signed_in())
        };
    }
    let status = http_status(&text);
    let (kind, retryable) = match status {
        Some(401) => (ErrorCode::NotSignedIn, false),
        Some(403) => (ErrorCode::Unauthorized, false),
        Some(429) => (ErrorCode::RateLimited, true),
        Some(500..=599) => (ErrorCode::ServerError, true),
        Some(_) => (ErrorCode::BadResponse, false),
        None => by_code_and_wording(code, &text),
    };
    let mut failure =
        error(kind, retryable, Phase::RateLimits).with_param("rpcCode", code.to_string());
    if let Some(status) = status {
        failure = failure.with_param("httpStatus", status.to_string());
    }
    Verdict::Failed(failure)
}

fn by_code_and_wording(code: i64, text: &str) -> (ErrorCode, bool) {
    let says = |words: &[&str]| words.iter().any(|word| text.contains(word));
    match code {
        OVERLOADED => (ErrorCode::RateLimited, true),
        // The server rejected the request itself: our client and its protocol disagree.
        -32602 | -32601 | INVALID_REQUEST => (ErrorCode::ProtocolError, false),
        _ if says(&["timed out", "timeout"]) => (ErrorCode::Timeout, true),
        _ if says(&["no snapshots returned"]) => (ErrorCode::BadResponse, false),
        _ if says(&["error sending request", "connect", "dns", "network", "unreachable"]) => {
            (ErrorCode::Network, true)
        }
        _ => (ErrorCode::Internal, true),
    }
}

/// The backend client words a bad status as `... failed: 401 Unauthorized; content-type=...`.
fn http_status(text: &str) -> Option<u16> {
    const MARK: &str = "failed: ";
    let rest = &text[text.find(MARK)? + MARK.len()..];
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    if digits.len() == 3 { digits.parse().ok() } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn failed(code: i64, message: &str) -> HarnessError {
        match from_rate_limits_error(code, message) {
            Verdict::Failed(error) => error,
            Verdict::NoLimits => panic!("{message}: expected a failure"),
        }
    }

    #[test]
    fn signed_out_and_api_key_are_told_apart() {
        let out = failed(-32600, "codex account authentication required to read rate limits");
        assert_eq!((out.code, out.retryable), (ErrorCode::NotSignedIn, false));
        let key =
            from_rate_limits_error(-32600, "chatgpt authentication required to read rate limits");
        assert_eq!(key, Verdict::NoLimits);
        // The same words under another code are not an auth verdict.
        assert_eq!(failed(-32603, "authentication required").code, ErrorCode::Internal);
    }

    #[test]
    fn backend_statuses_map_to_specific_codes_and_keep_only_the_number() {
        let wording = |status: &str| {
            format!(
                "failed to fetch codex rate limits: GET https://x.invalid/usage failed: {status}; content-type=text/html; body=<html>secret-ish</html>"
            )
        };
        for (status, code, retryable) in [
            ("401 Unauthorized", ErrorCode::NotSignedIn, false),
            ("403 Forbidden", ErrorCode::Unauthorized, false),
            ("429 Too Many Requests", ErrorCode::RateLimited, true),
            ("500 Internal Server Error", ErrorCode::ServerError, true),
            ("503 Service Unavailable", ErrorCode::ServerError, true),
            ("418 I'm a teapot", ErrorCode::BadResponse, false),
        ] {
            let error = failed(-32603, &wording(status));
            assert_eq!((error.code, error.retryable), (code, retryable), "{status}");
            assert_eq!(
                error.params.get("httpStatus").map(String::as_str),
                status.get(..3),
                "{status}"
            );
            assert!(error.params.values().all(|v| !v.contains("secret")), "{:?}", error.params);
        }
    }

    #[test]
    fn network_style_failures_are_retryable() {
        let sending = failed(
            -32603,
            "failed to fetch codex rate limits: error sending request for url (https://x.invalid/)",
        );
        assert_eq!((sending.code, sending.retryable), (ErrorCode::Network, true));
        let slow = failed(-32603, "failed to fetch codex rate limits: operation timed out");
        assert_eq!((slow.code, slow.retryable), (ErrorCode::Timeout, true));
        let overloaded = failed(-32001, "Server overloaded; retry later.");
        assert_eq!((overloaded.code, overloaded.retryable), (ErrorCode::RateLimited, true));
        let empty = failed(-32603, "failed to fetch codex rate limits: no snapshots returned");
        assert_eq!((empty.code, empty.retryable), (ErrorCode::BadResponse, false));
        let unknown = failed(-32603, "something nobody anticipated");
        assert_eq!((unknown.code, unknown.retryable), (ErrorCode::Internal, true));
    }

    #[test]
    fn a_server_that_rejects_the_request_is_a_protocol_problem() {
        for code in [-32600, -32601, -32602] {
            let error = failed(code, "Invalid request: unknown variant `account/rateLimits/read`");
            assert_eq!((error.code, error.retryable), (ErrorCode::ProtocolError, false), "{code}");
        }
    }

    #[test]
    fn params_never_hold_the_server_text() {
        let error = failed(
            -32603,
            "failed to fetch: GET https://example.invalid/?token=abc failed: 500 oops",
        );
        let keys: Vec<_> = error.params.keys().map(String::as_str).collect();
        assert_eq!(keys, ["httpStatus", "phase", "rpcCode"]);
    }

    #[test]
    fn status_extraction_is_strict() {
        assert_eq!(http_status("x failed: 404 not found"), Some(404));
        assert_eq!(http_status("failed: 4040"), None);
        assert_eq!(http_status("failed: 40"), None);
        assert_eq!(http_status("failed: x"), None);
        assert_eq!(http_status("no marker 404"), None);
        assert_eq!(http_status("failed: "), None);
        assert_eq!(http_status("ação failed: 502"), Some(502));
    }

    #[test]
    fn transport_failures_keep_their_nature() {
        let from = |code| from_transport(&HarnessError::new(code, true), Phase::Initialize);
        assert_eq!(
            (from(ErrorCode::Timeout).code, from(ErrorCode::Timeout).retryable),
            (ErrorCode::Timeout, true)
        );
        assert!(!from(ErrorCode::ProtocolError).retryable);
        for closed in [ErrorCode::NotRunning, ErrorCode::InvalidInput, ErrorCode::Internal] {
            let error = from(closed);
            assert_eq!((error.code, error.retryable), (ErrorCode::HandshakeFailed, true));
            assert_eq!(error.params.get("phase").map(String::as_str), Some("initialize"));
        }
    }

    #[test]
    fn only_account_problems_forbid_the_fallback() {
        for code in ErrorCode::ALL {
            let expected = !matches!(code, ErrorCode::NotSignedIn | ErrorCode::Unauthorized);
            assert_eq!(allows_fallback(code), expected, "{code:?}");
        }
    }
}
