//! The CLI as a source of the whole poll: preference over the IDE, fallback, and every way the
//! output can be wrong. Fixtures are fabricated; no test runs a real `agy`.

mod system;

use serde_json::{Value, json};
use usage_model::settings::AntigravitySettings;
use usage_model::{ErrorCode, HarnessView, ViewState};

use super::*;
use crate::collector::{Config, collect_with};
use crate::discovery::DiscoveryError;
use crate::fake::{AbsentCli, Script, ScriptedCli, StaticDiscovery};
use crate::fixtures::{NOW, json as fixture, text};

fn on() -> AntigravitySettings {
    AntigravitySettings { enabled: true, consent: true }
}

fn ctx() -> Ctx {
    Ctx::new(NOW, "data-dir-the-collector-never-uses", "home-dir-the-collector-never-uses")
}

fn closed_ide() -> StaticDiscovery {
    StaticDiscovery::new(Err(DiscoveryError::NotRunning))
}

async fn poll_cli(cli: &impl Cli, discovery: &StaticDiscovery) -> HarnessView {
    collect_with(&on(), &ctx(), discovery, cli, &Config::default()).await
}

async fn from_output(output: Value) -> HarnessView {
    poll_cli(&ScriptedCli::new(Ok(output)), &closed_ide()).await
}

fn rows(view: &HarnessView) -> Vec<(&str, Option<f64>, Option<i64>)> {
    view.windows.iter().map(|w| (w.id.as_str(), w.used_pct, w.resets_at)).collect()
}

fn error_code(view: &HarnessView) -> Option<ErrorCode> {
    view.error.as_ref().map(|error| error.code)
}

fn source_id(view: &HarnessView) -> Option<&str> {
    view.source.as_ref().map(|source| source.id.as_str())
}

async fn working_ide() -> (crate::fake::FakeServer, StaticDiscovery) {
    let server =
        Script::new("t", text("status_ide"), Some(text("summary_full"))).serve_http().await;
    let discovery = StaticDiscovery::new(Ok(vec![server.target("t")]));
    (server, discovery)
}

#[tokio::test]
async fn a_machine_with_only_the_cli_shows_its_four_windows_instead_of_not_running() {
    let ide = closed_ide();
    let view = poll_cli(&ScriptedCli::new(Ok(fixture("cli_usage"))), &ide).await;

    assert_eq!(view.state, ViewState::Ready);
    assert_eq!(
        rows(&view),
        [
            ("gemini:five_hour", Some(38.7602), Some(1_791_791_351)),
            ("gemini:weekly", Some(10.6011), Some(1_792_331_686)),
            ("claude_gpt:five_hour", Some(0.0), Some(1_791_794_938)),
            ("claude_gpt:weekly", Some(85.5), Some(1_792_112_936)),
        ]
    );
    let source = view.source.as_ref().expect("source");
    assert_eq!((source.id.as_str(), source.official), ("agy_cli", false));
    assert_eq!(view.read_at, Some(NOW));
    assert!(view.error.is_none() && view.notices.is_empty() && !view.stale);
    assert_eq!(ide.calls(), 0, "a CLI answer ends the poll: the IDE is not asked");
}

#[tokio::test]
async fn windows_carry_the_lengths_the_plans_fix() {
    let view = from_output(fixture("cli_usage")).await;
    let minutes: Vec<_> = view.windows.iter().map(|w| w.window_minutes).collect();
    assert_eq!(minutes, [Some(300), Some(10_080), Some(300), Some(10_080)]);
}

#[tokio::test]
async fn the_cli_wins_over_a_working_ide_and_the_ide_is_not_asked() {
    let (server, ide) = working_ide().await;
    let view = poll_cli(&ScriptedCli::new(Ok(fixture("cli_usage"))), &ide).await;
    assert_eq!(source_id(&view), Some("agy_cli"));
    assert_eq!((ide.calls(), server.connections()), (0, 0));
}

#[tokio::test]
async fn without_the_cli_the_ide_path_runs_unchanged() {
    let (_server, ide) = working_ide().await;
    let view = poll_cli(&AbsentCli, &ide).await;
    assert_eq!(view.state, ViewState::Ready);
    assert_eq!(source_id(&view), Some("language_server"));
}

#[tokio::test]
async fn a_broken_cli_still_lets_a_working_ide_answer() {
    let (_server, ide) = working_ide().await;
    let cli = ScriptedCli::new(Err(CliError::Failed(HarnessError::new(ErrorCode::Timeout, true))));
    let view = poll_cli(&cli, &ide).await;
    assert_eq!(source_id(&view), Some("language_server"));
}

#[tokio::test]
async fn neither_source_present_is_the_ordinary_not_running_state() {
    let view = poll_cli(&AbsentCli, &closed_ide()).await;
    assert_eq!(error_code(&view), Some(ErrorCode::NotRunning));
    assert!(view.windows.is_empty());
}

#[tokio::test]
async fn a_cli_failure_is_reported_over_a_closed_ide() {
    let timeout = CliError::Failed(HarnessError::new(ErrorCode::Timeout, true));
    let view = poll_cli(&ScriptedCli::new(Err(timeout)), &closed_ide()).await;
    assert_eq!(error_code(&view), Some(ErrorCode::Timeout));
}

#[tokio::test]
async fn a_command_that_did_not_succeed_is_an_endpoint_change_not_zeros() {
    for status in ["ERROR", "", "FAILED"] {
        let mut output = fixture("cli_usage");
        output["status"] = json!(status);
        let view = from_output(output).await;
        assert_eq!(error_code(&view), Some(ErrorCode::EndpointChanged), "{status:?}");
        assert!(view.windows.is_empty());
    }
}

#[tokio::test]
async fn output_without_the_command_data_is_an_endpoint_change() {
    let no_buckets = json!({ "groups": [{ "name": "Gemini Models" }] });
    for output in [
        json!({ "status": "SUCCESS", "response": "Gemini Models\tWeekly\t89%" }),
        json!({ "status": "SUCCESS", "command": { "name": "usage" } }),
        json!({ "status": "SUCCESS", "command": { "data": { "groups": [] } } }),
        json!({ "status": "SUCCESS", "command": { "data": no_buckets } }),
        json!({ "command": fixture("cli_usage")["command"] }),
        json!([]),
        json!("text"),
    ] {
        let view = from_output(output.clone()).await;
        assert_eq!(error_code(&view), Some(ErrorCode::EndpointChanged), "{output}");
    }
}

#[tokio::test]
async fn a_pool_the_cli_does_not_list_simply_has_no_windows() {
    let mut output = fixture("cli_usage");
    output["command"]["data"]["groups"].as_array_mut().unwrap().pop();
    let view = from_output(output).await;
    let ids: Vec<_> = view.windows.iter().map(|w| w.id.as_str()).collect();
    assert_eq!(ids, ["gemini:five_hour", "gemini:weekly"]);
}

#[tokio::test]
async fn a_share_outside_zero_to_one_is_unknown_never_clamped() {
    for bad in [json!(7), json!(-0.2), json!("lots"), json!(null)] {
        let mut output = fixture("cli_usage");
        output["command"]["data"]["groups"][1]["buckets"][0]["remaining_fraction"] = bad.clone();
        let view = from_output(output).await;
        let window = view.windows.iter().find(|w| w.id == "claude_gpt:weekly").unwrap();
        assert_eq!(window.used_pct, None, "{bad}");
        assert!(window.resets_at.is_some(), "the reset time is still shown");
        let pools: Vec<_> = view.notices.iter().map(|n| n.params["pool"].as_str()).collect();
        assert_eq!(pools, ["claude_gpt"], "{bad}");
    }
}

#[tokio::test]
async fn a_group_nobody_knows_never_lands_in_a_known_pool() {
    let mut output = fixture("cli_usage");
    output["command"]["data"]["groups"][1]["name"] = json!("Image Models");
    output["command"]["data"]["groups"][1]["description"] = json!("Models: Imagen");
    let view = from_output(output).await;
    let groups: Vec<_> = view.windows.iter().filter_map(|w| w.group.as_deref()).collect();
    assert_eq!(groups, ["gemini", "gemini", "image_models", "image_models"]);
}

#[tokio::test]
async fn windows_whose_reset_has_passed_make_the_read_stale() {
    let mut output = fixture("cli_usage");
    for (group, bucket) in [(0, 0), (0, 1), (1, 0), (1, 1)] {
        output["command"]["data"]["groups"][group]["buckets"][bucket]["reset_time"] =
            json!("2026-10-09T00:00:00Z");
    }
    let view = from_output(output).await;
    assert_eq!(view.state, ViewState::Ready);
    assert!(view.stale);
    assert!(view.windows.iter().all(|w| w.resets_at.is_some_and(|reset| reset < NOW)));
}

#[tokio::test]
async fn nothing_from_the_output_text_reaches_the_view() {
    let mut output = fixture("cli_usage");
    output["response"] = json!("TOKEN-LIKE-ca9f ya29.secret");
    output["conversation_id"] = json!("conv-secret");
    let serialized = serde_json::to_string(&from_output(output).await).unwrap();
    assert!(!serialized.contains("ya29") && !serialized.contains("conv-secret"));
}
