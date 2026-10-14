//! `SafeClient`: one hardened way to fetch small JSON documents.

use std::fmt;
use std::net::{Ipv6Addr, SocketAddr};
use std::time::Duration;

use reqwest::header::{ACCEPT, CONTENT_TYPE, HeaderValue};
use reqwest::{Certificate, Client, Method, redirect};
use serde_json::Value;
use usage_model::{ErrorCode, HarnessError};

use super::body::read_json;
use super::classify::{check_content_type, failure, from_status, from_transport};
use super::options::{ClientOptions, TlsPolicy};
use super::request::RequestSpec;
use super::url_policy::normalize_host;

/// No poll needs longer; the cap also keeps timer arithmetic far from overflow.
const MAX_TIMEOUT: Duration = Duration::from_secs(600);
/// Largest body ever accepted, whatever the options ask for: these are small JSON documents.
const MAX_BODY_CAP: usize = 16 << 20;
const MAX_PEM_BYTES: usize = 1 << 20;

/// HTTPS-only JSON client with a hard deadline, a response size cap, no redirects and no proxies.
///
/// Every request is checked against the URL policy first, so a refused request never reaches the
/// network. Errors carry only non-secret params (see `classify`).
pub struct SafeClient {
    pub(super) inner: Client,
    pub(super) timeout: Duration,
    max_body_bytes: usize,
    allow_loopback_http: bool,
    /// Normalized host that may be called without certificate verification, if any.
    insecure_host: Option<String>,
    pub(super) release_tls: bool,
}

impl fmt::Debug for SafeClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SafeClient")
            .field("timeout", &self.timeout)
            .field("max_body_bytes", &self.max_body_bytes)
            .field("allow_loopback_http", &self.allow_loopback_http)
            .field("insecure_host", &self.insecure_host)
            .finish_non_exhaustive()
    }
}

fn tls_error() -> HarnessError {
    HarnessError::new(ErrorCode::TlsError, false)
}

fn parse_pem_bundle(pem: &[u8]) -> Result<Vec<Certificate>, HarnessError> {
    if pem.len() > MAX_PEM_BYTES {
        return Err(tls_error());
    }
    match Certificate::from_pem_bundle(pem) {
        Ok(certs) if !certs.is_empty() => Ok(certs),
        _ => Err(tls_error()),
    }
}

impl SafeClient {
    /// Builds the client. An unusable CA bundle or an empty insecure host is a `TlsError`; a
    /// user agent that cannot be a header value is `InvalidInput`.
    pub fn new(options: ClientOptions) -> Result<Self, HarnessError> {
        let user_agent = HeaderValue::from_str(&options.user_agent)
            .map_err(|_| HarnessError::new(ErrorCode::InvalidInput, false))?;
        let timeout = options.timeout.min(MAX_TIMEOUT);
        let loopback =
            [SocketAddr::from(([127, 0, 0, 1], 0)), SocketAddr::from((Ipv6Addr::LOCALHOST, 0))];
        let mut builder = Client::builder()
            .redirect(redirect::Policy::none())
            .timeout(timeout)
            .user_agent(user_agent)
            // A proxy variable must never capture credentials or loopback traffic.
            .no_proxy()
            // A poll every minute gains nothing from keep-alive and risks stale sockets.
            .pool_max_idle_per_host(0)
            // `localhost` must mean loopback even if the hosts file or DNS says otherwise.
            .resolve_to_addrs("localhost", &loopback);

        let mut insecure_host = None;
        builder = match &options.tls {
            TlsPolicy::System => builder,
            TlsPolicy::CustomCa(pem) => builder.tls_certs_only(parse_pem_bundle(pem)?),
            TlsPolicy::Insecure { host } => {
                let host = normalize_host(host);
                if host.is_empty() {
                    return Err(tls_error());
                }
                insecure_host = Some(host);
                builder.tls_danger_accept_invalid_certs(true)
            }
        };
        let inner = builder.build().map_err(|_| match options.tls {
            TlsPolicy::System => HarnessError::new(ErrorCode::Internal, false),
            _ => tls_error(),
        })?;
        Ok(Self {
            inner,
            timeout,
            max_body_bytes: options.max_body_bytes.min(MAX_BODY_CAP),
            allow_loopback_http: options.allow_loopback_http,
            insecure_host,
            release_tls: matches!(options.tls, TlsPolicy::System),
        })
    }

    pub async fn get_json(&self, request: &RequestSpec) -> Result<Value, HarnessError> {
        self.execute(Method::GET, request, None).await
    }

    pub async fn post_json(
        &self,
        request: &RequestSpec,
        body: &Value,
    ) -> Result<Value, HarnessError> {
        let bytes = serde_json::to_vec(body)
            .map_err(|_| HarnessError::new(ErrorCode::InvalidInput, false))?;
        self.execute(Method::POST, request, Some(bytes)).await
    }

    async fn execute(
        &self,
        method: Method,
        request: &RequestSpec,
        body: Option<Vec<u8>>,
    ) -> Result<Value, HarnessError> {
        // Everything that can be refused is refused here, before the first byte is sent.
        let prepared = request.prepare(self.allow_loopback_http)?;
        let host = prepared.url.host_str().unwrap_or_default().to_owned();
        if let Some(allowed) = &self.insecure_host
            && normalize_host(&host) != *allowed
        {
            return Err(HarnessError::new(ErrorCode::InvalidUrl, false).with_param("host", host));
        }

        let mut builder = self.inner.request(method, prepared.url).headers(prepared.headers);
        builder = builder.header(ACCEPT, HeaderValue::from_static("application/json"));
        if let Some(body) = body {
            builder = builder.header(CONTENT_TYPE, "application/json").body(body);
        }
        let call = async {
            let mut response = builder.send().await.map_err(|e| from_transport(&e, &host))?;
            if let Some(error) = from_status(response.status(), response.headers(), &host) {
                return Err(error);
            }
            check_content_type(response.headers(), &host)?;
            read_json(&mut response, self.max_body_bytes, &host).await
        };
        match tokio::time::timeout(self.timeout, call).await {
            Ok(result) => result,
            Err(_elapsed) => Err(failure(ErrorCode::Timeout, true, &host)),
        }
    }
}
