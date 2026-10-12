use std::path::{Path, PathBuf};

use app_core::MemorySecretStore;
use app_core::secrets::store_error;
use usage_model::ViewState;

use super::*;

const NOW: i64 = 1_800_000_000;

/// A secret store whose every call fails, like a locked Credential Manager.
struct BrokenStore;

impl SecretStore for BrokenStore {
    fn get(&self, _: SecretKind) -> Result<Option<usage_support::Secret>, HarnessError> {
        Err(store_error())
    }

    fn set(&self, _: SecretKind, _: &str) -> Result<(), HarnessError> {
        Err(store_error())
    }

    fn delete(&self, _: SecretKind) -> Result<(), HarnessError> {
        Err(store_error())
    }
}

fn temp_ctx() -> (tempfile::TempDir, Ctx) {
    let dir = tempfile::tempdir().expect("tempdir");
    let ctx = Ctx::new(NOW, dir.path(), dir.path());
    (dir, ctx)
}

#[tokio::test]
async fn each_collector_reads_its_own_harness() {
    let (_dir, ctx) = temp_ctx();
    let settings = Settings::default();
    let secrets = MemorySecretStore::new();
    for collector in adapters(None).collectors {
        let view = collector.collect(&settings, &secrets, &ctx).await;
        assert_eq!(view.id, collector.id());
    }
}

#[tokio::test]
async fn a_failing_secret_store_becomes_that_tabs_error_for_the_keyed_harnesses() {
    let (_dir, ctx) = temp_ctx();
    let settings = Settings::default();
    for collector in adapters(None).collectors {
        let view = collector.collect(&settings, &BrokenStore, &ctx).await;
        let id = collector.id();
        let keyed = matches!(
            id,
            HarnessId::Opencode
                | HarnessId::Kimi
                | HarnessId::Zai
                | HarnessId::Openrouter
                | HarnessId::Deepseek
                | HarnessId::Litellm
        );
        if keyed {
            assert_eq!(view.state, ViewState::Error, "{id:?}");
            assert_eq!(view.error, Some(store_error()), "{id:?}");
        } else {
            assert_ne!(view.state, ViewState::Error, "{id:?}");
        }
    }
}

fn claude_settings(config_dir: Option<&str>) -> ClaudeSettings {
    ClaudeSettings {
        enabled: false,
        shim_consent: true,
        oauth_usage: true,
        config_dir: config_dir.map(str::to_owned),
    }
}

#[test]
fn a_blank_setting_takes_the_folder_from_the_environment() {
    for blank in [None, Some(""), Some("   ")] {
        let resolved = with_env_dir(&claude_settings(blank), Some("D:\\perfis\\claude"));
        assert_eq!(resolved.config_dir.as_deref(), Some("D:\\perfis\\claude"), "{blank:?}");
    }
}

#[test]
fn a_folder_the_user_typed_wins_over_the_environment() {
    let typed = claude_settings(Some("E:\\meu\\claude"));
    let resolved = with_env_dir(&typed, Some("D:\\perfis\\claude"));
    assert_eq!(resolved.config_dir.as_deref(), Some("E:\\meu\\claude"));
}

#[test]
fn without_an_environment_folder_the_setting_is_left_alone() {
    assert_eq!(with_env_dir(&claude_settings(None), None), claude_settings(None));
    assert_eq!(with_env_dir(&claude_settings(Some("  ")), None), claude_settings(Some("  ")));
}

#[test]
fn only_the_folder_changes() {
    let resolved = with_env_dir(&claude_settings(None), Some("D:\\perfis\\claude"));
    let expected = ClaudeSettings {
        config_dir: Some("D:\\perfis\\claude".to_owned()),
        ..claude_settings(None)
    };
    assert_eq!(resolved, expected);
}

/// Where the shim looks for Claude Code's settings.json, given the environment folder and the
/// folder typed in the settings.
fn settings_json_seen(env_dir: Option<&Path>, typed: Option<&Path>, ctx: &Ctx) -> Option<PathBuf> {
    let shim = adapters(env_dir.map(|dir| dir.display().to_string())).claude_shim;
    let settings = ClaudeSettings {
        config_dir: typed.map(|dir| dir.display().to_string()),
        ..ClaudeSettings::default()
    };
    shim.status(&settings, ctx).settings_path.map(PathBuf::from)
}

#[test]
fn the_shim_follows_claude_config_dir_when_the_setting_is_blank() {
    let (dir, ctx) = temp_ctx();
    let env_dir = dir.path().join("perfil-env");
    let seen = settings_json_seen(Some(&env_dir), None, &ctx);
    assert_eq!(seen, Some(env_dir.join("settings.json")));
}

#[test]
fn a_folder_typed_in_the_settings_beats_the_environment_one() {
    let (dir, ctx) = temp_ctx();
    let (env_dir, typed) = (dir.path().join("perfil-env"), dir.path().join("digitada"));
    let seen = settings_json_seen(Some(&env_dir), Some(&typed), &ctx);
    assert_eq!(seen, Some(typed.join("settings.json")));
}

#[test]
fn without_either_folder_the_shim_uses_dot_claude_under_home() {
    let (_dir, ctx) = temp_ctx();
    let seen = settings_json_seen(None, None, &ctx);
    assert_eq!(seen, Some(ctx.home_dir.join(".claude").join("settings.json")));
}

#[test]
fn install_and_uninstall_edit_the_folder_the_environment_names() {
    let (dir, ctx) = temp_ctx();
    let env_dir = dir.path().join("perfil-env");
    std::fs::create_dir_all(&env_dir).expect("mkdir");
    let settings_json = env_dir.join("settings.json");
    let before = br#"{"theme":"dark"}"#;
    std::fs::write(&settings_json, before).expect("write settings");
    let exe = dir.path().join("fabricated-shim.exe");
    std::fs::write(&exe, b"fabricated shim bytes").expect("write exe");

    let shim = adapters(Some(env_dir.display().to_string())).claude_shim;
    let settings = ClaudeSettings { shim_consent: true, ..ClaudeSettings::default() };

    let installed = shim.install(&settings, &ctx, &exe).expect("install");
    assert_eq!(installed.settings_path.map(PathBuf::from), Some(settings_json.clone()));
    let text = std::fs::read_to_string(&settings_json).expect("read settings");
    assert!(text.contains("statusLine"), "{text}");
    assert!(!ctx.home_dir.join(".claude").exists(), "the default folder was not touched");

    shim.uninstall(&settings, &ctx).expect("uninstall");
    let restored: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&settings_json).expect("read")).expect("json");
    assert_eq!(restored, serde_json::json!({ "theme": "dark" }));
}

/// Every file below `dir`, so a test need not know where the installer keeps its copy.
fn files_below(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(next) = pending.pop() {
        for entry in std::fs::read_dir(&next).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() { pending.push(path) } else { found.push(path) }
        }
    }
    found
}

#[test]
fn refresh_replaces_the_copy_in_the_folder_the_environment_names_and_nothing_else() {
    let (dir, ctx) = temp_ctx();
    let env_dir = dir.path().join("perfil-env");
    std::fs::create_dir_all(&env_dir).expect("mkdir");
    let settings_json = env_dir.join("settings.json");
    std::fs::write(&settings_json, br#"{"theme":"dark"}"#).expect("write settings");
    let exe = dir.path().join("fabricated-shim.exe");
    std::fs::write(&exe, b"fabricated shim build 1").expect("write exe");
    let shim = adapters(Some(env_dir.display().to_string())).claude_shim;
    let settings = ClaudeSettings { shim_consent: true, ..ClaudeSettings::default() };
    shim.install(&settings, &ctx, &exe).expect("install");
    let installed = std::fs::read(&settings_json).expect("read settings");

    std::fs::write(&exe, b"fabricated shim build 2").expect("write newer exe");
    shim.refresh_copy(&settings, &ctx, &exe).expect("refresh");

    let copies: Vec<_> = files_below(&ctx.data_dir.join("bin"))
        .iter()
        .map(|copy| std::fs::read_to_string(copy).expect("read copy"))
        .collect();
    assert_eq!(copies, ["fabricated shim build 2"]);
    assert_eq!(std::fs::read(&settings_json).expect("read"), installed);
    assert!(!ctx.home_dir.join(".claude").exists(), "the default folder was not touched");
}
