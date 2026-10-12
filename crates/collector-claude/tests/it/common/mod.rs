//! Scaffolding shared by every test of this crate: a throwaway home, config and data layout under
//! a temp dir (the real profile is never read or written), a fabricated "shim executable" and
//! writers for the files the collector reads. The unit tests include this file through `#[path]`,
//! so it must not name the crate under test.
#![allow(
    dead_code,
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "shared by test targets that each use part of it; tests assert by panicking"
)]

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::json;
use usage_model::settings::ClaudeSettings;
use usage_model::{Ctx, HarnessView};

/// The clock of every test; the fixtures' timestamps are relative to it.
pub const NOW: i64 = 1_800_000_000;
/// Obviously fake; the tests assert it never shows up in a view, an error or a notice.
pub const FAKE_TOKEN: &str = "fake-access-token-0001";
const SHIM_BUILD: &[u8] = b"fabricated shim build 1\n";

/// Fixtures are normalised to LF so a checkout with CRLF conversion cannot change an assertion.
fn lf(text: &str) -> String {
    text.replace("\r\n", "\n")
}

pub fn ok_both() -> String {
    lf(include_str!("../../fixtures/rate_limits/ok_both.json"))
}
pub fn ok_spend() -> String {
    lf(include_str!("../../fixtures/rate_limits/ok_spend.json"))
}
pub fn null_limits() -> String {
    lf(include_str!("../../fixtures/rate_limits/null_limits.json"))
}
pub fn settings_foreign() -> String {
    lf(include_str!("../../fixtures/settings/foreign.json"))
}
pub fn settings_plain() -> String {
    lf(include_str!("../../fixtures/settings/no_status_line.json"))
}
pub fn settings_static() -> String {
    lf(include_str!("../../fixtures/settings/static_status_line.json"))
}
pub fn usage_limits() -> String {
    lf(include_str!("../../fixtures/oauth/usage_limits.json"))
}
pub fn usage_flat() -> String {
    lf(include_str!("../../fixtures/oauth/usage_flat.json"))
}
pub fn usage_no_model_caps() -> String {
    lf(include_str!("../../fixtures/oauth/usage_no_model_caps.json"))
}

/// Claude Code's login file as the community describes it, with a fake token.
pub fn credentials_json(expires_at_ms: i64) -> String {
    json!({"claudeAiOauth": {
        "accessToken": FAKE_TOKEN, "refreshToken": "fake-refresh-token-0001",
        "expiresAt": expires_at_ms, "scopes": ["user:inference", "user:profile"],
        "subscriptionType": "max", "rateLimitTier": "default_claude_max_5x"
    }})
    .to_string()
}

pub struct Env {
    root: tempfile::TempDir,
    pub home: PathBuf,
    pub data: PathBuf,
}

impl Env {
    /// A home with an accented, spaced name (as on many pt-BR machines) and the data dir where
    /// the app puts it, below the home.
    pub fn new() -> Self {
        let root = tempfile::Builder::new().prefix("aism claude ").tempdir().unwrap();
        let home = root.path().join("Usuário Ação");
        let data = home.join("AppData").join("Local").join("ai-session-monitor");
        fs::create_dir_all(&home).unwrap();
        let env = Self { root, home, data };
        // Present from the start, so snapshots taken at any time agree about it.
        env.shim_source_with(SHIM_BUILD);
        env
    }

    /// The data dir is not below the home dir, so no `~` form can be used for it.
    pub fn with_data_outside_home() -> Self {
        let mut env = Self::new();
        env.data = env.root.path().join("Dados do usuário");
        env
    }

    pub fn root(&self) -> &Path {
        self.root.path()
    }

    /// Enabled and consented, OAuth off, the default config dir.
    pub fn settings(&self) -> ClaudeSettings {
        ClaudeSettings { shim_consent: true, ..ClaudeSettings::default() }
    }

    pub fn ctx(&self) -> Ctx {
        self.ctx_at(NOW)
    }

    pub fn ctx_at(&self, now: i64) -> Ctx {
        Ctx::new(now, self.data.clone(), self.home.clone())
    }

    pub fn config_dir(&self) -> PathBuf {
        self.home.join(".claude")
    }
    pub fn settings_json(&self) -> PathBuf {
        self.config_dir().join("settings.json")
    }
    pub fn backup(&self) -> PathBuf {
        self.config_dir().join("settings.json.aism-backup")
    }
    /// The second copy of the saved statusLine, beside the file it describes.
    pub fn state_copy(&self) -> PathBuf {
        self.config_dir().join("settings.json.aism-state")
    }
    pub fn credentials(&self) -> PathBuf {
        self.config_dir().join(".credentials.json")
    }
    pub fn rate_limits(&self) -> PathBuf {
        self.data.join("claude").join("rate_limits.json")
    }
    pub fn state_file(&self) -> PathBuf {
        self.data.join("claude").join("shim-state.json")
    }
    pub fn chain_file(&self) -> PathBuf {
        self.data.join("claude").join("chain.json")
    }
    pub fn cli_cache(&self) -> PathBuf {
        self.data.join("claude").join("cli-usage.json")
    }
    pub fn oauth_cache(&self) -> PathBuf {
        self.data.join("claude").join("oauth-usage.json")
    }
    /// The stable copy of the shim that settings.json points at.
    pub fn shim_copy(&self) -> PathBuf {
        let name = if cfg!(windows) { "statusline-shim.exe" } else { "statusline-shim" };
        self.data.join("bin").join(name)
    }

    pub fn write(&self, path: &Path, bytes: impl AsRef<[u8]>) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    pub fn write_settings(&self, text: impl AsRef<[u8]>) {
        self.write(&self.settings_json(), text);
    }

    pub fn settings_bytes(&self) -> Vec<u8> {
        fs::read(self.settings_json()).unwrap()
    }

    pub fn settings_text(&self) -> String {
        String::from_utf8(self.settings_bytes()).unwrap()
    }

    pub fn write_rate_limits(&self, text: &str) {
        self.write(&self.rate_limits(), text);
    }

    /// Where the installed command in `settings_json` records to (`--out`), which is the file the
    /// collector reads for that settings file. Only for the home-relative `~/` form of `new()`.
    pub fn recording_target(&self, settings_json: &Path) -> PathBuf {
        let doc: serde_json::Value =
            serde_json::from_slice(&fs::read(settings_json).unwrap()).unwrap();
        let command = doc["statusLine"]["command"].as_str().unwrap();
        let out = command.split_whitespace().skip_while(|word| *word != "--out").nth(1).unwrap();
        self.home.join(out.strip_prefix("~/").unwrap())
    }

    pub fn write_credentials(&self, expires_at_ms: i64) {
        self.write(&self.credentials(), credentials_json(expires_at_ms));
    }

    /// A stand-in for the shim build the installer copies; any non-empty file will do.
    pub fn shim_source(&self) -> PathBuf {
        self.root.path().join("build").join("statusline-shim.exe")
    }

    /// Replaces the stand-in, as a new app version would.
    pub fn shim_source_with(&self, bytes: &[u8]) -> PathBuf {
        let path = self.shim_source();
        self.write(&path, bytes);
        path
    }
}

/// Every file below `dir`, relative and with `/` separators, sorted. Empty when `dir` is absent.
pub fn files_under(dir: &Path) -> Vec<String> {
    fn walk(dir: &Path, prefix: &str, out: &mut Vec<String>) {
        let Ok(entries) = fs::read_dir(dir) else { return };
        for entry in entries.map(|entry| entry.unwrap()) {
            let name = format!("{prefix}{}", entry.file_name().to_string_lossy());
            if entry.file_type().unwrap().is_dir() {
                walk(&entry.path(), &format!("{name}/"), out);
            } else {
                out.push(name);
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, "", &mut out);
    out.sort();
    out
}

pub fn notice_codes(view: &HarnessView) -> Vec<&str> {
    view.notices.iter().map(|notice| notice.code.as_str()).collect()
}

/// The view as the UI would receive it, for "no secret in here" assertions.
pub fn wire(view: &HarnessView) -> String {
    serde_json::to_string(view).unwrap()
}
