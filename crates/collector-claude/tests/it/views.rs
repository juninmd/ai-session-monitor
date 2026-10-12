//! The views `collect` produces from what the shim recorded: Ready, NoData and NeedsSetup. The
//! recording is a fabricated fixture; the installation is the real `install_shim` into a temp dir.

use collector_claude::{collect, install_shim};
use serde_json::json;
use usage_model::{AmountUnit, HarnessId, HarnessView, Severity, Source, ViewState, WindowKind};

use crate::common::{
    Env, NOW, notice_codes, null_limits, ok_both, ok_spend, settings_foreign, settings_plain,
};
use crate::setup::{installed, view_of};

#[tokio::test]
async fn a_fresh_recording_is_ready_with_both_windows() {
    let env = installed();
    env.write_rate_limits(&ok_both());
    let view = view_of(&env).await;

    assert_eq!(
        (view.id, view.state, view.stale, view.read_at),
        (HarnessId::Claude, ViewState::Ready, false, Some(NOW))
    );
    assert_eq!(view.source, Some(Source { id: "statusline".into(), official: true }));
    assert!(
        view.notices.is_empty()
            && view.error.is_none()
            && view.plan.is_none()
            && view.meta.is_empty()
    );
    let [five_hour, weekly] = view.windows.as_slice() else {
        panic!("two windows expected: {view:?}")
    };
    assert_eq!(
        (five_hour.id.as_str(), five_hour.kind, five_hour.window_minutes),
        ("five_hour", WindowKind::FiveHour, Some(300))
    );
    assert_eq!((five_hour.used_pct, five_hour.resets_at), (Some(23.5), Some(NOW + 9_000)));
    assert_eq!(
        (weekly.id.as_str(), weekly.kind, weekly.window_minutes),
        ("weekly", WindowKind::Weekly, Some(10_080))
    );
    assert_eq!((weekly.used_pct, weekly.resets_at), (Some(41.2), Some(NOW + 400_000)));
}

#[tokio::test]
async fn the_view_crosses_the_wire_in_the_documented_shape() {
    let env = installed();
    env.write_rate_limits(&ok_both());
    let view = view_of(&env).await;
    let wire = serde_json::to_value(&view).unwrap();
    assert_eq!(
        (wire["id"].as_str(), wire["state"].as_str(), wire["readAt"].as_i64()),
        (Some("claude"), Some("ready"), Some(NOW))
    );
    assert_eq!(wire["source"], json!({"id": "statusline", "official": true}));
    assert_eq!(wire["windows"][0]["usedPct"], 23.5);
    assert_eq!(wire["windows"][1]["windowMinutes"], 10_080);
    assert_eq!(
        serde_json::from_value::<HarnessView>(wire).unwrap(),
        view,
        "it reads back unchanged"
    );
}

#[tokio::test]
async fn a_gateway_spend_limit_is_a_budget_window_with_dollars() {
    let env = installed();
    env.write_rate_limits(&ok_spend());
    let view = view_of(&env).await;
    assert_eq!(
        view.windows.iter().map(|w| w.id.as_str()).collect::<Vec<_>>(),
        ["five_hour", "weekly", "spend_limit"]
    );
    let spend = &view.windows[2];
    assert_eq!(
        (spend.kind, spend.used_pct, spend.window_minutes),
        (WindowKind::Budget, Some(62.8), None)
    );
    let amount = spend.amount.as_ref().unwrap();
    assert_eq!((amount.used, amount.limit, amount.unit), (314.12, Some(500.0), AmountUnit::Usd));
}

#[tokio::test]
async fn a_window_the_account_does_not_have_is_simply_absent() {
    let env = installed();
    let only_weekly = json!({"v": 1, "capturedAt": NOW, "rateLimits": {"seven_day": {"used_percentage": 12, "resets_at": NOW + 5000}}});
    env.write_rate_limits(&only_weekly.to_string());
    let view = view_of(&env).await;
    assert_eq!(view.windows.iter().map(|w| w.id.as_str()).collect::<Vec<_>>(), ["weekly"]);
    assert_eq!(view.state, ViewState::Ready);
}

#[tokio::test]
async fn out_of_range_numbers_are_unknown_not_clamped() {
    let env = installed();
    let odd = json!({"v": 1, "capturedAt": NOW, "rateLimits": {"five_hour": {"used_percentage": 140, "resets_at": "soon"}}});
    env.write_rate_limits(&odd.to_string());
    let window = view_of(&env).await.windows.remove(0);
    assert_eq!((window.used_pct, window.resets_at, window.window_minutes), (None, None, Some(300)));
}

#[tokio::test]
async fn an_installed_shim_with_no_recording_yet_is_waiting_for_the_first_session() {
    let env = installed();
    let view = view_of(&env).await;
    assert_eq!((view.state, view.read_at, view.stale), (ViewState::NoData, None, false));
    assert_eq!(view.source, Some(Source { id: "statusline".into(), official: true }));
    assert_eq!(notice_codes(&view), ["claude_waiting_first_session"]);
    assert_eq!(view.notices[0].severity, Severity::Info);
    assert!(view.windows.is_empty() && view.error.is_none());
}

#[tokio::test]
async fn a_session_without_rate_limits_is_no_data_with_a_warning() {
    let env = installed();
    let empty = json!({"v": 1, "capturedAt": NOW, "rateLimits": {}}).to_string();
    let missing = json!({"v": 1, "capturedAt": NOW}).to_string();
    for recording in [null_limits(), empty, missing] {
        env.write_rate_limits(&recording);
        let view = view_of(&env).await;
        assert_eq!((view.state, view.read_at), (ViewState::NoData, Some(NOW)), "{recording}");
        assert_eq!(notice_codes(&view), ["claude_no_rate_limits"]);
        assert_eq!(view.notices[0].severity, Severity::Warn);
        assert!(view.windows.is_empty() && view.error.is_none());
    }
}

#[tokio::test]
async fn without_a_settings_file_or_a_status_line_the_setup_is_needed() {
    let env = Env::new();
    let view = view_of(&env).await;
    assert_eq!(
        (view.state, view.notices.len(), view.error.is_none()),
        (ViewState::NeedsSetup, 0, true)
    );
    assert!(view.windows.is_empty() && view.source.is_none());

    env.write_settings(settings_plain());
    assert_eq!(view_of(&env).await.state, ViewState::NeedsSetup);
}

#[tokio::test]
async fn somebody_elses_status_line_needs_the_setup_too_and_is_left_alone() {
    let env = Env::new();
    env.write_settings(settings_foreign());
    let view = view_of(&env).await;
    assert_eq!((view.state, view.notices.len()), (ViewState::NeedsSetup, 0));
    assert_eq!(env.settings_text(), settings_foreign(), "collecting only ever reads");
}

#[tokio::test]
async fn an_installed_shim_whose_file_went_missing_needs_setup_and_says_why() {
    let env = installed();
    env.write_rate_limits(&ok_both());
    std::fs::remove_file(env.shim_copy()).unwrap();
    let view = view_of(&env).await;
    assert_eq!(view.state, ViewState::NeedsSetup);
    assert_eq!(notice_codes(&view), ["claude_shim_outdated"]);
    assert_eq!(view.notices[0].severity, Severity::Warn);
    assert!(view.windows.is_empty(), "stale numbers are not shown behind a broken install");
}

#[tokio::test]
async fn a_config_dir_outside_home_with_accents_and_spaces_works_end_to_end() {
    let env = Env::new();
    let dir = env.root().join("Configuração do Claude");
    let settings = usage_model::settings::ClaudeSettings {
        config_dir: Some(dir.display().to_string()),
        ..env.settings()
    };
    install_shim(&settings, &env.ctx(), &env.shim_source()).unwrap();
    assert!(dir.join("settings.json").is_file() && !env.config_dir().exists());
    env.write(&env.recording_target(&dir.join("settings.json")), ok_both());
    assert_eq!(collect(&settings, &env.ctx()).await.state, ViewState::Ready);
    assert_eq!(
        collect(&env.settings(), &env.ctx()).await.state,
        ViewState::NeedsSetup,
        "the default dir has no install"
    );
}

#[tokio::test]
async fn two_profiles_never_read_each_others_numbers() {
    let env = Env::new();
    let work_dir = env.root().join("Perfil de trabalho");
    let work = usage_model::settings::ClaudeSettings {
        config_dir: Some(work_dir.display().to_string()),
        ..env.settings()
    };
    install_shim(&env.settings(), &env.ctx(), &env.shim_source()).unwrap();
    install_shim(&work, &env.ctx(), &env.shim_source()).unwrap();

    env.write(&env.recording_target(&work_dir.join("settings.json")), ok_both());
    assert_eq!(collect(&work, &env.ctx()).await.state, ViewState::Ready);
    let main = collect(&env.settings(), &env.ctx()).await;
    assert_eq!(
        (main.state, notice_codes(&main), main.windows.len()),
        (ViewState::NoData, vec!["claude_waiting_first_session"], 0),
        "the other account's numbers are not this one's"
    );
}

#[tokio::test]
async fn a_data_dir_outside_home_works_end_to_end() {
    let env = Env::with_data_outside_home();
    install_shim(&env.settings(), &env.ctx(), &env.shim_source()).unwrap();
    env.write_rate_limits(&ok_both());
    assert_eq!(view_of(&env).await.state, ViewState::Ready);
}
