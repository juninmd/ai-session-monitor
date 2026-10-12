//! The gates of `collect`: a disabled tab or a missing consent answer at once, before the config
//! dir is even resolved. Valid, installed data is on disk in every case, so any read that
//! happened first would show up as a different state.

use collector_claude::collect;
use usage_model::settings::ClaudeSettings;
use usage_model::{Ctx, ErrorCode, HarnessId, HarnessView, ViewState};

use crate::common::{NOW, files_under};
use crate::setup::recording as installed;

#[tokio::test]
async fn a_disabled_tab_answers_before_anything_is_read() {
    let env = installed();
    assert_eq!(
        collect(&env.settings(), &env.ctx()).await.state,
        ViewState::Ready,
        "the setup is valid"
    );
    let off = ClaudeSettings { enabled: false, ..env.settings() };
    assert_eq!(collect(&off, &env.ctx()).await, HarnessView::disabled(HarnessId::Claude));
    let off_with_oauth = ClaudeSettings { oauth_usage: true, ..off };
    assert_eq!(
        collect(&off_with_oauth, &env.ctx()).await,
        HarnessView::disabled(HarnessId::Claude)
    );
}

#[tokio::test]
async fn a_missing_consent_answers_before_anything_is_read() {
    let env = installed();
    let unasked = ClaudeSettings { shim_consent: false, ..env.settings() };
    assert_eq!(collect(&unasked, &env.ctx()).await, HarnessView::needs_consent(HarnessId::Claude));
    let with_oauth = ClaudeSettings { oauth_usage: true, ..unasked };
    assert_eq!(
        collect(&with_oauth, &env.ctx()).await,
        HarnessView::needs_consent(HarnessId::Claude)
    );
}

#[tokio::test]
async fn a_disabled_tab_wins_over_a_missing_consent() {
    let env = installed();
    let neither = ClaudeSettings { enabled: false, shim_consent: false, ..env.settings() };
    assert_eq!(collect(&neither, &env.ctx()).await, HarnessView::disabled(HarnessId::Claude));
}

#[tokio::test]
async fn the_gates_come_before_the_config_dir_is_resolved() {
    let env = installed();
    let broken = Some("relative\0dir".to_owned());
    let off = ClaudeSettings { enabled: false, config_dir: broken.clone(), ..env.settings() };
    let unasked =
        ClaudeSettings { shim_consent: false, config_dir: broken.clone(), ..env.settings() };
    assert_eq!(collect(&off, &env.ctx()).await.state, ViewState::Disabled);
    assert_eq!(collect(&unasked, &env.ctx()).await.state, ViewState::NeedsConsent);

    // Past the gates the same config dir is what it is: an input error, named.
    let both = ClaudeSettings { config_dir: broken, ..env.settings() };
    let view = collect(&both, &env.ctx()).await;
    let error = view.error.unwrap();
    assert_eq!(
        (view.state, error.code, error.retryable),
        (ViewState::Error, ErrorCode::InvalidInput, false)
    );
    assert_eq!(error.params["field"], "configDir");
}

#[tokio::test]
async fn gated_calls_create_nothing_even_when_every_directory_is_missing() {
    let root = tempfile::tempdir().unwrap();
    let ctx = Ctx::new(NOW, root.path().join("sem dados"), root.path().join("sem casa"));
    for settings in [
        ClaudeSettings { enabled: false, ..ClaudeSettings::default() },
        ClaudeSettings { shim_consent: false, ..ClaudeSettings::default() },
    ] {
        collect(&settings, &ctx).await;
    }
    assert_eq!(files_under(root.path()), Vec::<String>::new());
    assert!(!root.path().join("sem dados").exists() && !root.path().join("sem casa").exists());
}

#[tokio::test]
async fn reading_never_creates_anything_either() {
    let root = tempfile::tempdir().unwrap();
    let ctx = Ctx::new(NOW, root.path().join("dados"), root.path().join("casa"));
    let consented = ClaudeSettings { shim_consent: true, ..ClaudeSettings::default() };
    let view = collect(&consented, &ctx).await;
    assert_eq!(view.state, ViewState::NeedsSetup);
    assert!(files_under(root.path()).is_empty() && !root.path().join("casa").exists());
}
