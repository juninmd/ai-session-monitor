//! Settings of the sources that have more than an enabled flag: the consent flags of Claude, Codex,
//! Cursor, Copilot, Antigravity and OpenCode, and the LiteLLM connection.

use serde::{Deserialize, Serialize};

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
pub struct CursorSettings {
    pub enabled: bool,
    /// The user agreed to read the sign-in token Cursor keeps in its local `state.vscdb` and to
    /// send it to cursor.com (unofficial).
    pub consent: bool,
}

impl Default for CursorSettings {
    fn default() -> Self {
        Self { enabled: true, consent: false }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CopilotSettings {
    pub enabled: bool,
    /// The user agreed to read the GitHub token the Copilot extension keeps in its local
    /// `apps.json`/`hosts.json` and to send it to api.github.com (unofficial endpoint). A token
    /// pasted by the user (`SecretKind::Copilot`) needs no flag: saving it is the consent.
    pub local_consent: bool,
}

impl Default for CopilotSettings {
    fn default() -> Self {
        Self { enabled: true, local_consent: false }
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
