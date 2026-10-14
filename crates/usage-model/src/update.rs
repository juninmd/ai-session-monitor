//! Safe, backend-owned updater state. Installer URLs, signatures and raw errors stay in Rust.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdatePhase {
    Idle,
    Checking,
    Current,
    Available,
    Downloading,
    Ready,
    Installing,
    Error,
    Unavailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateFailure {
    Check,
    Download,
    Signature,
    Install,
}

impl UpdatePhase {
    pub const ALL: [Self; 9] = [
        Self::Idle,
        Self::Checking,
        Self::Current,
        Self::Available,
        Self::Downloading,
        Self::Ready,
        Self::Installing,
        Self::Error,
        Self::Unavailable,
    ];
}

impl UpdateFailure {
    pub const ALL: [Self; 4] = [Self::Check, Self::Download, Self::Signature, Self::Install];
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatus {
    pub current_version: String,
    pub phase: UpdatePhase,
    pub available_version: Option<String>,
    pub notes: Option<String>,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub checked_at: Option<i64>,
    pub failure: Option<UpdateFailure>,
    /// False for debug builds and unsupported targets, even if a signed download is available.
    pub installable: bool,
}

impl UpdateStatus {
    pub fn new(version: impl Into<String>, installable: bool) -> Self {
        Self {
            current_version: version.into(),
            phase: UpdatePhase::Idle,
            available_version: None,
            notes: None,
            downloaded_bytes: 0,
            total_bytes: None,
            checked_at: None,
            failure: None,
            installable,
        }
    }
}
