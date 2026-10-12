//! The Go usage endpoint: one bounded GET with the user's key, parsed into windows.

pub(crate) mod month;
pub(crate) mod parse;

use std::time::Duration;

use usage_model::{ErrorCode, HarnessError};
use usage_support::{Auth, ClientOptions, RequestSpec, SafeClient, Secret, TlsPolicy};

pub(crate) use parse::Parsed;

/// The key is only ever sent to this host: it is a constant, not a setting.
const GO_BASE_URL: &str = "https://opencode.ai";
const USAGE_PATH: [&str; 4] = ["zen", "go", "v1", "usage"];
/// The real body is a few hundred bytes; anything near this cap is not the usage document.
const MAX_BODY_BYTES: usize = 64 * 1024;
/// Leaves room for the local read inside the scheduler's 30 s hard timeout.
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
        Self { base_url: GO_BASE_URL.to_owned(), timeout: TIMEOUT, allow_loopback_http: false }
    }

    #[cfg(test)]
    pub(crate) fn loopback(base_url: String, timeout: Duration) -> Self {
        Self { base_url, timeout, allow_loopback_http: true }
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
    parse::parse_usage(&body, now_secs)
}

/// 401/403 are ambiguous (valid keys have been rejected by the service), and a 404 on a fixed
/// route means the route is gone, not that the user mistyped something.
fn describe(error: HarnessError) -> HarnessError {
    match error.code {
        ErrorCode::Unauthorized => error.with_param("ambiguous", "true"),
        ErrorCode::NotFound => HarnessError { code: ErrorCode::EndpointChanged, ..error },
        _ => error,
    }
}

#[cfg(test)]
mod tests {
    use std::io::ErrorKind;
    use std::net::TcpListener;

    use super::*;

    #[test]
    fn production_sends_the_key_only_to_the_https_host_with_no_loopback_escape() {
        let production = Endpoint::production();
        assert_eq!(production.base_url, "https://opencode.ai");
        assert!(!production.allow_loopback_http);
        assert_eq!(USAGE_PATH.join("/"), "zen/go/v1/usage");
    }

    #[tokio::test]
    async fn plain_http_is_refused_before_any_connection_without_the_test_escape() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let base_url = format!("http://{}", listener.local_addr().unwrap());
        let endpoint =
            Endpoint { base_url, timeout: Duration::from_secs(2), allow_loopback_http: false };
        let error = fetch(&endpoint, &Secret::new("fab-key"), 0).await.unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidUrl);
        assert_eq!(listener.accept().unwrap_err().kind(), ErrorKind::WouldBlock);
    }
}
