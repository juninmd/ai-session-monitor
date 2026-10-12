//! The CLI stays unrun unless the flag is on and there is a usable statusLine view to add to.

use usage_model::{HarnessId, HarnessView};

use super::*;
use crate::testenv::{NOW, settings_foreign};

async fn quiet(env: &Env, settings: &ClaudeSettings, now: i64) -> HarnessView {
    let cli = ScriptedCli::says(REPORT);
    let view = poll_with(settings, env, &cli, now).await;
    assert_eq!(cli.calls(), 0, "the CLI must not be started: {view:?}");
    assert!(!env.cli_cache().exists(), "no throttle file either");
    assert!(!view.meta.contains_key("extra_source"));
    view
}

#[tokio::test]
async fn with_the_option_off_the_cli_is_never_started() {
    let env = installed_env();
    let view = quiet(&env, &env.settings(), NOW).await;
    assert_eq!(view.windows.len(), 2);
}

#[tokio::test]
async fn a_disabled_tab_or_a_missing_consent_starts_nothing() {
    let env = installed_env();
    let disabled = ClaudeSettings { enabled: false, ..on(&env) };
    assert_eq!(quiet(&env, &disabled, NOW).await, HarnessView::disabled(HarnessId::Claude));
    let unasked = ClaudeSettings { shim_consent: false, ..on(&env) };
    assert_eq!(quiet(&env, &unasked, NOW).await, HarnessView::needs_consent(HarnessId::Claude));
}

#[tokio::test]
async fn no_usable_statusline_view_means_no_process() {
    let env = Env::new();
    env.write_settings(settings_foreign());
    quiet(&env, &on(&env), NOW).await;

    let env = installed_env();
    std::fs::remove_file(env.rate_limits()).unwrap();
    quiet(&env, &on(&env), NOW).await;

    let env = installed_env();
    env.write_rate_limits("{ torn");
    quiet(&env, &on(&env), NOW).await;
}

#[tokio::test]
async fn a_stale_recording_does_not_wake_the_cli() {
    let env = installed_env();
    // Both windows of the fixture have reset by then.
    let view = quiet(&env, &on(&env), NOW + 400_001).await;
    assert!(view.stale);
}
