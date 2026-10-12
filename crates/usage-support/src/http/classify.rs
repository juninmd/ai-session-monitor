//! Turns statuses and transport failures into `HarnessError`s. Params are limited to `host`,
//! `httpStatus`, `retryAfterSecs` and `redirect`: never a URL path or query, header or body text.

use std::error::Error as StdError;

use reqwest::StatusCode;
use reqwest::header::{CONTENT_TYPE, HeaderMap, RETRY_AFTER};
use usage_model::{ErrorCode, HarnessError};

const MAX_RETRY_AFTER_SECS: u64 = 3600;

pub(super) fn failure(code: ErrorCode, retryable: bool, host: &str) -> HarnessError {
    HarnessError::new(code, retryable).with_param("host", host)
}

/// `None` for a 2xx; otherwise the error. Redirects are errors because they are never followed.
pub(super) fn from_status(
    status: StatusCode,
    headers: &HeaderMap,
    host: &str,
) -> Option<HarnessError> {
    let code = status.as_u16();
    let error = match code {
        200..=299 => return None,
        300..=399 => failure(ErrorCode::BadResponse, false, host).with_param("redirect", "true"),
        401 | 403 => failure(ErrorCode::Unauthorized, false, host),
        404 => failure(ErrorCode::NotFound, false, host),
        429 => {
            let error = failure(ErrorCode::RateLimited, true, host);
            match retry_after_secs(headers) {
                Some(secs) => error.with_param("retryAfterSecs", secs.to_string()),
                None => error,
            }
        }
        500..=599 => failure(ErrorCode::ServerError, true, host),
        _ => failure(ErrorCode::BadResponse, false, host),
    };
    Some(error.with_param("httpStatus", code.to_string()))
}

/// Integer seconds only (the HTTP-date form is ignored), capped so a hostile server cannot park
/// the poller for days.
fn retry_after_secs(headers: &HeaderMap) -> Option<u64> {
    let text = headers.get(RETRY_AFTER)?.to_str().ok()?.trim();
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    // All digits: a number too big for u64 still means "a long time".
    Some(text.parse::<u64>().unwrap_or(u64::MAX).min(MAX_RETRY_AFTER_SECS))
}

pub(super) fn from_transport(error: &reqwest::Error, host: &str) -> HarnessError {
    if error.is_timeout() {
        failure(ErrorCode::Timeout, true, host)
    } else if has_tls_cause(error) {
        failure(ErrorCode::TlsError, false, host)
    } else if error.is_builder() {
        failure(ErrorCode::Internal, false, host)
    } else {
        failure(ErrorCode::Network, true, host)
    }
}

/// True when a rustls error sits anywhere in the chain. `io::Error` hides the error it wraps from
/// `source()`, which is how a failed handshake reaches us, so it is opened explicitly.
fn has_tls_cause(error: &(dyn StdError + 'static)) -> bool {
    let mut current = Some(error);
    while let Some(err) = current {
        if err.is::<rustls::Error>() {
            return true;
        }
        if let Some(io) = err.downcast_ref::<std::io::Error>()
            && io.get_ref().is_some_and(|inner| has_tls_cause(inner))
        {
            return true;
        }
        current = err.source();
    }
    false
}

/// A missing content type is tolerated; HTML and other non-JSON types are not.
pub(super) fn check_content_type(headers: &HeaderMap, host: &str) -> Result<(), HarnessError> {
    match headers.get(CONTENT_TYPE) {
        None => Ok(()),
        Some(value) if value.to_str().is_ok_and(is_json_compatible) => Ok(()),
        Some(_) => Err(failure(ErrorCode::BadResponse, false, host)),
    }
}

/// JSON and the types servers commonly use for JSON they did not label properly.
fn is_json_compatible(content_type: &str) -> bool {
    let essence = content_type.split(';').next().unwrap_or_default().trim().to_ascii_lowercase();
    essence.is_empty()
        || essence.ends_with("+json")
        || matches!(
            essence.as_str(),
            "application/json" | "text/json" | "text/plain" | "application/octet-stream"
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_compatible_types() {
        for ok in [
            "application/json",
            "Application/JSON; charset=utf-8",
            "application/problem+json",
            "application/vnd.api+json;version=2",
            "text/json",
            "text/plain; charset=utf-8",
            "application/octet-stream",
            "",
            "  ",
        ] {
            assert!(is_json_compatible(ok), "{ok:?}");
        }
        for bad in [
            "text/html",
            "text/html; charset=utf-8",
            "application/xhtml+xml",
            "application/xml",
            "text/xml",
            "image/png",
            "text/css",
            "application/javascript",
            "multipart/form-data; boundary=x",
            "json",
            "application/jsonx",
        ] {
            assert!(!is_json_compatible(bad), "{bad:?}");
        }
    }

    #[test]
    fn retry_after_accepts_only_integer_seconds_and_caps_them() {
        let with = |value: &str| {
            let mut headers = HeaderMap::new();
            headers.insert(RETRY_AFTER, value.parse().unwrap());
            retry_after_secs(&headers)
        };
        assert_eq!(with("120"), Some(120));
        assert_eq!(with(" 7 "), Some(7));
        assert_eq!(with("0"), Some(0));
        assert_eq!(with("3600"), Some(3600));
        assert_eq!(with("3601"), Some(3600));
        assert_eq!(with("99999999999999999999999999"), Some(3600));
        for bad in ["", "-5", "+5", "1.5", "abc", "Wed, 21 Oct 2026 07:28:00 GMT", "12 s"] {
            assert_eq!(with(bad), None, "{bad:?}");
        }
        assert_eq!(retry_after_secs(&HeaderMap::new()), None);
    }
}
