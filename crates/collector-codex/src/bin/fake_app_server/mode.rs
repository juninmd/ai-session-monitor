//! Which behaviour the fake plays.

use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Plus plan: both windows and a credit balance.
    Ok,
    /// Only the five-hour window.
    PlusOnly,
    /// Only the weekly window, as on a plan without a five-hour one.
    ProNoPrimary,
    /// Signed in with an API key, told by `account/read`.
    ApiKey,
    /// The same, told only by the error of `account/rateLimits/read` (a server without `account/read`).
    ApiKeyRpc,
    /// No sign-in is needed at all.
    NoAuthProvider,
    NotSignedIn,
    NotSignedInRpc,
    BadHandshake,
    BadHandshakeResult,
    /// Never answers anything.
    Hang,
    /// Answers the handshake and the account, never the limits.
    HangAfterInit,
    /// Every answer takes a while; none is slow enough to fail alone.
    Slow,
    GarbageLine,
    HugeLine,
    ExitsEarly,
    ExitsMidway,
    /// The backend answered the limits request with this HTTP status.
    Backend(u16),
    Offline,
    BadShape,
    BadNumbers,
    /// Notifications, stray answers and a request of its own around every reply.
    Noisy,
    /// Starts a child of its own that outlives it unless the whole tree is killed.
    Grandchild,
}

impl Mode {
    pub fn parse(name: &str) -> Option<Self> {
        let name = name.trim();
        if let Some(status) = name.strip_prefix("backend_") {
            return status.parse().ok().map(Self::Backend);
        }
        Some(match name {
            "ok" => Self::Ok,
            "plus_only" => Self::PlusOnly,
            "pro_no_primary" => Self::ProNoPrimary,
            "api_key" => Self::ApiKey,
            "api_key_rpc" => Self::ApiKeyRpc,
            "no_auth_provider" => Self::NoAuthProvider,
            "not_signed_in" => Self::NotSignedIn,
            "not_signed_in_rpc" => Self::NotSignedInRpc,
            "bad_handshake" => Self::BadHandshake,
            "bad_handshake_result" => Self::BadHandshakeResult,
            "hang" => Self::Hang,
            "hang_after_init" => Self::HangAfterInit,
            "slow" => Self::Slow,
            "garbage_line" => Self::GarbageLine,
            "huge_line" => Self::HugeLine,
            "exits_early" => Self::ExitsEarly,
            "exits_midway" => Self::ExitsMidway,
            "offline" => Self::Offline,
            "bad_shape" => Self::BadShape,
            "bad_numbers" => Self::BadNumbers,
            "noisy" => Self::Noisy,
            "grandchild" => Self::Grandchild,
            _ => return None,
        })
    }
}

/// `None` for a name nobody defined, so that a typo in a test fails loudly.
pub fn load(dir: &Path) -> Option<Mode> {
    let name = std::fs::read_to_string(dir.join("fake-mode"))
        .ok()
        .or_else(|| std::env::var("FAKE_MODE").ok());
    match name {
        Some(name) => Mode::parse(&name),
        None => Some(Mode::Ok),
    }
}
