//! The user's non-secret configuration. Secrets (the LiteLLM key) live in the Windows Credential
//! Manager and never appear here. Unknown JSON keys are ignored and missing ones fall back to the
//! defaults, so an older settings file keeps loading after an upgrade.

use serde::{Deserialize, Serialize};

use crate::HarnessId;

pub const SETTINGS_VERSION: u32 = 1;
pub const MIN_INTERVAL_SECS: u32 = 30;
pub const MAX_INTERVAL_SECS: u32 = 3600;
pub const MIN_LITELLM_POLL_SECS: u32 = 60;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub version: u32,
    pub general: GeneralSettings,
    pub claude: ClaudeSettings,
    pub codex: CodexSettings,
    pub antigravity: AntigravitySettings,
    pub opencode: OpencodeSettings,
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
            HarnessId::Litellm => self.litellm.enabled,
        }
    }
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
    pub time_format: TimeFormat,
    pub week_format: WeekFormat,
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
            time_format: TimeFormat::Hm,
            week_format: WeekFormat::Hours,
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
mod tests {
    use super::*;

    #[test]
    fn empty_json_yields_defaults() {
        let settings: Settings = serde_json::from_str("{}").expect("parse");
        assert_eq!(settings, Settings::default());
        assert!(!settings.claude.oauth_usage, "the unofficial source must be opt-in");
        assert!(!settings.general.autostart, "autostart must be opt-in");
    }

    #[test]
    fn unknown_keys_are_ignored_and_known_ones_kept() {
        let json = r#"{"general":{"intervalSecs":120,"futureKey":1},"litellm":{"scope":"team","entityId":"t-1"}}"#;
        let settings: Settings = serde_json::from_str(json).expect("parse");
        assert_eq!(settings.general.interval_secs, 120);
        assert_eq!(settings.litellm.scope, LitellmScope::Team);
        assert_eq!(settings.litellm.entity_id.as_deref(), Some("t-1"));
        assert!(settings.codex.enabled);
    }

    #[test]
    fn normalize_clamps_out_of_range_values() {
        let mut settings = Settings::default();
        settings.general.interval_secs = 1;
        settings.general.warn_at_pct = 99;
        settings.litellm.poll_interval_secs = 5;
        settings.litellm.timeout_secs = 999;
        settings.normalize();
        assert_eq!(settings.general.interval_secs, MIN_INTERVAL_SECS);
        assert_eq!(settings.general.warn_at_pct, 90);
        assert_eq!(settings.litellm.poll_interval_secs, MIN_LITELLM_POLL_SECS);
        assert_eq!(settings.litellm.timeout_secs, 30);
    }

    #[test]
    fn tls_mode_round_trips_with_a_mode_tag() {
        let tls = TlsMode::Insecure { host: "gateway.example.dev".into() };
        let json = serde_json::to_string(&tls).expect("serialize");
        assert_eq!(json, r#"{"mode":"insecure","host":"gateway.example.dev"}"#);
        assert_eq!(serde_json::from_str::<TlsMode>(&json).expect("parse"), tls);
    }

    #[test]
    fn custom_ca_field_is_camel_case() {
        let tls = TlsMode::CustomCa { pem_path: r"C:\certs\corp.pem".into() };
        let json = serde_json::to_value(&tls).expect("serialize");
        assert_eq!(json["mode"], "custom_ca");
        assert_eq!(json["pemPath"], r"C:\certs\corp.pem");
    }

    #[test]
    fn settings_hold_no_secret_fields() {
        let json = serde_json::to_string(&Settings::default()).expect("serialize").to_lowercase();
        for forbidden in ["apikey", "token", "password", "secret"] {
            assert!(!json.contains(forbidden), "settings JSON mentions {forbidden}");
        }
    }
}
