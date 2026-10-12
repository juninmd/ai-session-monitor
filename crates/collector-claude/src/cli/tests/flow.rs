//! The CLI inside the whole poll: what is added, the throttle, the fallback to the OAuth API and
//! the guarantee that the account's token is not needed when the CLI answers.

use usage_model::{ErrorCode, HarnessError, ViewState, WindowKind};

use super::*;
use crate::oauth::fake_server::FakeServer;
use crate::testenv::{FAKE_TOKEN, NOW, notice_codes, usage_limits, wire};

#[tokio::test]
async fn fable_opus_and_sonnet_join_the_statusline_windows() {
    let env = installed_env();
    let cli = ScriptedCli::says(REPORT);
    let view = poll(&env, &cli, NOW).await;

    assert_eq!(view.state, ViewState::Ready);
    assert_eq!(ids(&view), ["five_hour", "weekly", "fable:weekly", "opus:weekly", "sonnet:weekly"]);
    assert_eq!(view.meta["extra_source"], "cli_usage");
    assert!(view.notices.is_empty() && view.error.is_none() && !view.stale);

    let fable = &view.windows[2];
    assert_eq!(
        (fable.kind, fable.group.as_deref(), fable.label.as_deref()),
        (WindowKind::Weekly, Some("fable"), Some("Fable"))
    );
    assert_eq!(
        (fable.used_pct, fable.resets_at, fable.window_minutes),
        (Some(0.0), Some(WEEKLY_RESET), Some(10_080))
    );
    let opus = &view.windows[3];
    assert_eq!((opus.used_pct, opus.resets_at), (Some(12.5), Some(WEEKLY_RESET)));
    let sonnet = &view.windows[4];
    assert_eq!((sonnet.used_pct, sonnet.resets_at), (Some(34.0), None));
    assert_eq!(cli.calls(), 1);
}

#[tokio::test]
async fn nothing_after_the_limits_block_reaches_the_view() {
    let env = installed_env();
    let view = poll(&env, &ScriptedCli::says(REPORT), NOW).await;
    let shown = wire(&view);
    for leaked in ["acme-secret", "Injected", "contributing", "requests"] {
        assert!(!shown.contains(leaked), "{leaked} leaked: {shown}");
    }
}

#[tokio::test]
async fn an_all_models_only_plan_adds_nothing_and_does_not_fall_back_to_the_api() {
    let env = installed_env();
    let only_all =
        "Current session: 3% used · resets a\nCurrent week (all models): 7% used · resets b";
    let cli = ScriptedCli::says(only_all);
    let view = poll(&env, &cli, NOW).await;

    assert_eq!(ids(&view), ["five_hour", "weekly"]);
    assert!(view.notices.is_empty() && view.meta.is_empty() && view.error.is_none());
    // The OAuth path would have complained about the missing login.
    assert_eq!(notice_codes(&view), Vec::<&str>::new());
}

#[tokio::test]
async fn the_answer_is_reused_for_ten_minutes_then_asked_again() {
    let env = installed_env();
    let cli = ScriptedCli::says(REPORT);
    poll(&env, &cli, NOW).await;
    let again = poll(&env, &cli, NOW + 599).await;
    assert_eq!(cli.calls(), 1, "a 30 s tick must not start a process");
    assert_eq!(ids(&again).len(), 5, "the cached rows still show");

    poll(&env, &cli, NOW + 600).await;
    assert_eq!(cli.calls(), 2);
}

#[tokio::test]
async fn the_cached_rows_follow_the_statuslines_epoch_not_a_stored_time() {
    let env = installed_env();
    let cli = ScriptedCli::says(REPORT);
    poll(&env, &cli, NOW).await;
    env.write_rate_limits(&ok_both().replace("1800400000", "1800500000"));
    let view = poll(&env, &cli, NOW + 60).await;
    assert_eq!(cli.calls(), 1);
    assert_eq!(view.windows[2].resets_at, Some(1_800_500_000));
}

#[tokio::test]
async fn polls_that_overlap_start_one_process() {
    let env = installed_env();
    let cli = ScriptedCli::says(REPORT);
    let (a, b, c) =
        tokio::join!(poll(&env, &cli, NOW), poll(&env, &cli, NOW), poll(&env, &cli, NOW));
    assert_eq!(cli.calls(), 1);
    assert!([a, b, c].iter().all(|v| v.windows.len() == 5));
}

#[tokio::test]
async fn the_configured_claude_dir_is_handed_to_the_cli() {
    let env = Env::new();
    let other = env.root().join("outro perfil");
    let settings = ClaudeSettings {
        config_dir: Some(format!("  {}  ", other.display())),
        oauth_usage: true,
        ..env.settings()
    };
    crate::install_shim(&settings, &env.ctx(), &env.shim_source()).unwrap();
    let paths = crate::paths::Paths::resolve(&settings, &env.ctx()).unwrap();
    env.write(&paths.rate_limits, ok_both());

    let cli = ScriptedCli::says(REPORT);
    let view = poll_with(&settings, &env, &cli, NOW).await;
    assert_eq!(view.windows.len(), 5, "{view:?}");
    assert_eq!(cli.config_dir(), Some(other));

    let plain = ScriptedCli::says(REPORT);
    poll(&installed_env(), &plain, NOW).await;
    assert_eq!(plain.config_dir(), None, "no setting, no variable");
}

// The OAuth API is only the fallback.

async fn oauth_ready() -> (Env, FakeServer) {
    let env = installed_env();
    env.write_credentials((NOW + 2 * 86_400) * 1000);
    (env, FakeServer::json(&usage_limits()))
}

fn endpoint(server: &FakeServer) -> Endpoint {
    Endpoint {
        base: server.base(),
        allow_loopback_http: true,
        timeout: std::time::Duration::from_secs(5),
    }
}

#[tokio::test]
async fn when_the_cli_answers_the_token_is_never_read_nor_sent() {
    let (env, server) = oauth_ready().await;
    let cli = ScriptedCli::says(REPORT);
    let view = crate::collect_with_cli(&on(&env), &env.ctx(), &endpoint(&server), &cli).await;
    server.assert_untouched();
    assert!(!env.oauth_cache().exists());
    assert_eq!(view.meta["extra_source"], "cli_usage");
    assert!(!wire(&view).contains(FAKE_TOKEN));
}

#[tokio::test]
async fn without_the_cli_the_oauth_api_answers() {
    let (env, server) = oauth_ready().await;
    let cli = ScriptedCli::new(Err(CliError::Absent));
    let view = crate::collect_with_cli(&on(&env), &env.ctx(), &endpoint(&server), &cli).await;
    assert_eq!(ids(&view), ["five_hour", "weekly", "opus:weekly", "sonnet:weekly"]);
    assert_eq!(view.meta["extra_source"], "oauth_usage");
    assert_eq!(server.connections(), 1);
    assert!(!env.cli_cache().exists(), "an absent CLI is looked for again at every poll");
}

#[tokio::test]
async fn a_failing_cli_falls_back_and_is_not_retried_for_half_an_hour() {
    let (env, server) = oauth_ready().await;
    let cli = ScriptedCli::new(Err(CliError::Failed(HarnessError::new(ErrorCode::Timeout, true))));
    let view = crate::collect_with_cli(&on(&env), &env.ctx(), &endpoint(&server), &cli).await;
    assert_eq!(view.meta["extra_source"], "oauth_usage");

    // Past the OAuth throttle but inside the CLI backoff.
    crate::collect_with_cli(&on(&env), &env.ctx_at(NOW + 1000), &endpoint(&server), &cli).await;
    assert_eq!(cli.calls(), 1);
    crate::collect_with_cli(&on(&env), &env.ctx_at(NOW + 1800), &endpoint(&server), &cli).await;
    assert_eq!(cli.calls(), 2);
}

#[tokio::test]
async fn an_answer_that_is_not_the_usage_report_falls_back_to_the_api() {
    let (env, server) = oauth_ready().await;
    let cli = ScriptedCli::says("I am a model answering a prompt.");
    let view = crate::collect_with_cli(&on(&env), &env.ctx(), &endpoint(&server), &cli).await;
    assert_eq!(view.meta["extra_source"], "oauth_usage");
}

#[tokio::test]
async fn both_sources_failing_leaves_the_statusline_and_one_notice() {
    let env = installed_env();
    let cli = ScriptedCli::new(Err(CliError::Absent));
    let view = poll(&env, &cli, NOW).await;
    assert_eq!(view.windows.len(), 2);
    assert_eq!(notice_codes(&view), ["claude_oauth_partial"]);
    assert_eq!(view.notices[0].params["reason"], "not_signed_in");
}
