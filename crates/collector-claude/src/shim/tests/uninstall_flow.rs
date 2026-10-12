//! `uninstall_shim`: the user's `statusLine` comes back exactly as it was, and only our files go.

use usage_model::settings::ClaudeSettings;
use usage_model::{ErrorCode, ShimState};

use super::*;
use crate::testenv::{settings_foreign, settings_plain, settings_static};

#[test]
fn the_original_file_comes_back_byte_for_byte() {
    let null_line = "{\n  \"a\": 1,\n  \"statusLine\": null,\n  \"z\": 2\n}\n".to_owned();
    let string_line = "{\n  \"statusLine\": \"echo oi\"\n}\n".to_owned();
    let cases = [
        (settings_foreign(), ShimState::Foreign),
        (settings_static(), ShimState::Foreign),
        (settings_plain(), ShimState::NotInstalled),
        (null_line, ShimState::NotInstalled),
        (string_line, ShimState::Foreign),
    ];
    for (original, restored_state) in cases {
        let env = Env::new();
        env.write_settings(&original);
        install(&env).unwrap();
        assert_ne!(env.settings_text(), original, "the install changed something");

        let result = uninstall(&env).unwrap();
        assert_eq!(env.settings_text(), original);
        assert_eq!(result.state, restored_state, "{original}");
    }
}

#[test]
fn a_file_that_did_not_exist_is_left_as_an_empty_object() {
    let env = Env::new();
    install(&env).unwrap();
    assert_eq!(uninstall(&env).unwrap().state, ShimState::NotInstalled);
    assert_eq!(env.settings_text(), "{}\n");
}

#[test]
fn our_files_go_and_the_backup_of_the_users_file_stays() {
    let env = Env::new();
    env.write_settings(settings_foreign());
    install(&env).unwrap();
    env.write(&env.rate_limits(), "{}");
    env.write(&env.oauth_cache(), "{}");
    env.write(&env.cli_cache(), "{}");
    assert!(env.state_copy().is_file(), "the record is also kept beside settings.json");

    uninstall(&env).unwrap();
    assert_eq!(files_under(&env.data), Vec::<String>::new(), "nothing of ours is left");
    assert_eq!(std::fs::read_dir(&env.data).unwrap().count(), 0, "not even a directory");
    assert_eq!(
        files_under(&env.config_dir()),
        ["settings.json", "settings.json.aism-backup"],
        "the copy of the record goes with the rest, the user's backup stays"
    );
    assert_eq!(std::fs::read_to_string(env.backup()).unwrap(), settings_foreign());
}

#[test]
fn nothing_installed_means_nothing_to_do() {
    let env = Env::new();
    env.write_settings(settings_plain());
    let before = snapshot(&env);
    assert_eq!(uninstall(&env).unwrap().state, ShimState::NotInstalled);
    assert_unchanged(&env, &before, "uninstalling nothing");

    let empty = Env::new();
    assert_eq!(uninstall(&empty).unwrap().state, ShimState::NotInstalled);
    assert!(!empty.config_dir().exists(), "no settings file is created to say nothing");
}

#[test]
fn uninstalling_twice_is_fine() {
    let env = Env::new();
    env.write_settings(settings_foreign());
    install(&env).unwrap();
    uninstall(&env).unwrap();
    let after_first = snapshot(&env);
    assert_eq!(uninstall(&env).unwrap().state, ShimState::Foreign);
    assert_unchanged(&env, &after_first, "a second uninstall");
}

#[test]
fn somebody_elses_command_is_never_removed() {
    let env = Env::new();
    env.write_settings(settings_foreign());
    install(&env).unwrap();
    let theirs = r#"{"statusLine":{"type":"command","command":"node ~/bar.js"}}"#;
    env.write_settings(theirs);
    let before = snapshot(&env);

    let error = uninstall(&env).unwrap_err();
    assert_eq!(
        (error.code, error.retryable, reason(&error)),
        (ErrorCode::InstallFailed, false, "foreign_status_line")
    );
    assert_unchanged(&env, &before, "neither their file nor ours");
}

#[test]
fn the_copy_of_the_record_counts_as_ours_when_a_foreign_command_is_found() {
    // The data dir is gone and the user put their own command back by hand: what is left beside
    // settings.json is ours like any other file, so nothing is guessed about their command.
    let env = Env::new();
    env.write_settings(settings_foreign());
    install(&env).unwrap();
    std::fs::remove_dir_all(&env.data).unwrap();
    env.write_settings(r#"{"statusLine":{"type":"command","command":"node ~/bar.js"}}"#);
    let before = snapshot(&env);

    let error = uninstall(&env).unwrap_err();
    assert_eq!((error.code, reason(&error)), (ErrorCode::InstallFailed, "foreign_status_line"));
    assert_unchanged(&env, &before, "neither their file nor the copy");
}

#[test]
fn without_the_state_file_but_with_its_copy_the_original_comes_back() {
    let env = Env::new();
    env.write_settings(settings_foreign());
    install(&env).unwrap();
    assert!(env.state_copy().is_file(), "written by the install");
    std::fs::remove_file(env.state_file()).unwrap();

    let result = uninstall(&env).unwrap();
    assert_eq!(env.settings_text(), settings_foreign());
    assert_eq!(result.state, ShimState::Foreign);
}

#[test]
fn without_any_saved_state_the_key_is_removed_and_the_backup_still_has_the_original() {
    let env = Env::new();
    env.write_settings(settings_foreign());
    install(&env).unwrap();
    for record in [env.state_file(), env.state_copy()] {
        if record.exists() {
            std::fs::remove_file(record).unwrap();
        }
    }

    uninstall(&env).unwrap();
    assert!(
        json(&env).get("statusLine").is_none(),
        "a dangling shim command would only break the status bar"
    );
    assert_eq!(keys(&json(&env)), ["model", "env", "permissions"]);
    assert_eq!(std::fs::read_to_string(env.backup()).unwrap(), settings_foreign());
}

#[test]
fn an_outdated_install_can_still_be_removed() {
    let env = Env::new();
    env.write_settings(settings_foreign());
    install(&env).unwrap();
    std::fs::remove_file(env.shim_copy()).unwrap();
    uninstall(&env).unwrap();
    assert_eq!(env.settings_text(), settings_foreign());
}

#[test]
fn leaving_needs_no_consent() {
    let env = Env::new();
    env.write_settings(settings_foreign());
    install(&env).unwrap();
    let revoked = ClaudeSettings { shim_consent: false, ..env.settings() };
    super::super::uninstall_shim(&revoked, &env.ctx()).unwrap();
    assert_eq!(env.settings_text(), settings_foreign());
}

#[test]
fn install_uninstall_install_ends_where_the_first_install_did() {
    let env = Env::new();
    env.write_settings(settings_foreign());
    install(&env).unwrap();
    let first = env.settings_bytes();
    uninstall(&env).unwrap();
    install(&env).unwrap();
    assert_eq!(env.settings_bytes(), first);
    assert_eq!(
        std::fs::read_to_string(env.backup()).unwrap(),
        settings_foreign(),
        "the backup is the first original"
    );
}

#[test]
fn a_settings_file_that_vanished_only_costs_us_our_files() {
    let env = Env::new();
    install(&env).unwrap();
    std::fs::remove_file(env.settings_json()).unwrap();
    assert_eq!(uninstall(&env).unwrap().state, ShimState::NotInstalled);
    assert!(!env.settings_json().exists() && files_under(&env.data).is_empty());
}

#[test]
fn an_unusable_config_dir_is_rejected_by_name() {
    let env = Env::new();
    let relative = ClaudeSettings { config_dir: Some("relative/dir".into()), ..env.settings() };
    let error = super::super::uninstall_shim(&relative, &env.ctx()).unwrap_err();
    assert_eq!(
        (error.code, error.params["field"].as_str()),
        (ErrorCode::InvalidInput, "configDir")
    );
}
