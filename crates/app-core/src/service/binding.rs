//! Where a stored LiteLLM key may be sent. The key is entered for one connection, so settings that
//! point the read somewhere else must not inherit it. A speed bump against a typo or a misbehaving
//! webview, not a boundary: whoever can rewrite the settings can also type a key for any server.

use url::{Host, Url};
use usage_model::settings::{LitellmSettings, TlsMode};

/// How the server certificate is trusted. An insecure host is pinned to the URL's host by
/// validation, so the mode alone says what the host would.
#[derive(PartialEq, Eq)]
enum Trust {
    System,
    CustomCa(String),
    Insecure,
}

/// The parts of a connection that decide who receives the key, normalized so that respelling the
/// URL (case, default port, trailing slash) is not a change. The URL parser lowercases the host.
#[derive(PartialEq, Eq)]
struct Destination {
    scheme: String,
    host: Host<String>,
    port: Option<u16>,
    path: String,
    header: String,
    trust: Trust,
}

/// `None` when there is no usable URL; the collector sends nothing then.
fn destination(litellm: &LitellmSettings) -> Option<Destination> {
    let url = Url::parse(litellm.base_url.as_deref()?.trim()).ok()?;
    let host = match url.host()? {
        Host::Domain(name) => Host::Domain(name.trim_end_matches('.').to_owned()),
        address => address.to_owned(),
    };
    let trust = match &litellm.tls {
        TlsMode::System => Trust::System,
        TlsMode::CustomCa { pem_path } => Trust::CustomCa(pem_path.trim().to_owned()),
        TlsMode::Insecure { .. } => Trust::Insecure,
    };
    Some(Destination {
        scheme: url.scheme().to_owned(),
        host,
        port: url.port_or_known_default(),
        path: url.path().trim_end_matches('/').to_owned(),
        header: litellm.auth_header.trim().to_ascii_lowercase(),
        trust,
    })
}

/// True when a key entered for `old` must not be used with `new`. Having no destination is a
/// state of its own: leaving it or arriving at it is a change, so two saves cannot launder a key.
pub(super) fn moves_the_key(old: &LitellmSettings, new: &LitellmSettings) -> bool {
    destination(old) != destination(new)
}
