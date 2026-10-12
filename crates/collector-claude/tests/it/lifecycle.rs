//! The whole story through the public API: look, install, record, read, leave.

use collector_claude::{collect, install_shim, refresh_shim_copy, shim_status, uninstall_shim};
use usage_model::settings::ClaudeSettings;
use usage_model::{ErrorCode, ShimState, ViewState};

use crate::common::{Env, files_under, notice_codes, ok_both, settings_foreign};
use crate::setup::{recording, view_of};

#[tokio::test]
async fn from_a_foreign_status_line_to_numbers_and_back() {
    let env = Env::new();
    env.write_settings(settings_foreign());
    assert_eq!(shim_status(&env.settings(), &env.ctx()).state, ShimState::Foreign);
    assert_eq!(view_of(&env).await.state, ViewState::NeedsSetup);

    let status = install_shim(&env.settings(), &env.ctx(), &env.shim_source()).unwrap();
    assert_eq!((status.state, status.chained), (ShimState::Installed, true));
    assert_eq!(notice_codes(&view_of(&env).await), ["claude_waiting_first_session"]);

    env.write_rate_limits(&ok_both());
    assert_eq!(view_of(&env).await.state, ViewState::Ready);

    let left = uninstall_shim(&env.settings(), &env.ctx()).unwrap();
    assert_eq!(left.state, ShimState::Foreign, "the user's own command is back");
    assert_eq!(env.settings_text(), settings_foreign());
    assert_eq!(view_of(&env).await.state, ViewState::NeedsSetup);
    assert!(files_under(&env.data).is_empty(), "nothing of ours is left in the data dir");
}

#[tokio::test]
async fn revoking_consent_stops_reading_and_installing_but_leaving_stays_possible() {
    let env = recording();
    let revoked = ClaudeSettings { shim_consent: false, ..env.settings() };
    assert_eq!(collect(&revoked, &env.ctx()).await.state, ViewState::NeedsConsent);
    let refused = install_shim(&revoked, &env.ctx(), &env.shim_source()).unwrap_err();
    assert_eq!(
        (refused.code, refused.params["field"].as_str()),
        (ErrorCode::InvalidInput, "shimConsent")
    );
    assert_eq!(uninstall_shim(&revoked, &env.ctx()).unwrap().state, ShimState::NotInstalled);
}

#[tokio::test]
async fn an_app_update_refreshes_the_copy_without_touching_the_settings() {
    let env = recording();
    let before = env.settings_bytes();
    let newer = env.shim_source_with(b"fabricated shim build 2\n");
    let status = install_shim(&env.settings(), &env.ctx(), &newer).unwrap();
    assert_eq!(status.state, ShimState::Installed);
    assert_eq!(std::fs::read(env.shim_copy()).unwrap(), b"fabricated shim build 2\n");
    assert_eq!(env.settings_bytes(), before);
    assert_eq!(view_of(&env).await.state, ViewState::Ready, "the recording is kept");
}

#[tokio::test]
async fn an_app_update_reaches_the_installed_copy_at_startup_without_a_reinstall() {
    let env = recording();
    let before = env.settings_bytes();
    let newer = env.shim_source_with(b"fabricated shim build 2\n");

    refresh_shim_copy(&env.settings(), &env.ctx(), &newer).unwrap();

    assert_eq!(std::fs::read_to_string(env.shim_copy()).unwrap(), "fabricated shim build 2\n");
    assert_eq!(env.settings_bytes(), before, "settings.json is not rewritten");
    assert_eq!(shim_status(&env.settings(), &env.ctx()).state, ShimState::Installed);
    assert_eq!(view_of(&env).await.state, ViewState::Ready, "the recording is kept");
}

#[tokio::test]
async fn a_broken_install_is_visible_and_repairable() {
    let env = recording();
    std::fs::remove_file(env.shim_copy()).unwrap();
    assert_eq!(shim_status(&env.settings(), &env.ctx()).state, ShimState::Outdated);
    assert_eq!(view_of(&env).await.state, ViewState::NeedsSetup);

    install_shim(&env.settings(), &env.ctx(), &env.shim_source()).unwrap();
    assert_eq!(view_of(&env).await.state, ViewState::Ready);
}

#[tokio::test]
async fn concurrent_polls_read_the_same_files_without_disturbing_each_other() {
    let env = recording();
    let polls: Vec<_> = (0..8)
        .map(|_| {
            let (settings, ctx) = (env.settings(), env.ctx());
            tokio::spawn(async move { collect(&settings, &ctx).await })
        })
        .collect();
    for poll in polls {
        assert_eq!(poll.await.unwrap().state, ViewState::Ready);
    }
    assert_eq!(shim_status(&env.settings(), &env.ctx()).state, ShimState::Installed);
}
