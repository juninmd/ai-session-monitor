//! `RequestSpec`: a URL plus the headers, path segments and query pairs of one call, validated
//! when built so that [`SafeClient`](super::SafeClient) can fail before any network I/O.

use std::fmt;

use reqwest::Url;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use usage_model::{ErrorCode, HarnessError};

use super::options::Auth;
use super::url_policy::{add_path, add_query, parse_url};
use crate::{Secret, scrub};

/// Headers the client owns: framing, connection management and the JSON content type.
const MANAGED_HEADERS: [&str; 11] = [
    "host",
    "content-length",
    "content-type",
    "transfer-encoding",
    "connection",
    "keep-alive",
    "upgrade",
    "te",
    "trailer",
    "proxy-authorization",
    "proxy-connection",
];

/// One request: where it goes and what it carries. Builder methods never fail; the first problem
/// (a bad header name or value, an empty secret) is kept and reported by the client as
/// `InvalidInput` before anything is sent.
#[derive(Clone)]
pub struct RequestSpec {
    url: String,
    segments: Vec<String>,
    query: Vec<(String, String)>,
    headers: HeaderMap,
    error: Option<HarnessError>,
}

/// A request whose URL passed the policy, ready to hand to the HTTP client.
pub(super) struct Prepared {
    pub url: Url,
    pub headers: HeaderMap,
}

impl RequestSpec {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            segments: Vec::new(),
            query: Vec::new(),
            headers: HeaderMap::new(),
            error: None,
        }
    }

    /// Appends one path segment after the URL's own path, percent-encoded.
    #[must_use]
    pub fn segment(mut self, segment: impl Into<String>) -> Self {
        self.segments.push(segment.into());
        self
    }

    /// Appends a query pair; key and value are percent-encoded.
    #[must_use]
    pub fn query(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query.push((key.into(), value.into()));
        self
    }

    /// A header that is not secret and may show up in diagnostics.
    #[must_use]
    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.insert(name, value, false);
        self
    }

    /// A header carrying a credential (CSRF token, API key): marked sensitive, never logged.
    #[must_use]
    pub fn secret_header(mut self, name: &str, value: &Secret) -> Self {
        self.insert(name, value.expose(), true);
        self
    }

    /// `Authorization: Bearer <secret>`, or whatever shape `auth` describes.
    #[must_use]
    pub fn auth(mut self, auth: Auth<'_>) -> Self {
        let secret = auth.secret.expose();
        if secret.is_empty() {
            self.fail(ErrorCode::InvalidInput);
            return self;
        }
        let value = match &auth.scheme {
            Some(scheme) => format!("{scheme} {secret}"),
            None => secret.to_owned(),
        };
        self.insert(&auth.header, &value, true);
        self
    }

    fn fail(&mut self, code: ErrorCode) {
        self.error.get_or_insert_with(|| HarnessError::new(code, false));
    }

    fn insert(&mut self, name: &str, value: &str, sensitive: bool) {
        let name = HeaderName::from_bytes(name.as_bytes());
        let value = HeaderValue::from_str(value);
        match (name, value) {
            (Ok(name), Ok(mut value)) if !MANAGED_HEADERS.contains(&name.as_str()) => {
                value.set_sensitive(sensitive);
                self.headers.insert(name, value);
            }
            _ => self.fail(ErrorCode::InvalidInput),
        }
    }

    /// Applies the URL policy and assembles the final URL. No network I/O.
    pub(super) fn prepare(&self, allow_loopback_http: bool) -> Result<Prepared, HarnessError> {
        if let Some(error) = &self.error {
            return Err(error.clone());
        }
        let mut url = parse_url(&self.url, allow_loopback_http)?;
        add_path(&mut url, self.segments.iter().map(String::as_str))?;
        add_query(&mut url, self.query.iter().map(|(k, v)| (k.as_str(), v.as_str())));
        Ok(Prepared { url, headers: self.headers.clone() })
    }
}

impl fmt::Debug for RequestSpec {
    /// Shows the scrubbed URL, query keys and header names; never a value.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let keys: Vec<&str> = self.query.iter().map(|(k, _)| k.as_str()).collect();
        let names: Vec<&str> = self.headers.keys().map(HeaderName::as_str).collect();
        f.debug_struct("RequestSpec")
            .field("url", &scrub(&self.url))
            .field("segments", &self.segments.len())
            .field("query_keys", &keys)
            .field("headers", &names)
            .field("error", &self.error.as_ref().map(|e| e.code))
            .finish()
    }
}
