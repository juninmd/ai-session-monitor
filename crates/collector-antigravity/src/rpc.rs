//! The two Connect (JSON over HTTP) calls the collector makes to the language server.
//!
//! The server is on this machine and presents a self-signed certificate, so certificate checks are
//! switched off for `127.0.0.1` and for nothing else ([`TlsPolicy::Insecure`] refuses any other
//! host before it touches the network). The CSRF token travels as a sensitive header.

use std::time::Duration;

use serde_json::{Value, json};
use usage_model::HarnessError;
use usage_support::{ClientOptions, RequestSpec, SafeClient, TlsPolicy};

use crate::discovery::Target;

const SERVICE: &str = "exa.language_server_pb.LanguageServerService";
const CSRF_HEADER: &str = "X-Codeium-Csrf-Token";
const LOOPBACK: &str = "127.0.0.1";

/// The RPCs we call. Both are unary POSTs with a JSON body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Method {
    /// The plan and, on older builds, a quota per model.
    UserStatus,
    /// Quota buckets with their 5 hour and weekly windows. Newer builds only; the IDE answers 404.
    QuotaSummary,
}

impl Method {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::UserStatus => "GetUserStatus",
            Self::QuotaSummary => "RetrieveUserQuotaSummary",
        }
    }

    /// The bodies community clients send; `locale` keeps the prose descriptions in English.
    fn body(self) -> Value {
        match self {
            Self::UserStatus => json!({ "metadata": {
                "ideName": "antigravity",
                "extensionName": "antigravity",
                "ideVersion": "unknown",
                "locale": "en",
            } }),
            Self::QuotaSummary => json!({ "forceRefresh": true }),
        }
    }
}

/// Limits of one client; tests shrink them.
#[derive(Clone, Debug)]
pub(crate) struct Limits {
    pub request_timeout: Duration,
    pub max_body_bytes: usize,
}

impl Default for Limits {
    fn default() -> Self {
        // A model list with its metadata is tens of KiB; this leaves room, not a free-for-all.
        Self { request_timeout: Duration::from_secs(4), max_body_bytes: 4 << 20 }
    }
}

pub(crate) struct Rpc {
    client: SafeClient,
}

impl Rpc {
    pub(crate) fn new(limits: &Limits) -> Result<Self, HarnessError> {
        let client = SafeClient::new(ClientOptions {
            timeout: limits.request_timeout,
            max_body_bytes: limits.max_body_bytes,
            tls: TlsPolicy::Insecure { host: LOOPBACK.to_owned() },
            // Plain http is only ever used for the extension server fallback, and only to loopback.
            allow_loopback_http: true,
            ..ClientOptions::default()
        })?;
        Ok(Self { client })
    }

    pub(crate) async fn call(
        &self,
        target: &Target,
        method: Method,
    ) -> Result<Value, HarnessError> {
        let request =
            RequestSpec::new(format!("{}://{LOOPBACK}:{}", target.scheme.as_str(), target.port))
                .segment(SERVICE)
                .segment(method.name())
                .header("Connect-Protocol-Version", "1")
                .secret_header(CSRF_HEADER, &target.token);
        self.client.post_json(&request, &method.body()).await
    }
}
