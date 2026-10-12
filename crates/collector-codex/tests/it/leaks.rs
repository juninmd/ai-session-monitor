//! Nothing the server or a session file says may reach a view: no email, no path, no error text,
//! no conversation. The fake and the fixtures are seeded with `sentinel` strings to look for.

use usage_model::HarnessView;

use crate::common::rollout::{self, chatter, event_with};
use crate::common::{FIVE_HOUR_RESET, Rig};

const SENTINELS: [&str; 10] = [
    "sentinel-person",
    "example.invalid",
    "sentinel-user-agent",
    "sentinel-codex-home",
    "sentinel-backend-body",
    "sentinel-handshake-text",
    "sentinel-garbage",
    "chatgpt.invalid",
    "fake-app-server",
    "fake_app_server",
];

/// Where a view could be inspected: its JSON (what the UI gets) and its debug text.
fn everything_in(view: &HarnessView) -> String {
    format!("{}\n{view:?}", serde_json::to_string(view).unwrap())
}

/// The last component of each temporary directory is random and unique, so it survives the
/// escaping of backslashes in JSON where a whole path would not match.
fn own_names(rig: &Rig) -> Vec<String> {
    let last = |path: &std::path::Path| path.file_name().unwrap().to_string_lossy().to_lowercase();
    vec![last(rig.bin.path()), last(rig.home.path()), last(&rig.exe)]
}

fn assert_clean(view: &HarnessView, rig: &Rig, what: &str) {
    let text = everything_in(view).to_lowercase();
    let own = own_names(rig);
    for needle in SENTINELS.iter().copied().chain(own.iter().map(String::as_str)) {
        assert!(!text.contains(needle), "{what}: the view contains {needle:?}: {text}");
    }
    assert!(view.meta.is_empty(), "{what}: {:?}", view.meta);
}

#[tokio::test]
async fn nothing_the_server_says_is_copied_into_a_view() {
    let modes = [
        "ok",
        "plus_only",
        "api_key",
        "noisy",
        "not_signed_in",
        "bad_handshake",
        "garbage_line",
        "bad_shape",
        "bad_numbers",
        "exits_midway",
        "backend_401",
        "backend_403",
        "backend_429",
        "backend_500",
        "backend_418",
        "offline",
    ];
    for mode in modes {
        let rig = Rig::new(mode);
        let view = rig.collect().await;
        assert_clean(&view, &rig, mode);
    }
}

#[tokio::test]
async fn nothing_from_a_session_file_but_the_limits_is_kept() {
    let rig = Rig::without_binary();
    let mut limits = rollout::rate_limits(Some((31.0, FIVE_HOUR_RESET)), None);
    limits["note"] = serde_json::json!("sentinel-extra-field");
    limits["plan_type"] = serde_json::json!("plus");
    let mut event =
        serde_json::from_str::<serde_json::Value>(&event_with("2026-10-10T19:59:00.000Z", limits))
            .unwrap();
    event["payload"]["info"] =
        serde_json::json!({"last_message": "sentinel-prompt-text sk-sentinel-not-a-key"});
    let lines = [chatter("sentinel-prompt-text sk-sentinel-not-a-key"), event.to_string()];
    rollout::write_session(rig.home.path(), "2026/10/10", "rollout-a.jsonl", &lines);

    let view = rig.collect().await;
    assert_eq!(view.windows.len(), 1, "the fixture must be read for this test to mean anything");
    let text = everything_in(&view);
    for needle in ["sentinel-prompt-text", "sk-sentinel", "sentinel-extra-field"] {
        assert!(!text.contains(needle), "the view contains {needle:?}: {text}");
    }
    assert_clean(&view, &rig, "rollout");
}

#[tokio::test]
async fn a_hostile_plan_name_is_not_shown() {
    let rig = Rig::without_binary();
    let mut limits = rollout::rate_limits(Some((31.0, FIVE_HOUR_RESET)), None);
    limits["plan_type"] = serde_json::json!("<script>alert(1)</script> sentinel-plan");
    let line = event_with("2026-10-10T19:59:00.000Z", limits);
    rollout::write_session(rig.home.path(), "2026/10/10", "rollout-a.jsonl", &[line]);
    let view = rig.collect().await;
    assert_eq!(view.windows.len(), 1);
    assert_eq!(view.plan, None, "a plan name that is not a plain word is dropped");
}

#[test]
fn declared_notice_codes_are_the_documented_ones() {
    let docs = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/ipc.md");
    let docs = std::fs::read_to_string(docs).unwrap();
    assert_eq!(collector_codex::NOTICE_CODES, ["codex_rollout_fallback", "codex_no_limits"]);
    for code in collector_codex::NOTICE_CODES {
        let row = docs.lines().find(|line| line.contains(&format!("`{code}`")));
        assert!(row.is_some(), "{code} is missing from the notice catalog in docs/ipc.md");
    }
}
