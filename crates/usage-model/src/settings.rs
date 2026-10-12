//! The user's non-secret configuration. Secrets (the LiteLLM key) live in the Windows Credential
//! Manager and never appear here. Unknown JSON keys are ignored and missing ones fall back to the
//! defaults, so an older settings file keeps loading after an upgrade.

use serde::{Deserialize, Serialize};

use crate::HarnessId;

pub const SETTINGS_VERSION: u32 = 1;
pub const MIN_INTERVAL_SECS: u32 = 30;
pub const MAX_INTERVAL_SECS: u32 = 3600;
pub const MIN_LITELLM_POLL_SECS: u32 = 60;
pub const MAX_ALERT_THRESHOLDS: usize = 4;
pub const DEFAULT_ALERT_THRESHOLDS: [u8; 2] = [80, 95];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub version: u32,
    pub general: GeneralSettings,
    pub claude: ClaudeSettings,
    pub codex: CodexSettings,
    pub antigravity: AntigravitySettings,
    pub opencode: OpencodeSettings,
    pub kimi: KimiSettings,
    pub zai: ZaiSettings,
    pub litellm: LitellmSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: SETTINGS_VERSION,
            general: GeneralSettings::default(),
            claude: ClaudeSettings::default(),
            codex: CodexSettings::default(),
            antigravity: AntigravitySettings::default(),
            opencode: OpencodeSettings::default(),
            kimi: KimiSettings::default(),
            zai: ZaiSettings::default(),
            litellm: LitellmSettings::default(),
        }
    }
}

impl Settings {
    /// Clamp every numeric field into its supported range. Call after loading or patching.
    pub fn normalize(&mut self) {
        self.version = SETTINGS_VERSION;
        self.general.interval_secs =
            self.general.interval_secs.clamp(MIN_INTERVAL_SECS, MAX_INTERVAL_SECS);
        self.general.warn_at_pct = self.general.warn_at_pct.clamp(5, 90);
        normalize_thresholds(&mut self.general.alert_thresholds);
        self.litellm.poll_interval_secs =
            self.litellm.poll_interval_secs.clamp(MIN_LITELLM_POLL_SECS, MAX_INTERVAL_SECS);
        self.litellm.timeout_secs = self.litellm.timeout_secs.clamp(1, 30);
    }

    pub fn is_enabled(&self, id: HarnessId) -> bool {
        match id {
            HarnessId::Claude => self.claude.enabled,
            HarnessId::Codex => self.codex.enabled,
            HarnessId::Antigravity => self.antigravity.enabled,
            HarnessId::Opencode => self.opencode.enabled,
            HarnessId::Kimi => self.kimi.enabled,
            HarnessId::Zai => self.zai.enabled,
            HarnessId::Litellm => self.litellm.enabled,
        }
    }
}

/// Drops out-of-range values, sorts, dedupes and keeps the `MAX_ALERT_THRESHOLDS` lowest.
fn normalize_thresholds(values: &mut Vec<u8>) {
    values.retain(|pct| (1..=99).contains(pct));
    values.sort_unstable();
    values.dedup();
    values.truncate(MAX_ALERT_THRESHOLDS);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeFormat {
    /// `1h 53m`
    Hm,
    /// `1:53`
    Colon,
    /// `1,9 h`
    Decimal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WeekFormat {
    /// `107h 12m`
    Hours,
    /// `4d 11h`
    DaysHours,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GeneralSettings {
    /// How often collectors poll while the app runs.
    pub interval_secs: u32,
    /// Start with Windows. Off until the user opts in because it writes a Run registry entry.
    pub autostart: bool,
    /// Harness the tray ring follows. `None` means the lowest available percentage of all.
    pub tray_source: Option<HarnessId>,
    /// Available percentage at or below which a window turns to the warning colour.
    pub warn_at_pct: u8,
    pub notify_on_out: bool,
    pub notify_on_reset: bool,
    /// Used percentages (1..=99) that raise one toast each time a window crosses them. Empty
    /// switches the alerts off. Sorted, deduplicated, at most `MAX_ALERT_THRESHOLDS`.
    pub alert_thresholds: Vec<u8>,
    /// Poll less often while every window is far from its limit and its reset is far away.
    pub adaptive: bool,
    /// Keep a local record of past quota windows (percentages and times only) for the burndown.
    pub history_enabled: bool,
    pub time_format: TimeFormat,
    pub week_format: WeekFormat,
    /// Present quota as "used" instead of "available". Wording only: stored numbers stay used %.
    pub show_used: bool,
    /// The flyout opens by itself once, on the very first launch.
    pub first_run_done: bool,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            interval_secs: 60,
            autostart: false,
            tray_source: None,
            warn_at_pct: 25,
            notify_on_out: false,
            notify_on_reset: false,
            alert_thresholds: DEFAULT_ALERT_THRESHOLDS.to_vec(),
            adaptive: true,
            history_enabled: true,
            time_format: TimeFormat::Hm,
            week_format: WeekFormat::Hours,
            show_used: false,
            first_run_done: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ClaudeSettings {
    pub enabled: bool,
    /// The user agreed to install the statusLine command in Claude Code's settings.json.
    pub shim_consent: bool,
    /// Opt-in, off by default: also query the undocumented OAuth usage endpoint (Opus/Sonnet).
    pub oauth_usage: bool,
    /// Override of `%USERPROFILE%\.claude` (also honours `CLAUDE_CONFIG_DIR` when unset).
    pub config_dir: Option<String>,
}

impl Default for ClaudeSettings {
    fn default() -> Self {
        Self { enabled: true, shim_consent: false, oauth_usage: false, config_dir: None }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CodexSettings {
    pub enabled: bool,
    /// The user agreed to let the monitor spawn `codex app-server`.
    pub consent: bool,
    /// Explicit path to `codex`/`codex.cmd`/`codex.exe`; `None` searches PATH.
    pub binary_path: Option<String>,
}

impl Default for CodexSettings {
    fn default() -> Self {
        Self { enabled: true, consent: false, binary_path: None }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AntigravitySettings {
    pub enabled: bool,
    /// The user agreed to read the local language server (unofficial).
    pub consent: bool,
}

impl Default for AntigravitySettings {
    fn default() -> Self {
        Self { enabled: true, consent: false }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct OpencodeSettings {
    pub enabled: bool,
    /// The user agreed to let the monitor open OpenCode's local database read-only for the
    /// consumption estimate. The Go API key (remote windows) needs no flag: saving it is the consent.
    pub local_consent: bool,
    /// Override of OpenCode's data directory; `None` resolves it like OpenCode does.
    pub data_dir: Option<String>,
}

impl Default for OpencodeSettings {
    fn default() -> Self {
        Self { enabled: true, local_consent: false, data_dir: None }
    }
}

/// Kimi Code (Moonshot) quota. Saving the API key is the consent, so there is no flag for it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct KimiSettings {
    pub enabled: bool,
}

impl Default for KimiSettings {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// z.ai (GLM Coding Plan) quota. Saving the API key is the consent, so there is no flag for it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ZaiSettings {
    pub enabled: bool,
}

impl Default for ZaiSettings {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LitellmScope {
    /// The calling virtual key (`/key/info`).
    Key,
    /// A user budget (`/user/info`).
    User,
    /// A team budget (`/team/info`).
    Team,
}

/// How the LiteLLM server certificate is checked.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case", rename_all_fields = "camelCase")]
pub enum TlsMode {
    /// The Windows certificate store.
    System,
    /// Trust this PEM bundle (a private CA).
    CustomCa { pem_path: String },
    /// Skip verification, only for the host the user explicitly confirmed.
    Insecure { host: String },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LitellmSettings {
    pub enabled: bool,
    /// `https://host[/prefix]`; `None` until the user fills the connection form.
    pub base_url: Option<String>,
    pub scope: LitellmScope,
    /// User id or team id; required for `User`/`Team`, ignored for `Key`.
    pub entity_id: Option<String>,
    /// Header that carries the key; `Authorization` unless the proxy uses a custom one.
    pub auth_header: String,
    pub poll_interval_secs: u32,
    pub timeout_secs: u32,
    pub tls: TlsMode,
}

impl Default for LitellmSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            base_url: None,
            scope: LitellmScope::Key,
            entity_id: None,
            auth_header: "Authorization".to_owned(),
            poll_interval_secs: 300,
            timeout_secs: 10,
            tls: TlsMode::System,
        }
    }
}

#[cfg(test)]
mod tests;
