//! Which URLs the client may call, and safe ways to build them. Nothing here touches the network.

use std::net::IpAddr;

use reqwest::Url;
use usage_model::{ErrorCode, HarnessError};

/// Longer URLs are refused: they come from settings and no endpoint here needs more.
const MAX_URL_BYTES: usize = 4096;

fn invalid_url(host: Option<&str>) -> HarnessError {
    let error = HarnessError::new(ErrorCode::InvalidUrl, false);
    match host {
        Some(host) => error.with_param("host", host),
        None => error,
    }
}

/// Parses `raw` and enforces the policy: `https` (or `http` to loopback when allowed), a host, no
/// userinfo, no fragment, no control characters, no surrounding whitespace. A path prefix and a
/// query are kept. The error carries the host when it is known, never the rest of the URL.
pub(crate) fn parse_url(raw: &str, allow_loopback_http: bool) -> Result<Url, HarnessError> {
    let malformed = raw.is_empty()
        || raw.len() > MAX_URL_BYTES
        || raw.chars().any(char::is_control)
        || raw.trim() != raw;
    if malformed {
        return Err(invalid_url(None));
    }
    let url = Url::parse(raw).map_err(|_| invalid_url(None))?;
    let host = match url.host_str() {
        Some(host) if !host.is_empty() => host.to_owned(),
        _ => return Err(invalid_url(None)),
    };
    let scheme_ok = match url.scheme() {
        "https" => true,
        "http" => allow_loopback_http && is_loopback_host(&host),
        _ => false,
    };
    let has_credentials = !url.username().is_empty() || url.password().is_some();
    if !scheme_ok || has_credentials || url.fragment().is_some() {
        return Err(invalid_url(Some(&host)));
    }
    Ok(url)
}

/// Applies the URL policy without keeping the result, for validating settings at save time.
pub fn check_url(raw: &str, allow_loopback_http: bool) -> Result<(), HarnessError> {
    parse_url(raw, allow_loopback_http).map(|_| ())
}

/// `127.0.0.0/8`, `::1` (also as an IPv4-mapped address) and `localhost`.
pub(crate) fn is_loopback_host(host: &str) -> bool {
    let host = host.trim_start_matches('[').trim_end_matches(']');
    host.eq_ignore_ascii_case("localhost")
        || host.parse::<IpAddr>().is_ok_and(|ip| ip.to_canonical().is_loopback())
}

/// Lowercased, without IPv6 brackets or a trailing dot, so `LiteLLM.Corp.` equals `litellm.corp`.
pub(crate) fn normalize_host(host: &str) -> String {
    let host = host.trim().trim_start_matches('[').trim_end_matches(']');
    host.strip_suffix('.').unwrap_or(host).to_ascii_lowercase()
}

/// Percent-encodes everything except RFC 3986 unreserved characters, byte by byte (UTF-8).
pub(crate) fn encode_component(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            out.push(char::from(byte));
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

/// Appends `pairs` to the query of `url`, percent-encoding keys and values and keeping any query
/// already there.
pub fn append_query(url: &str, pairs: &[(&str, &str)]) -> Result<String, HarnessError> {
    let mut url = Url::parse(url).map_err(|_| invalid_url(None))?;
    add_query(&mut url, pairs.iter().map(|&(key, value)| (key, value)));
    Ok(url.into())
}

pub(crate) fn add_query<'a>(url: &mut Url, pairs: impl Iterator<Item = (&'a str, &'a str)>) {
    let mut query = url.query().unwrap_or_default().to_owned();
    for (key, value) in pairs {
        if !query.is_empty() {
            query.push('&');
        }
        query.push_str(&encode_component(key));
        query.push('=');
        query.push_str(&encode_component(value));
    }
    if !query.is_empty() {
        url.set_query(Some(&query));
    }
}

/// Appends path segments after the existing path (a prefix such as `/litellm` is kept),
/// percent-encoding each one. Empty, `.` and `..` segments are refused: they would change where
/// the request goes.
pub fn append_path(url: &str, segments: &[&str]) -> Result<String, HarnessError> {
    let mut url = Url::parse(url).map_err(|_| invalid_url(None))?;
    add_path(&mut url, segments.iter().copied())?;
    Ok(url.into())
}

pub(crate) fn add_path<'a>(
    url: &mut Url,
    segments: impl Iterator<Item = &'a str>,
) -> Result<(), HarnessError> {
    let mut path = url.path().trim_end_matches('/').to_owned();
    let mut any = false;
    for segment in segments {
        if matches!(segment, "" | "." | "..") {
            return Err(invalid_url(url.host_str()));
        }
        path.push('/');
        path.push_str(&encode_component(segment));
        any = true;
    }
    if any {
        url.set_path(&path);
    }
    Ok(())
}
