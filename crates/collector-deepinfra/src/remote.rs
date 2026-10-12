//! The requests: a bounded GET of the checklist, then a best-effort GET of the month's usage.

use std::time::Duration;

use serde_json::Value;
use usage_model::{ErrorCode, HarnessError};
use usage_support::{Auth, ClientOptions, RequestSpec, SafeClient, Secret, TlsPolicy};

use crate::parse::{Checklist, parse_checklist, parse_month_spend};

/// The key is only ever sent to this host: it is a constant, not a setting.
const BASE_URL: &str = "https://api.deepinfra.com";
const CHECKLIST_PATH: [&str; 2] = ["payment", "checklist"];
const USAGE_PATH: [&str; 2] = ["payment", "usage"];
/// The real bodies are a few KiB; anything near this cap is not the document.
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
    pub checklist: Checklist,
    /// `None` when the usage call failed or was not the document we know.
    pub month_spend: Option<f64>,
}

pub(crate) async fn fetch(endpoint: &Endpoint, key: &Secret) -> Result<Fetched, HarnessError> {
    let client = SafeClient::new(ClientOptions {
        timeout: endpoint.timeout,
        max_body_bytes: MAX_BODY_BYTES,
        tls: TlsPolicy::System,
        allow_loopback_http: endpoint.allow_loopback_http,
        ..ClientOptions::default()
    })?;
    let checklist =
        parse_checklist(&get(&client, endpoint, &CHECKLIST_PATH, key).await.map_err(describe)?)?;
    // The month spend is a bonus: its failure is not the poll's.
    let month_spend =
        get(&client, endpoint, &USAGE_PATH, key).await.ok().and_then(|b| parse_month_spend(&b));
    Ok(Fetched { checklist, month_spend })
}

async fn get(
    client: &SafeClient,
    endpoint: &Endpoint,
    path: &[&str],
    key: &Secret,
) -> Result<Value, HarnessError> {
    let mut spec = path
        .iter()
        .fold(RequestSpec::new(endpoint.base_url.as_str()), |spec, segment| spec.segment(*segment))
        .auth(Auth::new(key));
    if path == USAGE_PATH {
        spec = spec.query("from", "current");
    }
    client.get_json(&spec).await
}

/// A 404 on a fixed route means the route is gone, not that the user mistyped.
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
        assert_eq!(production.base_url, "https://api.deepinfra.com");
        assert!(!production.allow_loopback_http);
        assert_eq!(CHECKLIST_PATH.join("/"), "payment/checklist");
        assert_eq!(USAGE_PATH.join("/"), "payment/usage");
    }
}
