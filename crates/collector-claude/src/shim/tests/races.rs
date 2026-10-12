//! Edits that land between our read and our write must never be overwritten. The `Hooks` seam runs
//! a closure at that exact moment, once everything the settings file refers to is in place.

use std::cell::Cell;

use usage_model::{ErrorCode, ShimState};

use super::*;
use crate::shim::install::{Hooks, install_with};
use crate::shim::uninstall::uninstall_with;
use crate::testenv::settings_foreign;

fn install_racing(env: &Env, during: &dyn Fn()) -> Result<ShimStatus, HarnessError> {
    let hooks = Hooks { before_commit: during };
    install_with(&env.settings(), &env.ctx(), &env.shim_source(), &hooks)
}

fn uninstall_racing(env: &Env, during: &dyn Fn()) -> Result<ShimStatus, HarnessError> {
    uninstall_with(&env.settings(), &env.ctx(), &Hooks { before_commit: during })
}

fn assert_conflict(error: &HarnessError) {
    assert_eq!(
        (error.code, error.retryable, reason(error)),
        (ErrorCode::InstallFailed, true, "concurrent_modification")
    );
}

#[test]
fn an_edit_between_our_read_and_our_write_aborts_the_install_and_survives() {
    let env = Env::new();
    env.write_settings(settings_foreign());
    let edited = settings_foreign().replace("\"opus\"", "\"sonnet\"");

    let error = install_racing(&env, &|| env.write_settings(&edited)).unwrap_err();
    assert_conflict(&error);
    assert_eq!(env.settings_text(), edited, "the other writer's file is untouched");

    // Nothing is wedged: a retry builds on the new content.
    assert_eq!(install(&env).unwrap().state, ShimState::Installed);
    assert_eq!(json(&env)["model"], "sonnet");
}

#[test]
fn a_file_deleted_in_between_aborts_the_install() {
    let env = Env::new();
    env.write_settings(settings_foreign());
    let error =
        install_racing(&env, &|| std::fs::remove_file(env.settings_json()).unwrap()).unwrap_err();
    assert_conflict(&error);
    assert!(!env.settings_json().exists(), "not recreated behind the other writer's back");
}

#[test]
fn a_file_created_in_between_aborts_an_install_that_found_none() {
    let env = Env::new();
    let created = r#"{"model":"haiku"}"#;
    let error = install_racing(&env, &|| env.write_settings(created)).unwrap_err();
    assert_conflict(&error);
    assert_eq!(env.settings_text(), created);
}

#[test]
fn a_file_that_appears_in_between_and_cannot_be_read_is_a_conflict_too() {
    let env = Env::new();
    let too_big = vec![b' '; 2 * 1024 * 1024 + 1];
    let error = install_racing(&env, &|| env.write_settings(&too_big)).unwrap_err();
    assert_conflict(&error);
    assert_eq!(env.settings_bytes(), too_big, "what appeared is left alone");
}

#[test]
fn a_rewrite_with_identical_bytes_is_not_a_conflict() {
    let env = Env::new();
    env.write_settings(settings_foreign());
    let same = env.settings_bytes();
    let result = install_racing(&env, &|| env.write_settings(&same)).unwrap();
    assert_eq!(result.state, ShimState::Installed);
}

#[test]
fn the_hook_runs_once_per_operation_and_only_when_a_write_is_in_reach() {
    let env = Env::new();
    env.write_settings(settings_foreign());
    let calls = Cell::new(0);
    install_racing(&env, &|| calls.set(calls.get() + 1)).unwrap();
    assert_eq!(calls.get(), 1);
    uninstall_racing(&env, &|| calls.set(calls.get() + 1)).unwrap();
    assert_eq!(calls.get(), 2);

    // Early exits happen before the commit point.
    let bad = Env::new();
    bad.write_settings("{ not json");
    assert!(install_racing(&bad, &|| calls.set(99)).is_err());
    assert!(uninstall_racing(&bad, &|| calls.set(99)).is_err());
    let clean = Env::new();
    uninstall_racing(&clean, &|| calls.set(99)).unwrap();
    assert_eq!(calls.get(), 2);
}

#[test]
fn an_edit_during_uninstall_aborts_it_and_keeps_our_files() {
    let env = Env::new();
    env.write_settings(settings_foreign());
    install(&env).unwrap();
    let edited = env.settings_text().replace("\"opus\"", "\"haiku\"");

    let error = uninstall_racing(&env, &|| env.write_settings(&edited)).unwrap_err();
    assert_conflict(&error);
    assert_eq!(env.settings_text(), edited);
    assert!(env.shim_copy().is_file() && env.state_file().is_file() && env.chain_file().is_file());
    assert!(env.state_copy().is_file(), "the record beside settings.json stays too");

    // The retry restores the user's command and keeps the concurrent edit.
    uninstall(&env).unwrap();
    assert_eq!(json(&env)["model"], "haiku");
    assert_eq!(status_line(&env)["command"], "~/.claude/statusline.sh");
}

#[test]
fn a_file_deleted_during_uninstall_aborts_it() {
    let env = Env::new();
    install(&env).unwrap();
    let error =
        uninstall_racing(&env, &|| std::fs::remove_file(env.settings_json()).unwrap()).unwrap_err();
    assert_conflict(&error);
    assert!(env.shim_copy().is_file(), "our files stay until the settings are settled");
}
