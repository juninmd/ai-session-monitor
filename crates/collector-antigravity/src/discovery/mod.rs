//! Finding the language server of the running Antigravity app without admin rights.
//!
//! `Discovery` is the seam: production reads the same-user process table and `netstat`
//! ([`SystemDiscovery`]); tests hand the collector a loopback fake instead of touching real
//! processes. Everything pure (flags, port tables, target planning) lives here and is unit tested.

mod flags;
mod netstat;
mod system;

use std::collections::BTreeSet;
use std::future::Future;

use usage_model::{ErrorCode, HarnessError};
use usage_support::Secret;

pub(crate) use flags::ServerFlags;
use flags::usable;
pub(crate) use system::SystemDiscovery;

/// More than this many HTTPS ports of one process are not worth probing.
const MAX_HTTPS_PORTS: usize = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Scheme {
    Https,
    Http,
}

impl Scheme {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Https => "https",
            Self::Http => "http",
        }
    }
}

/// One address to call, with the token that server expects. `Debug` masks the token.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Target {
    pub scheme: Scheme,
    pub port: u16,
    pub token: Secret,
}

/// Why there is nothing to call.
#[derive(Clone, Debug)]
pub(crate) enum DiscoveryError {
    /// No Antigravity language server process: the IDE is closed, a normal state.
    NotRunning,
    /// A server process exists but offers no address yet (starting up), or its port table could
    /// not be read (then the cause comes along).
    NoPort(Option<HarnessError>),
    /// A server process exists but its token is missing or unusable.
    NoToken,
    /// The process table itself could not be read.
    Failed(HarnessError),
}

impl From<DiscoveryError> for HarnessError {
    fn from(error: DiscoveryError) -> Self {
        match error {
            DiscoveryError::NotRunning => Self::new(ErrorCode::NotRunning, true),
            DiscoveryError::NoPort(None) => {
                Self::new(ErrorCode::NotRunning, true).with_param("reason", "no_port")
            }
            DiscoveryError::NoPort(Some(cause)) | DiscoveryError::Failed(cause) => cause,
            DiscoveryError::NoToken => {
                Self::new(ErrorCode::Unauthorized, false).with_param("reason", "no_token")
            }
        }
    }
}

pub(crate) trait Discovery: Send + Sync {
    /// Addresses to try, most promising first.
    fn find(&self) -> impl Future<Output = Result<Vec<Target>, DiscoveryError>> + Send;
}

/// The addresses of one server: HTTPS on the port it was told to use and on every port it listens
/// on, then HTTP on the extension server port as a last resort. A server may listen on several
/// ports and only one speaks the protocol, so a wrong guess costs one refused connection.
pub(crate) fn plan_targets(flags: &ServerFlags, listening: &BTreeSet<u16>) -> Vec<Target> {
    let mut targets = Vec::new();
    if let Some(token) = usable(flags.csrf_token.as_ref()) {
        let mut seen = BTreeSet::new();
        let ports = flags.https_port.into_iter().chain(listening.iter().copied());
        for port in ports.filter(|port| Some(*port) != flags.lsp_port) {
            if seen.insert(port) && seen.len() <= MAX_HTTPS_PORTS {
                targets.push(Target { scheme: Scheme::Https, port, token: token.clone() });
            }
        }
    }
    let http_token =
        usable(flags.extension_csrf_token.as_ref()).or_else(|| usable(flags.csrf_token.as_ref()));
    if let (Some(port), Some(token)) = (flags.extension_port, http_token) {
        targets.push(Target { scheme: Scheme::Http, port, token });
    }
    targets
}

#[cfg(test)]
mod tests;
