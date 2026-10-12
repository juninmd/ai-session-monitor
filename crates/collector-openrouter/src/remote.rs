//! The requests: a bounded GET of the key document, then a best-effort GET of the credits one.

use std::time::Duration;

use serde_json::Value;
use usage_model::{ErrorCode, HarnessError};
use usage_support::{Auth, ClientOptions, RequestSpec, SafeClient, Secret, TlsPolicy};

use crate::parse::{Credits, KeyInfo, parse_credits, parse_key};

/// The key is only ever sent to this host: it is a constant, not a setting.
const BASE_URL: &str = "https://openrouter.ai";
const KEY_PATH: [&str; 3] = ["api", "v1", "key"];
const CREDITS_PATH: [&str; 3] = ["api", "v1", "credits"];
/// The real bodies are a few hundred bytes; anything near this cap is not the document.
const MAX_BODY_BYTES: usize = 64 * 1024;
const TIMEOUT: Duration = Duration::from_secs(15);

/// Where the key goes. Release builds can only construct [`Endpoint::production`]; tests add a
/// loopback constructor so the key can be traced to the URLs it was given.
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

#[derive(Debug)]
pub(crate) struct Fetched {
    pub key: KeyInfo,
    /// `None` when the credits call was refused (an ordinary key) or unreadable.
    pub credits: Option<Credits>,
}

pub(crate) async fn fetch(endpoint: &Endpoint, key: &Secret) -> Result<Fetched, HarnessError> {
    let client = SafeClient::new(ClientOptions {
        timeout: endpoint.timeout,
        max_body_bytes: MAX_BODY_BYTES,
        tls: TlsPolicy::System,
        allow_loopback_http: endpoint.allow_loopback_http,
        ..ClientOptions::default()
    })?;
    let info = parse_key(&get(&client, endpoint, &KEY_PATH, key).await.map_err(describe)?)?;
    // The balance is a bonus: its failure (403 for an ordinary key, 429, a timeout) is not the poll's.
    let credits =
        get(&client, endpoint, &CREDITS_PATH, key).await.ok().and_then(|b| parse_credits(&b));
    Ok(Fetched { key: info, credits })
}

async fn get(
    client: &SafeClient,
    endpoint: &Endpoint,
    path: &[&str],
    key: &Secret,
) -> Result<Value, HarnessError> {
    let spec = path
        .iter()
        .fold(RequestSpec::new(endpoint.base_url.as_str()), |spec, segment| spec.segment(*segment))
        .auth(Auth::new(key));
    client.get_json(&spec).await
}

/// A 404 on a fixed route means the route is gone, not that the user mistyped. A 402 has no code
/// of its own and stays a bad response whose `httpStatus` param names it.
fn describe(error: HarnessError) -> HarnessError {
    match error.code {
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
        assert_eq!(production.base_url, "https://openrouter.ai");
        assert!(!production.allow_loopback_http);
        assert_eq!(KEY_PATH.join("/"), "api/v1/key");
        assert_eq!(CREDITS_PATH.join("/"), "api/v1/credits");
    }
}
