//! Payloads that cross the Tauri IPC boundary besides `Snapshot` and `Settings`. Mirrored by Zod in
//! `src/lib/contract/ipc.ts`; `contract/ipc.json` pins both sides to the same shapes.

use serde::{Deserialize, Serialize};

/// Secrets live only in the Windows Credential Manager. The UI can write or clear them and ask
/// whether they exist; it can never read them back.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecretKind {
    Litellm,
    Opencode,
    Kimi,
    Zai,
    Openrouter,
    Deepseek,
}

impl SecretKind {
    pub const ALL: [Self; 6] =
        [Self::Litellm, Self::Opencode, Self::Kimi, Self::Zai, Self::Openrouter, Self::Deepseek];

    /// Credential Manager target name suffix.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Litellm => "litellm",
            Self::Opencode => "opencode",
            Self::Kimi => "kimi",
            Self::Zai => "zai",
            Self::Openrouter => "openrouter",
            Self::Deepseek => "deepseek",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecretStatus {
    pub litellm: bool,
    pub opencode: bool,
    pub kimi: bool,
    pub zai: bool,
    pub openrouter: bool,
    pub deepseek: bool,
}

/// Known destinations the UI may ask the backend to open. The UI never passes a URL, so a
/// compromised webview cannot make the app open arbitrary targets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalLink {
    OpencodeGoConsole,
    OpencodeBilling,
    KimiConsole,
    ZaiConsole,
    OpenrouterConsole,
    DeepseekConsole,
    ProjectReadme,
}

impl ExternalLink {
    pub const ALL: [Self; 7] = [
        Self::OpencodeGoConsole,
        Self::OpencodeBilling,
        Self::KimiConsole,
        Self::ZaiConsole,
        Self::OpenrouterConsole,
        Self::DeepseekConsole,
        Self::ProjectReadme,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShimState {
    /// No `statusLine` in Claude Code's settings.json.
    NotInstalled,
    /// `statusLine` runs our shim.
    Installed,
    /// `statusLine` runs somebody else's command; installing would wrap it.
    Foreign,
    /// Our shim is configured but its executable moved or is missing.
    Outdated,
    /// settings.json exists but cannot be read or parsed; nothing is touched.
    Unreadable,
}

impl ShimState {
    pub const ALL: [Self; 5] =
        [Self::NotInstalled, Self::Installed, Self::Foreign, Self::Outdated, Self::Unreadable];
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShimStatus {
    pub state: ShimState,
    /// The shim forwards to the user's previous statusLine command.
    pub chained: bool,
    /// Absolute path of the Claude Code settings.json being managed; display only.
    pub settings_path: Option<String>,
}
