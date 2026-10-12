//! `install_shim`: what lands where, from every kind of starting point.

use serde_json::json;
use usage_model::settings::ClaudeSettings;
use usage_model::{Ctx, ErrorCode, ShimState};

use super::*;
use crate::testenv::{NOW, settings_foreign, settings_plain, settings_static};

#[test]
fn over_a_missing_file_it_creates_a_settings_file_holding_only_the_status_line() {
    let env = Env::new();
    let result = install(&env).unwrap();
    assert_eq!((result.state, result.chained), (ShimState::Installed, false));
    assert_eq!(json(&env), json!({"statusLine": {"type": "command", "command": command(false)}}));
    assert_eq!(
        env.settings_text(),
        format!(
            "{{\n  \"statusLine\": {{\n    \"type\": \"command\",\n    \"command\": \"{}\"\n  }}\n}}\n",
            command(false)
        )
    );
    assert!(!env.backup().exists() && !env.chain_file().exists(), "nothing to back up or chain");
    assert_eq!(std::fs::read(env.shim_copy()).unwrap(), std::fs::read(env.shim_source()).unwrap());
    let saved = read_json(&env.state_file());
    assert_eq!(
        (saved["hadStatusLine"].as_bool(), saved["statusLine"].is_null()),
        (Some(false), true)
    );
}

#[test]
fn over_settings_without_a_status_line_the_key_goes_last_and_the_rest_is_kept() {
    let env = Env::new();
    env.write_settings(settings_plain());
    install(&env).unwrap();

    let (before, after) = (parse(&settings_plain()), json(&env));
    let mut expected_order = keys(&before);
    expected_order.push("statusLine".to_owned());
    assert_eq!(keys(&after), expected_order, "order, with the new key last");
    for key in keys(&before) {
        assert_eq!(after[&key], before[&key], "{key}");
    }
    assert_eq!(
        std::fs::read_to_string(env.backup()).unwrap(),
        settings_plain(),
        "the original is kept next to it"
    );
}

#[test]
fn over_a_foreign_command_it_chains_it_and_keeps_the_other_status_line_keys() {
    let env = Env::new();
    env.write_settings(settings_foreign());
    let result = install(&env).unwrap();
    assert_eq!((result.state, result.chained), (ShimState::Installed, true));

    let line = status_line(&env);
    assert_eq!(line, json!({"type": "command", "command": command(true), "padding": 2}));
    assert_eq!(keys(&line), ["type", "command", "padding"], "the object keeps its own order");
    assert_eq!(read_json(&env.chain_file()), json!({"command": "~/.claude/statusline.sh"}));
    let saved = read_json(&env.state_file());
    assert_eq!(
        saved["statusLine"],
        json!({"type": "command", "command": "~/.claude/statusline.sh", "padding": 2})
    );
    assert_eq!(std::fs::read_to_string(env.backup()).unwrap(), settings_foreign());

    let (before, after) = (parse(&settings_foreign()), json(&env));
    assert_eq!(keys(&after), keys(&before), "statusLine stays where it was");
    for key in ["model", "env", "permissions"] {
        assert_eq!(after[key], before[key], "{key}");
    }
    assert!(env.settings_text().contains("ação ✓ 日本"), "unicode is written as is, not escaped");
}

#[test]
fn a_status_line_that_is_not_a_command_is_replaced_without_a_chain_and_saved_whole() {
    let env = Env::new();
    env.write_settings(settings_static());
    let result = install(&env).unwrap();
    assert_eq!((result.state, result.chained), (ShimState::Installed, false));
    assert_eq!(status_line(&env), json!({"type": "command", "command": command(false)}));
    assert!(!env.chain_file().exists());
    assert_eq!(
        read_json(&env.state_file())["statusLine"],
        json!({"type": "static", "text": "olá"})
    );
    assert_eq!(keys(&json(&env)), ["alfa", "statusLine", "omega"]);
}

#[test]
fn installing_twice_changes_nothing_and_never_nests_the_shim() {
    let env = Env::new();
    env.write_settings(settings_foreign());
    install(&env).unwrap();
    let after_first = snapshot(&env);
    let modified = std::fs::metadata(env.settings_json()).unwrap().modified().unwrap();

    let again = install(&env).unwrap();
    assert_eq!((again.state, again.chained), (ShimState::Installed, true));
    assert_unchanged(&env, &after_first, "a second install");
    assert_eq!(
        std::fs::metadata(env.settings_json()).unwrap().modified().unwrap(),
        modified,
        "not even rewritten"
    );
    assert_eq!(command_of(&env).matches("statusline-shim").count(), 1);
    assert_eq!(
        read_json(&env.chain_file()),
        json!({"command": "~/.claude/statusline.sh"}),
        "the chain is still the user's"
    );
}

#[test]
fn an_outdated_install_is_repaired_in_place() {
    let env = Env::new();
    env.write_settings(settings_foreign());
    install(&env).unwrap();
    let settings_before = env.settings_bytes();
    std::fs::remove_file(env.shim_copy()).unwrap();
    assert_eq!(status(&env).state, ShimState::Outdated);

    assert_eq!(install(&env).unwrap().state, ShimState::Installed);
    assert_eq!(env.settings_bytes(), settings_before, "the command was already right");
    assert!(env.shim_copy().is_file());
}

#[test]
fn a_new_shim_build_replaces_the_copy_and_nothing_else() {
    let env = Env::new();
    env.write_settings(settings_foreign());
    install(&env).unwrap();
    let settings_before = env.settings_bytes();

    let newer = env.shim_source_with(b"fabricated shim build 2\n");
    super::super::install_shim(&env.settings(), &env.ctx(), &newer).unwrap();
    assert_eq!(std::fs::read(env.shim_copy()).unwrap(), b"fabricated shim build 2\n");
    assert_eq!(env.settings_bytes(), settings_before);
}

#[test]
fn a_moved_data_dir_is_outdated_until_the_shim_is_installed_there() {
    let env = Env::new();
    install(&env).unwrap();
    let moved = Ctx::new(NOW, env.root().join("nova pasta"), env.home.clone());
    assert_eq!(super::super::shim_status(&env.settings(), &moved).state, ShimState::Outdated);

    let result = super::super::install_shim(&env.settings(), &moved, &env.shim_source()).unwrap();
    assert_eq!(result.state, ShimState::Installed);
    assert!(command_of(&env).contains("nova pasta"), "{}", command_of(&env));
    assert!(moved.data_dir.join("bin").join(EXE_NAME).is_file());
}

#[test]
fn a_command_the_user_put_in_place_of_ours_is_wrapped_and_later_restored() {
    let env = Env::new();
    env.write_settings(settings_foreign());
    install(&env).unwrap();
    let theirs = json!({"statusLine": {"type": "command", "command": "python ~/novo.py"}});
    env.write_settings(theirs.to_string());

    let result = install(&env).unwrap();
    assert_eq!((result.state, result.chained), (ShimState::Installed, true));
    assert_eq!(read_json(&env.chain_file()), json!({"command": "python ~/novo.py"}));
    uninstall(&env).unwrap();
    assert_eq!(json(&env), theirs, "the latest command is what comes back");
}

#[test]
fn odd_status_lines_are_saved_verbatim_and_never_chained() {
    for original in
        [json!(null), json!("echo oi"), json!(7), json!(["a"]), json!({"command": 5, "x": 1})]
    {
        let env = Env::new();
        env.write_settings(json!({"a": 1, "statusLine": original}).to_string());
        let result = install(&env).unwrap();
        assert_eq!((result.state, result.chained), (ShimState::Installed, false), "{original}");
        assert_eq!(read_json(&env.state_file())["statusLine"], original);
    }
}

#[test]
fn without_consent_nothing_is_touched() {
    let env = Env::new();
    env.write_settings(settings_foreign());
    let source = env.shim_source();
    let before = snapshot(&env);
    let settings = ClaudeSettings { shim_consent: false, ..env.settings() };
    let error = super::super::install_shim(&settings, &env.ctx(), &source).unwrap_err();
    assert_eq!(
        (error.code, error.retryable, error.params["field"].as_str()),
        (ErrorCode::InvalidInput, false, "shimConsent")
    );
    assert_unchanged(&env, &before, "install without consent");
}

#[test]
fn a_missing_or_empty_shim_build_is_reported_and_nothing_is_written() {
    let env = Env::new();
    env.write_settings(settings_foreign());
    let empty = env.shim_source_with(b"");
    let missing = env.root().join("build").join("nao-existe.exe");
    let before = snapshot(&env);
    for source in [&empty, &missing] {
        let error = super::super::install_shim(&env.settings(), &env.ctx(), source).unwrap_err();
        assert_eq!(
            (error.code, error.retryable, reason(&error)),
            (ErrorCode::InstallFailed, false, "shim_missing")
        );
    }
    assert_unchanged(&env, &before, "install without a shim build");
}

#[test]
fn an_unusable_config_dir_is_rejected_by_name() {
    let env = Env::new();
    let relative = ClaudeSettings { config_dir: Some("relative/dir".into()), ..env.settings() };
    let error = super::super::install_shim(&relative, &env.ctx(), &env.shim_source()).unwrap_err();
    assert_eq!(
        (error.code, error.params["field"].as_str()),
        (ErrorCode::InvalidInput, "configDir")
    );
    assert!(!env.data.exists());
}

#[test]
fn a_relative_data_or_home_dir_is_rejected_before_anything_is_written() {
    let env = Env::new();
    env.write_settings(settings_foreign());
    let before = snapshot(&env);
    for ctx in [
        Ctx::new(NOW, "relative/data", env.home.clone()),
        Ctx::new(NOW, env.data.clone(), "relative/home"),
    ] {
        let error = super::super::install_shim(&env.settings(), &ctx, &env.shim_source());
        let error = error.unwrap_err();
        assert_eq!((error.code, error.retryable), (ErrorCode::InvalidInput, false));
    }
    assert_unchanged(&env, &before, "install with a relative directory");
}

#[test]
fn the_chain_limit_counts_bytes_because_the_shim_reads_bytes() {
    let env = Env::new();
    // 12,000 characters but 36,000 bytes: more than the shim reads back from the chain file, so
    // the user's own status line would silently vanish.
    let wide = json!({"statusLine": {"type": "command", "command": "€".repeat(12_000)}});
    env.write_settings(wide.to_string());
    let before = snapshot(&env);
    let error = install(&env).unwrap_err();
    assert_eq!((error.code, reason(&error)), (ErrorCode::InstallFailed, "status_line_too_large"));
    assert_unchanged(&env, &before, "install over a command the shim would not run");
}

#[test]
fn the_chain_limit_counts_the_escaped_file_because_the_shim_reads_that() {
    let env = Env::new();
    // 11,000 bytes of control characters, but six bytes each once escaped in chain.json: the shim
    // reads a 64 KiB prefix, so the JSON would be cut off and the user's status line would vanish.
    let escaped = json!({"statusLine": {"type": "command", "command": "\u{1}".repeat(11_000)}});
    env.write_settings(escaped.to_string());
    let before = snapshot(&env);
    let error = install(&env).unwrap_err();
    assert_eq!((error.code, reason(&error)), (ErrorCode::InstallFailed, "status_line_too_large"));
    assert_unchanged(&env, &before, "install over a command whose chain file is too big to read");
}

#[test]
fn a_chained_command_over_the_limit_is_refused_before_anything_is_written() {
    let env = Env::new();
    let huge =
        |n: usize| json!({"statusLine": {"type": "command", "command": "x".repeat(n)}}).to_string();
    env.write_settings(huge(30_001));
    let before = snapshot(&env);
    let error = install(&env).unwrap_err();
    assert_eq!((error.code, reason(&error)), (ErrorCode::InstallFailed, "status_line_too_large"));
    assert_unchanged(&env, &before, "install over a huge status line");

    env.write_settings(huge(30_000));
    assert_eq!(install(&env).unwrap().state, ShimState::Installed, "the limit itself is fine");
}
