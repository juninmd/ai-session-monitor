//! Everything decided before the first byte of I/O: the validated inputs of one poll.

use std::path::PathBuf;
use std::time::Duration;

use usage_model::settings::{LitellmScope, LitellmSettings, TlsMode};
use usage_model::{ErrorCode, HarnessError};
use usage_support::{Secret, append_path, check_url};

use crate::record::Kind;

const MIN_TIMEOUT_SECS: u32 = 1;
const MAX_TIMEOUT_SECS: u32 = 30;
const MAX_ID_CHARS: usize = 256;

/// How the server certificate is checked, already validated but not yet read from disk.
pub(crate) enum Tls {
    System,
    CustomCa(PathBuf),
    Insecure(String),
}

pub(crate) struct Plan<'a> {
    pub secret: &'a Secret,
    pub base: String,
    /// Host only, lowercased: what the view shows and what `Insecure` mode is pinned to.
    pub host: String,
    pub scope: Kind,
    /// Trimmed user or team id; `None` for the key scope.
    pub entity: Option<String>,
    pub header: String,
    /// `Bearer` for `Authorization`; a custom key header carries the raw key.
    pub scheme: Option<String>,
    pub tls: Tls,
    pub timeout: Duration,
    pub allow_loopback_http: bool,
}

/// `InvalidInput` naming the settings field the UI should highlight.
fn invalid(field: &str) -> HarnessError {
    HarnessError::new(ErrorCode::InvalidInput, false).with_param("field", field)
}

impl<'a> Plan<'a> {
    pub(crate) fn new(
        settings: &LitellmSettings,
        base: &str,
        secret: &'a Secret,
    ) -> Result<Self, HarnessError> {
        check_url(base, true)?;
        let host = host_of(&append_path(base, &[])?)
            .ok_or_else(|| HarnessError::new(ErrorCode::InvalidUrl, false))?;
        if base.contains('?') {
            return Err(HarnessError::new(ErrorCode::InvalidUrl, false).with_param("host", host));
        }
        let (scope, entity) = scope_of(settings)?;
        let header = header_name(&settings.auth_header)?;
        let visible_ascii = secret.expose().bytes().all(|b| (0x20..=0x7e).contains(&b));
        if !visible_ascii {
            return Err(invalid("value"));
        }
        let tls = tls_of(&settings.tls, &host)?;
        Ok(Self {
            secret,
            base: base.to_owned(),
            scheme: header.eq_ignore_ascii_case("authorization").then(|| "Bearer".to_owned()),
            header,
            tls,
            timeout: Duration::from_secs(u64::from(
                settings.timeout_secs.clamp(MIN_TIMEOUT_SECS, MAX_TIMEOUT_SECS),
            )),
            // Valid only because of the loopback allowance, so plain http was the point.
            allow_loopback_http: check_url(base, false).is_err(),
            host,
            scope,
            entity,
        })
    }
}

/// An id that is safe to put in a query: bounded and free of control characters.
pub(crate) fn valid_id(id: &str) -> bool {
    !id.is_empty() && id.chars().count() <= MAX_ID_CHARS && !id.chars().any(char::is_control)
}

fn scope_of(settings: &LitellmSettings) -> Result<(Kind, Option<String>), HarnessError> {
    let kind = match settings.scope {
        LitellmScope::Key => return Ok((Kind::Key, None)),
        LitellmScope::User => Kind::User,
        LitellmScope::Team => Kind::Team,
    };
    match settings.entity_id.as_deref().map(str::trim) {
        Some(id) if valid_id(id) => Ok((kind, Some(id.to_owned()))),
        _ => Err(invalid("entityId")),
    }
}

fn header_name(raw: &str) -> Result<String, HarnessError> {
    let name = raw.trim();
    let token = !name.is_empty() && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-');
    if token { Ok(name.to_owned()) } else { Err(invalid("authHeader")) }
}

fn tls_of(mode: &TlsMode, host: &str) -> Result<Tls, HarnessError> {
    match mode {
        TlsMode::System => Ok(Tls::System),
        TlsMode::CustomCa { pem_path } => {
            let path = PathBuf::from(pem_path.trim());
            // A relative path would be read from whatever directory the app was launched in.
            if path.is_absolute() { Ok(Tls::CustomCa(path)) } else { Err(invalid("pemPath")) }
        }
        TlsMode::Insecure { host: confirmed } => {
            let confirmed = confirmed.trim();
            if !confirmed.is_empty() && same_host(confirmed, host) {
                Ok(Tls::Insecure(confirmed.to_owned()))
            } else {
                Err(invalid("host"))
            }
        }
    }
}

/// Same comparison as the HTTP client's insecure-host pin: case, brackets and one trailing dot.
fn same_host(a: &str, b: &str) -> bool {
    fn norm(host: &str) -> String {
        let host = host.trim().trim_start_matches('[').trim_end_matches(']');
        host.strip_suffix('.').unwrap_or(host).to_ascii_lowercase()
    }
    norm(a) == norm(b)
}

/// Host of a URL that already passed the policy: no scheme, userinfo, port, path or query.
fn host_of(url: &str) -> Option<String> {
    let authority = url.split_once("://")?.1.split(['/', '?', '#']).next()?;
    let host_port = authority.rsplit('@').next()?;
    let host = match host_port.strip_prefix('[') {
        Some(inner) => format!("[{}]", inner.split_once(']')?.0),
        None => host_port.split(':').next()?.to_owned(),
    };
    (!host.is_empty() && host != "[]").then(|| host.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_is_extracted_without_scheme_port_path_or_userinfo() {
        for (url, host) in [
            ("https://gateway.example.dev/litellm", "gateway.example.dev"),
            ("https://Gateway.Example.DEV:8443/a/b?x=1#f", "gateway.example.dev"),
            ("http://127.0.0.1:4000/", "127.0.0.1"),
            ("http://[::1]:4000/litellm", "[::1]"),
            ("https://user:pw@host.example.dev/", "host.example.dev"),
            ("https://host.example.dev", "host.example.dev"),
        ] {
            assert_eq!(host_of(url).as_deref(), Some(host), "{url}");
        }
        for url in ["", "host.example.dev", "https://", "https:///x", "http://[::1", "http://[]/"] {
            assert_eq!(host_of(url), None, "{url:?}");
        }
    }

    #[test]
    fn hosts_compare_like_the_client_pin_does() {
        assert!(same_host("LiteLLM.Corp.", "litellm.corp"));
        assert!(same_host("[::1]", "::1"));
        assert!(!same_host("litellm.corp", "litellm.corp.evil.test"));
        assert!(!same_host("a.example.dev", "b.example.dev"));
    }

    #[test]
    fn ids_are_bounded_and_free_of_control_characters() {
        assert!(valid_id("team-a"));
        assert!(valid_id("équipe/α β+c@d.test"));
        assert!(valid_id(&"x".repeat(MAX_ID_CHARS)));
        assert!(!valid_id(""));
        assert!(!valid_id(&"x".repeat(MAX_ID_CHARS + 1)));
        assert!(!valid_id("a\nb"));
        assert!(!valid_id("a\u{0}b"));
    }
}
