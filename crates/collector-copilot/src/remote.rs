//! The quota request: one bounded GET with the GitHub token, parsed into windows.

use std::time::Duration;

use usage_model::{ErrorCode, HarnessError};
use usage_support::{Auth, ClientOptions, RequestSpec, SafeClient, Secret, TlsPolicy};

use crate::parse::{Parsed, parse_usage};

/// The token is only ever sent to this host: it is a constant, not a setting.
const BASE_URL: &str = "https://api.github.com";
const USAGE_PATH: [&str; 2] = ["copilot_internal", "user"];
/// The real body is a few KiB (it also lists organizations and endpoints); anything near this cap
/// is not the quota document.
const MAX_BODY_BYTES: usize = 128 * 1024;
const TIMEOUT: Duration = Duration::from_secs(15);

/// The header set the Copilot editor extensions send; the endpoint answers editors only.
pub(crate) const HEADERS: [(&str, &str); 4] = [
    ("Editor-Version", "vscode/1.96.2"),
    ("Editor-Plugin-Version", "copilot-chat/0.26.7"),
    ("User-Agent", "GitHubCopilotChat/0.26.7"),
    ("X-Github-Api-Version", "2025-04-01"),
];

/// Where the token goes. Release builds can only construct [`Endpoint::production`]; tests add a
/// loopback constructor so the token can be traced to the one URL it was given.
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
    token: &Secret,
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
        .fold(RequestSpec::new(endpoint.base_url.as_str()), |spec, part| spec.segment(*part));
    let spec = HEADERS.iter().fold(spec, |spec, (name, value)| spec.header(name, value));
    // `Authorization: token <t>`, the scheme word GitHub's REST API documents for OAuth tokens.
    let auth = Auth { scheme: Some("token".to_owned()), ..Auth::new(token) };
    let body = client.get_json(&spec.auth(auth)).await.map_err(describe)?;
    parse_usage(&body, now_secs)
}

/// A refused token is ambiguous (a fine-grained or app token, or an account without Copilot, is
/// refused like a wrong one), and a 404 on a fixed internal route means the route is gone.
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
    fn production_sends_the_token_only_to_the_https_host_with_no_loopback_escape() {
        let production = Endpoint::production();
        assert_eq!(production.base_url, "https://api.github.com");
        assert!(!production.allow_loopback_http);
        assert_eq!(USAGE_PATH.join("/"), "copilot_internal/user");
    }
}
