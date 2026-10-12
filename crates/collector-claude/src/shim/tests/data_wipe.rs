//! The app data dir can vanish (a data-deleting uninstall of the app, manual cleanup) while
//! settings.json keeps naming our shim. What the user had must still come back from the copy of the
//! saved record that sits beside settings.json, and never from the backup, which holds the first
//! original and may be older.

use serde_json::json;
use usage_model::{Ctx, ShimState};

use super::*;
use crate::testenv::{NOW, settings_foreign, settings_plain};

fn wipe_data(env: &Env) {
    std::fs::remove_dir_all(&env.data).unwrap();
}

#[test]
fn reinstalling_after_a_wipe_keeps_the_users_command_chained() {
    let env = Env::new();
    env.write_settings(settings_foreign());
    install(&env).unwrap();
    let installed = env.settings_bytes();
    wipe_data(&env);
    assert_eq!(status(&env).state, ShimState::Outdated, "a dangling command is what a wipe leaves");

    let result = install(&env).unwrap();
    assert_eq!((result.state, result.chained), (ShimState::Installed, true));
    assert_eq!(env.settings_bytes(), installed, "the same command, --chain included");
    assert_eq!(read_json(&env.chain_file()), json!({"command": "~/.claude/statusline.sh"}));
}

#[test]
fn a_moved_data_dir_keeps_the_chain_too() {
    let env = Env::new();
    env.write_settings(settings_foreign());
    install(&env).unwrap();
    let moved = Ctx::new(NOW, env.root().join("nova pasta"), env.home.clone());

    let result = super::super::install_shim(&env.settings(), &moved, &env.shim_source()).unwrap();
    assert_eq!((result.state, result.chained), (ShimState::Installed, true));
    let chain = moved.data_dir.join("claude").join("chain.json");
    assert_eq!(read_json(&chain), json!({"command": "~/.claude/statusline.sh"}));
}

#[test]
fn uninstalling_after_a_wipe_restores_the_exact_original() {
    let env = Env::new();
    env.write_settings(settings_foreign());
    install(&env).unwrap();
    wipe_data(&env);

    let result = uninstall(&env).unwrap();
    assert_eq!(env.settings_text(), settings_foreign());
    assert_eq!(result.state, ShimState::Foreign);
}

#[test]
fn the_latest_original_comes_back_not_the_first_one_the_backup_holds() {
    let env = Env::new();
    env.write_settings(settings_foreign());
    install(&env).unwrap();
    let theirs = json!({"statusLine": {"type": "command", "command": "python ~/novo.py"}});
    env.write_settings(theirs.to_string());
    install(&env).unwrap();
    wipe_data(&env);

    uninstall(&env).unwrap();
    assert_eq!(json(&env), theirs);
    assert_eq!(std::fs::read_to_string(env.backup()).unwrap(), settings_foreign());
}

#[test]
fn with_no_record_anywhere_a_reinstall_does_not_dig_the_backup_up() {
    // The backup holds the first original, which may be older than what the user has set since.
    let env = Env::new();
    env.write_settings(settings_foreign());
    install(&env).unwrap();
    wipe_data(&env);
    assert!(env.state_copy().is_file(), "written by the install");
    std::fs::remove_file(env.state_copy()).unwrap();

    let result = install(&env).unwrap();
    assert_eq!((result.state, result.chained), (ShimState::Installed, false));
    assert!(!env.chain_file().exists());
    assert_eq!(std::fs::read_to_string(env.backup()).unwrap(), settings_foreign());
}

#[test]
fn a_wipe_after_an_install_with_no_status_line_still_ends_without_one() {
    let env = Env::new();
    env.write_settings(settings_plain());
    install(&env).unwrap();
    assert!(env.state_copy().is_file(), "the copy says there was nothing to bring back");
    wipe_data(&env);

    uninstall(&env).unwrap();
    assert_eq!(json(&env), parse(&settings_plain()));
}

#[test]
fn the_copy_sits_beside_settings_json_and_says_what_the_state_file_says() {
    let env = Env::new();
    env.write_settings(settings_foreign());
    install(&env).unwrap();

    assert!(env.state_copy().is_file(), "written by the install");
    assert_eq!(read_json(&env.state_copy()), read_json(&env.state_file()));
    assert_eq!(
        files_under(&env.config_dir()),
        ["settings.json", "settings.json.aism-backup", "settings.json.aism-state"],
        "no temp file stays behind"
    );
}

#[test]
fn a_lost_or_unreadable_state_file_is_rebuilt_from_the_copy_by_the_next_install() {
    for damage in [None, Some("not json"), Some(r#"{"v":2,"hadStatusLine":true,"statusLine":1}"#)] {
        let env = Env::new();
        env.write_settings(settings_foreign());
        install(&env).unwrap();
        let (state, chain) = (read_json(&env.state_file()), read_json(&env.chain_file()));
        match damage {
            Some(text) => std::fs::write(env.state_file(), text).unwrap(),
            None => std::fs::remove_file(env.state_file()).unwrap(),
        }
        std::fs::remove_file(env.chain_file()).unwrap();

        let result = install(&env).unwrap();
        assert_eq!((result.state, result.chained), (ShimState::Installed, true), "{damage:?}");
        assert_eq!(read_json(&env.state_file()), state, "{damage:?}");
        assert_eq!(read_json(&env.chain_file()), chain, "{damage:?}");
    }
}

#[test]
fn a_lost_copy_is_rebuilt_from_the_state_file_by_the_next_install() {
    let env = Env::new();
    env.write_settings(settings_foreign());
    install(&env).unwrap();
    assert!(env.state_copy().is_file(), "written by the install");
    std::fs::remove_file(env.state_copy()).unwrap();

    install(&env).unwrap();
    assert_eq!(read_json(&env.state_copy()), read_json(&env.state_file()));
}
