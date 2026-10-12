//! Two Claude profiles (`CLAUDE_CONFIG_DIR`) on one machine share one app data dir. Everything an
//! install owns there is per profile, so what one profile does never reaches the other's.

use std::path::PathBuf;

use serde_json::json;
use usage_model::settings::ClaudeSettings;
use usage_model::{ShimState, ShimStatus};

use super::*;
use crate::paths::Paths;
use crate::testenv::{settings_foreign, settings_plain};

/// Chained to `python ~/trabalho.py`; the other keys must come back as they were.
const WORK: &str = "{\n  \"model\": \"sonnet\",\n  \"statusLine\": {\n    \"type\": \"command\",\n    \"command\": \"python ~/trabalho.py\",\n    \"padding\": 1\n  }\n}\n";

struct Profile {
    settings: ClaudeSettings,
    dir: PathBuf,
}

impl Profile {
    fn settings_json(&self) -> PathBuf {
        self.dir.join("settings.json")
    }

    fn state_copy(&self) -> PathBuf {
        self.dir.join("settings.json.aism-state")
    }

    fn text(&self) -> String {
        std::fs::read_to_string(self.settings_json()).unwrap()
    }

    fn command(&self) -> String {
        read_json(&self.settings_json())["statusLine"]["command"].as_str().unwrap().to_owned()
    }
}

/// The default profile (`~/.claude`, chained to statusline.sh) and one of its own (chained to
/// `WORK`), neither installed yet.
fn two_profiles() -> (Env, Profile, Profile) {
    let env = Env::new();
    env.write_settings(settings_foreign());
    let main = Profile { settings: env.settings(), dir: env.config_dir() };
    let dir = env.root().join("Perfil de trabalho");
    env.write(&dir.join("settings.json"), WORK);
    let settings = ClaudeSettings { config_dir: Some(dir.display().to_string()), ..env.settings() };
    (env, main, Profile { settings, dir })
}

fn install_in(env: &Env, profile: &Profile) -> ShimStatus {
    super::super::install_shim(&profile.settings, &env.ctx(), &env.shim_source()).unwrap()
}

fn uninstall_in(env: &Env, profile: &Profile) -> ShimStatus {
    super::super::uninstall_shim(&profile.settings, &env.ctx()).unwrap()
}

fn status_in(env: &Env, profile: &Profile) -> (ShimState, bool) {
    let status = super::super::shim_status(&profile.settings, &env.ctx());
    (status.state, status.chained)
}

fn paths_in(env: &Env, profile: &Profile) -> Paths {
    Paths::resolve(&profile.settings, &env.ctx()).unwrap()
}

/// Everything one profile's install owns, as (file, text); `None` for a file that is absent.
fn owned(env: &Env, profile: &Profile) -> Vec<(String, Option<String>)> {
    let paths = paths_in(env, profile);
    [profile.settings_json(), paths.state, paths.chain, paths.exe]
        .into_iter()
        .map(|file| {
            let name = file.strip_prefix(env.root()).unwrap().to_string_lossy().replace('\\', "/");
            (name, std::fs::read(&file).ok().map(|b| String::from_utf8_lossy(&b).into_owned()))
        })
        .collect()
}

fn assert_nothing_left(env: &Env, profiles: &[&Profile]) {
    assert_eq!(files_under(&env.data), Vec::<String>::new());
    assert_eq!(std::fs::read_dir(&env.data).unwrap().count(), 0, "not even a directory");
    for profile in profiles {
        let kept = ["settings.json", "settings.json.aism-backup"];
        assert_eq!(files_under(&profile.dir), kept, "only the user's own files remain");
    }
}

#[test]
fn installing_in_a_second_profile_leaves_the_first_ones_install_untouched() {
    let (env, main, work) = two_profiles();
    install_in(&env, &main);
    let before = owned(&env, &main);
    assert!(before.iter().all(|(_, text)| text.is_some()), "{before:?}");

    let result = install_in(&env, &work);
    assert_eq!((result.state, result.chained), (ShimState::Installed, true));
    assert_eq!(owned(&env, &main), before);
    assert_eq!(status_in(&env, &main), (ShimState::Installed, true));
    assert!(owned(&env, &work).iter().all(|(_, text)| text.is_some()), "its files are its own");
    assert!(work.state_copy().is_file(), "and its record sits beside its own settings.json");
    assert_eq!(
        read_json(&paths_in(&env, &main).chain),
        json!({"command": "~/.claude/statusline.sh"})
    );
    assert_eq!(read_json(&paths_in(&env, &work).chain), json!({"command": "python ~/trabalho.py"}));
    assert_ne!(main.command(), work.command(), "each one records to a file of its own");
}

#[test]
fn removing_the_second_profile_leaves_the_first_installed_and_chained() {
    let (env, main, work) = two_profiles();
    install_in(&env, &main);
    install_in(&env, &work);
    let main_installed = owned(&env, &main);

    let left = uninstall_in(&env, &work);
    assert_eq!(work.text(), WORK, "its own original, byte for byte");
    assert_eq!(left.state, ShimState::Foreign);
    assert_eq!(status_in(&env, &main), (ShimState::Installed, true));
    assert_eq!(owned(&env, &main), main_installed, "nothing of the first install was touched");
    assert!(main.state_copy().is_file() && !work.state_copy().exists());

    uninstall_in(&env, &main);
    assert_eq!(env.settings_text(), settings_foreign());
    assert_nothing_left(&env, &[&main, &work]);
}

#[test]
fn removing_the_first_profile_first_restores_its_own_original_and_keeps_the_second() {
    let (env, main, work) = two_profiles();
    install_in(&env, &main);
    install_in(&env, &work);
    let work_installed = owned(&env, &work);

    uninstall_in(&env, &main);
    assert_eq!(env.settings_text(), settings_foreign(), "not the second profile's command");
    assert_eq!(status_in(&env, &work), (ShimState::Installed, true));
    assert_eq!(owned(&env, &work), work_installed, "nothing of the second install was touched");
    assert!(work.state_copy().is_file() && !main.state_copy().exists());

    uninstall_in(&env, &work);
    assert_eq!(work.text(), WORK);
    assert_nothing_left(&env, &[&main, &work]);
}

#[test]
fn a_second_profile_without_a_status_line_does_not_unchain_the_first() {
    let (env, main, work) = two_profiles();
    env.write(&work.settings_json(), settings_plain());
    install_in(&env, &main);

    install_in(&env, &work);
    assert_eq!(status_in(&env, &main), (ShimState::Installed, true));
    assert!(!paths_in(&env, &work).chain.exists(), "nothing to chain there");
    uninstall_in(&env, &main);
    assert_eq!(env.settings_text(), settings_foreign());
    uninstall_in(&env, &work);
    assert_eq!(work.text(), settings_plain());
    assert_nothing_left(&env, &[&main, &work]);
}

#[test]
fn after_a_wipe_each_profile_is_repaired_from_its_own_copy() {
    let (env, main, work) = two_profiles();
    install_in(&env, &main);
    install_in(&env, &work);
    assert!(main.state_copy().is_file() && work.state_copy().is_file(), "one per profile");
    let (main_installed, work_installed) = (main.text(), work.text());
    std::fs::remove_dir_all(&env.data).unwrap();

    assert!(install_in(&env, &work).chained);
    assert_eq!(status_in(&env, &main).0, ShimState::Outdated, "not repaired by the other one");
    assert!(install_in(&env, &main).chained);
    assert_eq!((main.text(), work.text()), (main_installed, work_installed));

    std::fs::remove_dir_all(&env.data).unwrap();
    uninstall_in(&env, &work);
    uninstall_in(&env, &main);
    assert_eq!(work.text(), WORK);
    assert_eq!(env.settings_text(), settings_foreign());
}
