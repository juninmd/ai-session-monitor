//! What a collector produces and the UI renders. Every field is always serialized (`null`, never
//! omitted) so the TypeScript contract in `src/lib/contract/view.ts` has a single shape per type.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::HarnessId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewState {
    /// The user switched this source off.
    Disabled,
    /// Nothing is read until the user grants consent (see `Settings`).
    NeedsConsent,
    /// Configuration is missing (for example the LiteLLM URL or key).
    NeedsSetup,
    /// First read still in flight.
    Loading,
    /// `windows` hold usable data. The data may be stale, see `HarnessView::stale`.
    Ready,
    /// The source works but has not reported anything yet (no Claude Code session so far).
    NoData,
    /// The read failed, see `HarnessView::error`.
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowKind {
    FiveHour,
    Weekly,
    Monthly,
    Daily,
    Budget,
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AmountUnit {
    Usd,
    Credits,
    Requests,
    Tokens,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Amount {
    pub used: f64,
    /// `None` means the source reports no cap (not "unlimited": the cap may live elsewhere).
    pub limit: Option<f64>,
    pub unit: AmountUnit,
}

/// One quota window: a percentage plus the moment it resets.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageWindow {
    /// Stable within one harness, for example `five_hour`, `weekly`, `gemini:five_hour`.
    pub id: String,
    pub kind: WindowKind,
    /// Pool or model family this window belongs to (`opus`, `sonnet`, `gemini`, `claude_gpt`).
    pub group: Option<String>,
    /// Provider-supplied display name (a model or budget name). `None`: the UI names it by kind.
    pub label: Option<String>,
    /// Length of the window in minutes when the source states it.
    pub window_minutes: Option<u32>,
    /// 0..=100. `None` means unknown and must never be shown as 0 %.
    pub used_pct: Option<f64>,
    /// Epoch seconds (UTC) at which the window resets.
    pub resets_at: Option<i64>,
    pub amount: Option<Amount>,
}

impl UsageWindow {
    pub fn new(id: impl Into<String>, kind: WindowKind) -> Self {
        Self {
            id: id.into(),
            kind,
            group: None,
            label: None,
            window_minutes: None,
            used_pct: None,
            resets_at: None,
            amount: None,
        }
    }
}

/// A small labelled number or text (`rpm_limit`, `credits_balance`, ...).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fact {
    pub id: String,
    pub number: Option<f64>,
    pub text: Option<String>,
    pub unit: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Warn,
    Error,
}

/// A short code the UI turns into a localized sentence. `params` never carry secrets.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Notice {
    pub code: String,
    pub severity: Severity,
    pub params: BTreeMap<String, String>,
}

/// Failure codes the UI maps to a message and an action. Keep in sync with `ERROR_CODES` in
/// `src/lib/contract/view.ts`; `error_codes_match_the_shared_list` fails when they drift.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    Internal,
    Timeout,
    Network,
    BadResponse,
    ResponseTooLarge,
    RateLimited,
    Unauthorized,
    NotFound,
    ServerError,
    TlsError,
    InvalidUrl,
    /// A value the user typed is unusable (empty secret, missing id for the chosen scope).
    InvalidInput,
    SettingsUnreadable,
    InstallFailed,
    UnsupportedPlan,
    OauthExpired,
    BinaryNotFound,
    SpawnFailed,
    HandshakeFailed,
    ProtocolError,
    NotSignedIn,
    NotRunning,
    EndpointChanged,
}

impl ErrorCode {
    pub const ALL: [Self; 23] = [
        Self::Internal,
        Self::Timeout,
        Self::Network,
        Self::BadResponse,
        Self::ResponseTooLarge,
        Self::RateLimited,
        Self::Unauthorized,
        Self::NotFound,
        Self::ServerError,
        Self::TlsError,
        Self::InvalidUrl,
        Self::InvalidInput,
        Self::SettingsUnreadable,
        Self::InstallFailed,
        Self::UnsupportedPlan,
        Self::OauthExpired,
        Self::BinaryNotFound,
        Self::SpawnFailed,
        Self::HandshakeFailed,
        Self::ProtocolError,
        Self::NotSignedIn,
        Self::NotRunning,
        Self::EndpointChanged,
    ];
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HarnessError {
    pub code: ErrorCode,
    /// Worth retrying on the next tick (network blips) versus needing the user (401, not found).
    pub retryable: bool,
    /// Non-secret details such as `httpStatus`. Never include tokens, keys or request URLs with keys.
    pub params: BTreeMap<String, String>,
}

impl HarnessError {
    pub fn new(code: ErrorCode, retryable: bool) -> Self {
        Self { code, retryable, params: BTreeMap::new() }
    }

    #[must_use]
    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    /// Machine id such as `statusline`, `oauth_usage`, `app_server`, `rollout`, `key_info`.
    pub id: String,
    /// `true` only for a documented, vendor-supported interface.
    pub official: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HarnessView {
    pub id: HarnessId,
    pub state: ViewState,
    pub source: Option<Source>,
    /// Epoch seconds at which the underlying data was produced (file mtime, response time),
    /// not when we polled. Drives the "read N min ago" chip.
    pub read_at: Option<i64>,
    /// The data is older than its source promises or a window already reset. Windows stay
    /// visible but must be dimmed and not counted in the tray summary.
    pub stale: bool,
    pub plan: Option<String>,
    pub windows: Vec<UsageWindow>,
    pub facts: Vec<Fact>,
    pub notices: Vec<Notice>,
    /// Set in the `Error` state, and also on a `stale` `Ready` view whose last refresh failed (the
    /// windows are then the previous good ones).
    pub error: Option<HarnessError>,
    /// Display-only strings (`alias`, `host`, `scope`, ...). Never secrets.
    pub meta: BTreeMap<String, String>,
}

impl HarnessView {
    pub fn with_state(id: HarnessId, state: ViewState) -> Self {
        Self {
            id,
            state,
            source: None,
            read_at: None,
            stale: false,
            plan: None,
            windows: Vec::new(),
            facts: Vec::new(),
            notices: Vec::new(),
            error: None,
            meta: BTreeMap::new(),
        }
    }

    pub fn loading(id: HarnessId) -> Self {
        Self::with_state(id, ViewState::Loading)
    }

    pub fn disabled(id: HarnessId) -> Self {
        Self::with_state(id, ViewState::Disabled)
    }

    pub fn needs_consent(id: HarnessId) -> Self {
        Self::with_state(id, ViewState::NeedsConsent)
    }

    pub fn needs_setup(id: HarnessId) -> Self {
        Self::with_state(id, ViewState::NeedsSetup)
    }

    pub fn error(id: HarnessId, error: HarnessError) -> Self {
        let mut view = Self::with_state(id, ViewState::Error);
        view.error = Some(error);
        view
    }
}

/// Everything the UI needs, in tab order.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    /// Epoch seconds.
    pub generated_at: i64,
    pub views: Vec<HarnessView>,
}

#[cfg(test)]
mod tests;
