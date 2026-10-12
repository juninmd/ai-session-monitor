//! `refresh_shim_copy`: an app update's new shim build reaches the installed copy at startup, and
//! nothing else on disk moves (settings.json, the saved state, the chain file, the recording).

use std::path::{Path, PathBuf};
use std::time::{Duration, UNIX_EPOCH};

use serde_json::json;
use usage_model::settings::ClaudeSettings;
use usage_model::{ErrorCode, ShimState};

use super::*;
use crate::paths::Paths;
use crate::testenv::{settings_foreign, settings_plain};

/// What `Env::new` stands in for the bundled build; the copy `install` makes holds the same.
const BUILD_1: &str = "fabricated shim build 1\n";
const BUILD_2: &str = "fabricated shim build 2\n";

fn refresh(env: &Env, build: &Path) -> Result<(), HarnessError> {
    super::super::refresh_shim_copy(&env.settings(), &env.ctx(), build)
}

/// A new app version's bundled build.
fn newer(env: &Env) -> PathBuf {
    env.shim_source_with(BUILD_2.as_bytes())
}

fn text(file: &Path) -> String {
    String::from_utf8(std::fs::read(file).unwrap()).unwrap()
}

/// The files whose bytes differ between two snapshots, created and removed ones included.
fn differing(before: &Snapshot, after: &Snapshot) -> Vec<String> {
    let names: BTreeSet<&String> = before.keys().chain(after.keys()).collect();
    names.into_iter().filter(|name| before.get(*name) != after.get(*name)).cloned().collect()
}

/// `file` the way `snapshot` names it: relative to the temp root, with `/` separators.
fn named(env: &Env, file: &Path) -> String {
    file.strip_prefix(env.root()).unwrap().to_string_lossy().replace('\\', "/")
}

/// Installed over the user's own chained command: state, chain and backup files all exist.
fn installed_over_foreign(env: &Env) {
    env.write_settings(settings_foreign());
    install(env).unwrap();
}

#[test]
fn a_new_build_replaces_the_copy_and_nothing_else() {
    let env = Env::new();
    installed_over_foreign(&env);
    let build = newer(&env);
    let before = snapshot(&env);

    refresh(&env, &build).unwrap();

    assert_eq!(text(&env.shim_copy()), BUILD_2);
    assert_eq!(differing(&before, &snapshot(&env)), [named(&env, &env.shim_copy())]);
    let after = status(&env);
    assert_eq!((after.state, after.chained), (ShimState::Installed, true));
}

#[test]
fn an_uninstall_during_the_refresh_is_not_undone() {
    let env = Env::new();
    installed_over_foreign(&env);
    let build = newer(&env);

    super::super::refresh::refresh_with(&env.settings(), &env.ctx(), &build, &|| {
        uninstall(&env).unwrap();
    })
    .unwrap();

    assert!(!env.shim_copy().exists(), "the refresh must not resurrect the copy");
    assert_eq!(status(&env).state, ShimState::Foreign);
}

#[test]
fn only_the_copy_of_the_profile_the_settings_name_is_refreshed() {
    let env = Env::new();
    let dir = env.root().join("Perfil de trabalho");
    env.write(&dir.join("settings.json"), settings_plain());
    let work = ClaudeSettings { config_dir: Some(dir.display().to_string()), ..env.settings() };
    install(&env).unwrap();
    super::super::install_shim(&work, &env.ctx(), &env.shim_source()).unwrap();
    let work_copy = Paths::resolve(&work, &env.ctx()).unwrap().exe;
    assert_ne!(work_copy, env.shim_copy(), "one copy per profile");
    let build = newer(&env);
    let before = snapshot(&env);

    super::super::refresh_shim_copy(&work, &env.ctx(), &build).unwrap();

    assert_eq!(text(&work_copy), BUILD_2);
    assert_eq!(text(&env.shim_copy()), BUILD_1, "the other profile waits for its own turn");
    assert_eq!(differing(&before, &snapshot(&env)), [named(&env, &work_copy)]);
}

#[test]
fn nothing_is_touched_unless_the_install_is_ours_healthy_and_consented() {
    type Arrange = fn(&Env);
    type Settings = fn(&Env) -> ClaudeSettings;
    let same: Settings = |env| env.settings();
    let cases: [(&str, Arrange, Settings); 8] = [
        ("never installed", |env| env.write_settings(settings_plain()), same),
        ("consent revoked", installed_over_foreign, |env| ClaudeSettings {
            shim_consent: false,
            ..env.settings()
        }),
        (
            "somebody else's status line",
            |env| {
                installed_over_foreign(env);
                let theirs =
                    json!({"statusLine": {"type": "command", "command": "python ~/novo.py"}});
                env.write_settings(theirs.to_string());
            },
            same,
        ),
        (
            "status line removed by hand",
            |env| {
                installed_over_foreign(env);
                env.write_settings(settings_plain());
            },
            same,
        ),
        (
            "the copy is gone",
            |env| {
                installed_over_foreign(env);
                std::fs::remove_file(env.shim_copy()).unwrap();
            },
            same,
        ),
        (
            "the command points elsewhere",
            |env| {
                installed_over_foreign(env);
                let other = command(false).replace("claude/rate_limits.json", "claude/outro.json");
                let moved = json!({"statusLine": {"type": "command", "command": other}});
                env.write_settings(moved.to_string());
            },
            same,
        ),
        (
            "settings.json is not json",
            |env| {
                installed_over_foreign(env);
                env.write_settings("{ not json");
            },
            same,
        ),
        ("unusable config dir", installed_over_foreign, |env| ClaudeSettings {
            config_dir: Some("relative/dir".into()),
            ..env.settings()
        }),
    ];
    let mut problems = Vec::new();
    for (name, arrange, settings) in cases {
        let env = Env::new();
        arrange(&env);
        let build = newer(&env);
        let before = snapshot(&env);

        let result = super::super::refresh_shim_copy(&settings(&env), &env.ctx(), &build);

        let changed = differing(&before, &snapshot(&env));
        if result != Ok(()) || !changed.is_empty() {
            problems.push(format!("{name}: {result:?}, files that differ: {changed:?}"));
        }
    }
    assert_eq!(problems, Vec::<String>::new(), "every case, not just the first");
}

#[test]
fn a_missing_or_empty_bundled_build_is_reported_and_the_working_copy_stays() {
    let env = Env::new();
    installed_over_foreign(&env);
    let empty = env.shim_source_with(b"");
    let missing = env.root().join("build").join("nao-existe.exe");
    let before = snapshot(&env);
    for build in [&empty, &missing] {
        let error = refresh(&env, build).unwrap_err();
        assert_eq!(
            (error.code, error.retryable, reason(&error)),
            (ErrorCode::InstallFailed, false, "shim_missing")
        );
    }
    assert_unchanged(&env, &before, "refresh without a usable bundled build");
    assert_eq!(text(&env.shim_copy()), BUILD_1);
}

#[test]
fn a_copy_that_is_already_current_is_not_rewritten() {
    let env = Env::new();
    installed_over_foreign(&env);
    // Pinned in the past: a rewrite would move it to now.
    let past = UNIX_EPOCH + Duration::from_secs(1_000_000_000);
    let copy = std::fs::OpenOptions::new().write(true).open(env.shim_copy()).unwrap();
    copy.set_modified(past).unwrap();
    drop(copy);

    refresh(&env, &env.shim_source()).unwrap();

    let modified = std::fs::metadata(env.shim_copy()).unwrap().modified().unwrap();
    assert_eq!(modified, past, "same bytes, so nothing was written");
}

// A copy Claude Code is running cannot be replaced on Windows; elsewhere the rename goes through.
#[cfg(windows)]
mod running_copy {
    use std::fs::{File, OpenOptions};
    use std::os::windows::fs::OpenOptionsExt;

    use super::*;

    /// Readers allowed, but nobody may write, rename or delete while it is open.
    fn run(env: &Env) -> File {
        OpenOptions::new().read(true).share_mode(1).open(env.shim_copy()).unwrap()
    }

    #[test]
    fn a_copy_that_is_already_current_is_not_written_so_a_running_one_is_no_problem() {
        let env = Env::new();
        installed_over_foreign(&env);
        let before = snapshot(&env);
        let running = run(&env);

        refresh(&env, &env.shim_source()).unwrap();

        drop(running);
        assert_unchanged(&env, &before, "refresh with the build that is already there");
    }

    #[test]
    fn a_copy_that_is_running_is_reported_as_retryable_and_left_alone_until_it_stops() {
        let env = Env::new();
        installed_over_foreign(&env);
        let build = newer(&env);
        let before = snapshot(&env);
        let running = run(&env);

        let error = refresh(&env, &build).unwrap_err();
        assert_eq!(
            (error.code, error.retryable, reason(&error)),
            (ErrorCode::InstallFailed, true, "copy_failed")
        );
        assert_unchanged(&env, &before, "refresh of a copy that is running");

        drop(running);
        refresh(&env, &build).unwrap();
        assert_eq!(text(&env.shim_copy()), BUILD_2, "the next start catches up");
    }
}
