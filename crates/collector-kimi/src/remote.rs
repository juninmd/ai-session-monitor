//! The usages request: one bounded GET with the user's key, parsed into windows.

use std::time::Duration;

use usage_model::{ErrorCode, HarnessError};
use usage_support::{Auth, ClientOptions, RequestSpec, SafeClient, Secret, TlsPolicy};

use crate::parse::{Parsed, parse_usage};

/// The key is only ever sent to this host: it is a constant, not a setting.
const BASE_URL: &str = "https://api.kimi.com";
const USAGE_PATH: [&str; 3] = ["coding", "v1", "usages"];
/// The real body is a few hundred bytes; anything near this cap is not the usage document.
const MAX_BODY_BYTES: usize = 64 * 1024;
const TIMEOUT: Duration = Duration::from_secs(15);

/// Where the key goes. Release builds can only construct [`Endpoint::production`]; tests add a
/// loopback constructor so the key can be traced to the one URL it was given.
pub(crate) struct Endpoint {
    base_url: String,
    timeout: Duration,
    allow_loopback_http: bool,
}

impl Endpoint {
    pub(crate) fn production() -> Self {
        Self { base_url: BASE_URL.to_owned(), timeout: TIMEOUT, allow_loopback_http: false }
    }

    #[cfg(test)]
    pub(crate) fn loopback(base_url: String, timeout: Duration) -> Self {
        Self { base_url, timeout, allow_loopback_http: true }
    }

    /// Production rules (no plain http) aimed at a test URL.
    #[cfg(test)]
    pub(crate) fn strict(base_url: String) -> Self {
        Self { base_url, timeout: Duration::from_secs(2), allow_loopback_http: false }
    }
}

pub(crate) async fn fetch(
    endpoint: &Endpoint,
    key: &Secret,
    now_secs: i64,
) -> Result<Parsed, HarnessError> {
    let client = SafeClient::new(ClientOptions {
        timeout: endpoint.timeout,
        max_body_bytes: MAX_BODY_BYTES,
        tls: TlsPolicy::System,
        allow_loopback_http: endpoint.allow_loopback_http,
        ..ClientOptions::default()
    })?;
    let spec = USAGE_PATH
        .iter()
        .fold(RequestSpec::new(endpoint.base_url.as_str()), |spec, segment| spec.segment(*segment))
        .auth(Auth::new(key));
    let body = client.get_json(&spec).await.map_err(describe)?;
    parse_usage(&body, now_secs)
}

/// A refused key is ambiguous (a Moonshot Open Platform key is refused like a wrong one), and a 404
/// on a fixed route means the route is gone, not that the user mistyped something.
fn describe(error: HarnessError) -> HarnessError {
    match error.code {
        ErrorCode::Unauthorized => error.with_param("ambiguous", "true"),
        ErrorCode::NotFound => HarnessError { code: ErrorCode::EndpointChanged, ..error },
        _ => error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_sends_the_key_only_to_the_https_host_with_no_loopback_escape() {
        let production = Endpoint::production();
        assert_eq!(production.base_url, "https://api.kimi.com");
        assert!(!production.allow_loopback_http);
        assert_eq!(USAGE_PATH.join("/"), "coding/v1/usages");
    }
}
