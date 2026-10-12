//! The static, fabricated session files under `tests/fixtures/rollout`, copied into a temporary
//! home. Their shape follows the openai/codex sources; no real session was used to write them.

use std::fs;
use std::path::Path;

use usage_model::{ErrorCode, Fact, Severity, Source, ViewState};

use crate::common::rollout::session_path;
use crate::common::{FIVE_HOUR_RESET, NOW, Rig, WEEKLY_RESET, assert_failure};

/// A rig without a binary (so the rollout is the only source) holding `fixture` as its newest
/// session.
fn rig_with(fixture: &str) -> Rig {
    let rig = Rig::without_binary();
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/rollout").join(fixture);
    let target =
        session_path(rig.home.path(), "2026/10/10", "rollout-2026-10-10T19-41-07-fixture.jsonl");
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::copy(source, target).unwrap();
    rig
}

fn used(view: &usage_model::HarnessView) -> Vec<(&str, Option<f64>)> {
    view.windows.iter().map(|w| (w.id.as_str(), w.used_pct)).collect()
}

#[tokio::test]
async fn a_session_with_several_token_counts_is_read_from_its_last_one() {
    let view = rig_with("plus-session.jsonl").collect().await;
    assert_eq!(view.state, ViewState::Ready, "{view:?}");
    assert_eq!(view.source, Some(Source { id: "rollout".into(), official: false }));
    // 19:41:21 on the fixture's clock, 1119 s before NOW.
    assert_eq!(view.read_at, Some(NOW - 1_119));
    assert_eq!(used(&view), [("five_hour", Some(26.5)), ("weekly", Some(5.5))]);
    assert_eq!(view.windows[0].resets_at, Some(FIVE_HOUR_RESET));
    assert_eq!(view.windows[1].resets_at, Some(WEEKLY_RESET));
    assert_eq!(view.plan.as_deref(), Some("Plus"));
    let credits = Fact {
        id: "credits_balance".into(),
        number: Some(7.25),
        text: None,
        unit: Some("credits".into()),
    };
    assert_eq!(view.facts, [credits]);
    assert!(!view.stale);
    let [notice] = view.notices.as_slice() else { panic!("{:?}", view.notices) };
    assert_eq!((notice.code.as_str(), notice.severity), ("codex_rollout_fallback", Severity::Info));
}

#[tokio::test]
async fn the_conversation_in_a_session_never_reaches_the_view() {
    let view = rig_with("plus-session.jsonl").collect().await;
    let text = format!("{}{view:?}", serde_json::to_string(&view).unwrap());
    for needle in ["fixture-prompt", "fixture-answer", "fabricated", "cli_version", "originator"] {
        assert!(!text.contains(needle), "{needle}: {text}");
    }
}

#[tokio::test]
async fn a_session_cut_off_mid_line_is_read_up_to_the_last_whole_event() {
    let view = rig_with("truncated.jsonl").collect().await;
    assert_eq!(view.state, ViewState::Ready, "{view:?}");
    assert_eq!(used(&view), [("five_hour", Some(11.0))]);
    assert_eq!(view.read_at, Some(NOW - 600), "dated by the whole event, not the cut one");
}

#[tokio::test]
async fn multibyte_text_around_the_event_changes_nothing() {
    let view = rig_with("unicode.jsonl").collect().await;
    assert_eq!(view.state, ViewState::Ready, "{view:?}");
    assert_eq!(used(&view), [("weekly", Some(12.0))], "a plan without a primary window");
    assert_eq!(view.plan.as_deref(), Some("Pro"));
    assert_eq!(view.read_at, Some(NOW - 1_795));
}

#[tokio::test]
async fn a_session_that_never_reported_limits_is_no_data() {
    let view = rig_with("no-limits.jsonl").collect().await;
    assert_failure(&view, ErrorCode::BinaryNotFound, false);
}
