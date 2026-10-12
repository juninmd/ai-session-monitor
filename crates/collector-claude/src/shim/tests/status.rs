//! `shim_status`: every state, and the promise that looking never changes anything.

use serde_json::json;
use usage_model::settings::ClaudeSettings;
use usage_model::{Ctx, ShimState, ShimStatus};

use super::*;
use crate::testenv::{NOW, settings_foreign, settings_plain, settings_static};

fn expected(env: &Env, state: ShimState, chained: bool) -> ShimStatus {
    ShimStatus { state, chained, settings_path: Some(env.settings_json().display().to_string()) }
}

#[test]
fn no_settings_file_is_not_installed_and_looking_creates_nothing() {
    let env = Env::new();
    assert_eq!(status(&env), expected(&env, ShimState::NotInstalled, false));
    assert!(!env.config_dir().exists() && !env.data.exists());
}

#[test]
fn settings_without_a_status_line_are_not_installed() {
    for text in [settings_plain(), "{}".to_owned(), r#"{"statusLine": null}"#.to_owned()] {
        let env = Env::new();
        env.write_settings(&text);
        assert_eq!(status(&env), expected(&env, ShimState::NotInstalled, false), "{text}");
    }
}

#[test]
fn somebody_elses_status_line_is_foreign_whatever_its_shape() {
    let shapes = [
        json!({"type": "command", "command": "~/.claude/statusline.sh"}),
        json!({"type": "static", "text": "olá"}),
        json!({"type": "command"}),
        json!({"command": 7}),
        json!("echo oi"),
        json!(5),
        json!([]),
        // Mentions the shim but is a pipeline, which is somebody's own script.
        json!({"type": "command", "command": "statusline-shim --out x.json | tee log"}),
    ];
    for shape in shapes {
        let env = Env::new();
        env.write_settings(json!({"statusLine": shape}).to_string());
        assert_eq!(status(&env), expected(&env, ShimState::Foreign, false), "{shape}");
    }
    let env = Env::new();
    env.write_settings(settings_foreign());
    assert_eq!(status(&env).state, ShimState::Foreign);
    env.write_settings(settings_static());
    assert_eq!(status(&env).state, ShimState::Foreign);
}

#[test]
fn an_installed_shim_reports_whether_it_chains() {
    let env = Env::new();
    install(&env).unwrap();
    assert_eq!(status(&env), expected(&env, ShimState::Installed, false));

    let over_foreign = Env::new();
    over_foreign.write_settings(settings_foreign());
    install(&over_foreign).unwrap();
    assert_eq!(status(&over_foreign), expected(&over_foreign, ShimState::Installed, true));
}

#[test]
fn a_chain_file_that_went_missing_is_reported_as_not_chained() {
    let env = Env::new();
    env.write_settings(settings_foreign());
    install(&env).unwrap();
    std::fs::remove_file(env.chain_file()).unwrap();
    assert_eq!(status(&env), expected(&env, ShimState::Installed, false));
}

#[test]
fn a_missing_executable_makes_it_outdated() {
    let env = Env::new();
    env.write_settings(settings_foreign());
    install(&env).unwrap();
    std::fs::remove_file(env.shim_copy()).unwrap();
    assert_eq!(status(&env), expected(&env, ShimState::Outdated, true));
}

#[test]
fn a_command_that_points_elsewhere_is_outdated() {
    let env = Env::new();
    install(&env).unwrap();

    let moved = Ctx::new(NOW, env.root().join("nova pasta"), env.home.clone());
    let from_new_place = super::super::shim_status(&env.settings(), &moved);
    assert_eq!(from_new_place.state, ShimState::Outdated, "the data dir moved");

    let other_out = command(false).replace("claude/rate_limits.json", "claude/outro.json");
    env.write_settings(
        json!({"statusLine": {"type": "command", "command": other_out}}).to_string(),
    );
    assert_eq!(
        status(&env).state,
        ShimState::Outdated,
        "recording somewhere the app does not read"
    );
}

#[test]
fn settings_that_are_not_json_are_unreadable() {
    let env = Env::new();
    env.write_settings("{ not json");
    assert_eq!(status(&env), expected(&env, ShimState::Unreadable, false));
}

#[test]
fn an_unusable_config_dir_is_unreadable_and_has_no_path_to_show() {
    let env = Env::new();
    let relative = ClaudeSettings { config_dir: Some("relative/dir".into()), ..env.settings() };
    let status = super::super::shim_status(&relative, &env.ctx());
    assert_eq!(
        status,
        ShimStatus { state: ShimState::Unreadable, chained: false, settings_path: None }
    );
}

#[test]
fn the_config_dir_override_decides_which_file_is_looked_at() {
    let env = Env::new();
    let other = env.root().join("Configuração do Claude");
    env.write(&other.join("settings.json"), settings_foreign());
    let settings =
        ClaudeSettings { config_dir: Some(other.display().to_string()), ..env.settings() };
    let found = super::super::shim_status(&settings, &env.ctx());
    assert_eq!(found.state, ShimState::Foreign);
    assert_eq!(found.settings_path, Some(other.join("settings.json").display().to_string()));
    assert_eq!(status(&env).state, ShimState::NotInstalled, "the default dir has nothing");
}

#[test]
fn looking_never_writes() {
    let env = Env::new();
    env.write_settings(settings_foreign());
    install(&env).unwrap();
    let before = snapshot(&env);
    for _ in 0..3 {
        status(&env);
    }
    assert_unchanged(&env, &before, "looking");
}
