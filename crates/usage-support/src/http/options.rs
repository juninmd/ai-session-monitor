//! Client configuration: limits, TLS policy and the authentication header.

use std::fmt;
use std::time::Duration;

use crate::Secret;

/// Hard limits and trust settings for one [`SafeClient`](super::SafeClient).
#[derive(Clone, Debug)]
pub struct ClientOptions {
    /// Total deadline for a request, from connecting to the last body byte (at most ten minutes).
    pub timeout: Duration,
    /// Largest response body accepted (at most 16 MiB); more is `ResponseTooLarge`.
    pub max_body_bytes: usize,
    pub tls: TlsPolicy,
    pub user_agent: String,
    /// Allow plain `http` to loopback hosts (`127.0.0.0/8`, `::1`, `localhost`) and nothing else.
    pub allow_loopback_http: bool,
}

impl Default for ClientOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(10),
            max_body_bytes: 1024 * 1024,
            tls: TlsPolicy::System,
            user_agent: concat!("ai-session-monitor/", env!("CARGO_PKG_VERSION")).to_owned(),
            allow_loopback_http: false,
        }
    }
}

/// Who the client trusts to vouch for a server.
#[derive(Clone, PartialEq, Eq)]
pub enum TlsPolicy {
    /// The operating system trust store (on Windows, the certificate store).
    System,
    /// Exactly the CA certificates in this PEM bundle, and no others.
    CustomCa(Vec<u8>),
    /// No certificate or host name verification, for requests to `host` only. Any request to
    /// another host is refused before it touches the network. The traffic is encrypted but the
    /// server is not authenticated, so use it only for a service on the same machine or network.
    Insecure { host: String },
}

impl fmt::Debug for TlsPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::System => f.write_str("System"),
            Self::CustomCa(pem) => write!(f, "CustomCa({} bytes)", pem.len()),
            Self::Insecure { host } => f.debug_struct("Insecure").field("host", host).finish(),
        }
    }
}

/// Where and how a credential travels. Build with [`Auth::new`] for `Authorization: Bearer ...`
/// and override fields for other shapes: `Auth { header: "X-Api-Key".into(), scheme: None, ..Auth::new(&key) }`.
#[derive(Debug)]
pub struct Auth<'a> {
    pub header: String,
    /// Word placed before the secret (`Bearer`, `Token`); `None` sends the secret alone.
    pub scheme: Option<String>,
    pub secret: &'a Secret,
}

impl<'a> Auth<'a> {
    pub fn new(secret: &'a Secret) -> Self {
        Self { header: "Authorization".to_owned(), scheme: Some("Bearer".to_owned()), secret }
    }
}
