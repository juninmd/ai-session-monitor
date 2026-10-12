//! Checks a `Settings` value before it is saved. Numeric ranges are not checked here:
//! `Settings::normalize` clamps them. Errors name the offending field and never carry its value.

use std::path::{Component, Path, Prefix};

use url::Url;
use usage_model::settings::{LitellmScope, LitellmSettings, TlsMode};
use usage_model::{ErrorCode, HarnessError, Settings};
use usage_support::check_url;

const MAX_ENTITY_CHARS: usize = 256;
const MAX_HEADER_NAME_CHARS: usize = 128;
const MAX_PATH_CHARS: usize = 1024;

/// Headers the HTTP client owns (mirrors `usage_support::http::request`); using one to carry the
/// key would make every poll fail, so it is refused up front.
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

pub(crate) fn invalid_input(field: &'static str) -> HarnessError {
    HarnessError::new(ErrorCode::InvalidInput, false).with_param("field", field)
}

fn invalid_url(field: &'static str) -> HarnessError {
    HarnessError::new(ErrorCode::InvalidUrl, false).with_param("field", field)
}

/// True when `text` has more than `max` characters, without scanning a huge string to the end.
pub(crate) fn exceeds_chars(text: &str, max: usize) -> bool {
    text.len() > max.saturating_mul(4) || text.chars().count() > max
}

/// The first problem found, in form order (connection first, then the local paths).
pub(crate) fn validate(settings: &Settings) -> Result<(), HarnessError> {
    let litellm = &settings.litellm;
    let base = litellm.base_url.as_deref().map(check_base_url).transpose()?;
    check_entity(litellm)?;
    check_header_name(&litellm.auth_header)?;
    check_tls(&litellm.tls, base.as_ref())?;
    optional_path("configDir", settings.claude.config_dir.as_deref())?;
    optional_path("binaryPath", settings.codex.binary_path.as_deref())?;
    optional_path("dataDir", settings.opencode.data_dir.as_deref())
}

/// https, or http to loopback only; no userinfo, query or fragment.
fn check_base_url(raw: &str) -> Result<Url, HarnessError> {
    let bad = || invalid_url("baseUrl");
    check_url(raw, true).map_err(|_| bad())?;
    let url = Url::parse(raw).map_err(|_| bad())?;
    if url.query().is_some() {
        return Err(bad());
    }
    Ok(url)
}

fn check_entity(litellm: &LitellmSettings) -> Result<(), HarnessError> {
    let bad = || invalid_input("entityId");
    match litellm.entity_id.as_deref() {
        Some(id) if exceeds_chars(id, MAX_ENTITY_CHARS) || id.chars().any(char::is_control) => {
            Err(bad())
        }
        Some(id) if !id.trim().is_empty() => Ok(()),
        _ if litellm.scope == LitellmScope::Key => Ok(()),
        _ => Err(bad()),
    }
}

/// Letters, digits and hyphen, as the LiteLLM collector and the settings form require (a wider
/// RFC 9110 token would be saved and then fail every poll), and not a header the client manages.
fn check_header_name(name: &str) -> Result<(), HarnessError> {
    let plain = |byte: u8| byte.is_ascii_alphanumeric() || byte == b'-';
    let usable = !name.is_empty()
        && name.len() <= MAX_HEADER_NAME_CHARS
        && name.bytes().all(plain)
        && !MANAGED_HEADERS.iter().any(|managed| managed.eq_ignore_ascii_case(name));
    if usable { Ok(()) } else { Err(invalid_input("authHeader")) }
}

fn check_tls(tls: &TlsMode, base: Option<&Url>) -> Result<(), HarnessError> {
    match tls {
        TlsMode::System => Ok(()),
        TlsMode::CustomCa { pem_path } => check_path("pemPath", pem_path),
        TlsMode::Insecure { host } => {
            let confirmed = normalize_host(host);
            let target = base.and_then(Url::host_str).map(normalize_host);
            if !confirmed.is_empty() && target.as_deref() == Some(confirmed.as_str()) {
                Ok(())
            } else {
                Err(invalid_input("host"))
            }
        }
    }
}

/// Same spelling rules as the HTTP client uses to match the confirmed host.
fn normalize_host(host: &str) -> String {
    let host = host.trim().trim_start_matches('[').trim_end_matches(']');
    host.strip_suffix('.').unwrap_or(host).to_ascii_lowercase()
}

fn optional_path(field: &'static str, path: Option<&str>) -> Result<(), HarnessError> {
    path.map_or(Ok(()), |path| check_path(field, path))
}

fn check_path(field: &'static str, path: &str) -> Result<(), HarnessError> {
    let usable = !exceeds_chars(path, MAX_PATH_CHARS)
        && !path.contains('\0')
        && Path::new(path).is_absolute()
        && is_plain_local(Path::new(path));
    if usable { Ok(()) } else { Err(invalid_input(field)) }
}

/// A path on a drive. A share (`\\host\x`) would open a network connection, with the user's
/// credentials, to whatever host the path names; a device path or a reserved name (`NUL`,
/// `COM1`) is not a file and can block the reader. Mapped drives keep working.
fn is_plain_local(path: &Path) -> bool {
    path.components().all(|part| match part {
        Component::Prefix(prefix) => {
            matches!(prefix.kind(), Prefix::Disk(_) | Prefix::VerbatimDisk(_))
        }
        Component::Normal(name) => !is_reserved_device(&name.to_string_lossy()),
        _ => true,
    })
}

/// `CON`, `PRN`, `AUX`, `NUL`, `COM1`-`COM9` and `LPT1`-`LPT9`, with or without an extension.
fn is_reserved_device(name: &str) -> bool {
    let stem = name.split('.').next().unwrap_or(name).trim_end().to_ascii_uppercase();
    let numbered = ["COM", "LPT"].iter().any(|prefix| {
        stem.strip_prefix(prefix).is_some_and(|n| matches!(n.as_bytes(), [b'1'..=b'9']))
    });
    numbered || matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
}

#[cfg(test)]
mod header_tests;
#[cfg(test)]
mod path_tests;
#[cfg(test)]
mod tests;
