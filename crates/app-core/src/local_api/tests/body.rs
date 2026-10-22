use serde_json::Value;
use usage_model::{
    ErrorCode, Fact, HarnessError, HarnessId, HarnessView, Notice, Severity, Snapshot, Source,
    UsageWindow, ViewState, WindowKind,
};

use super::super::body::{limits, rfc3339};

const NOW: i64 = 1_791_633_600; // 2026-10-10T12:00:00Z

fn window(id: &str, used: Option<f64>, resets_in_min: Option<i64>) -> UsageWindow {
    let mut window = UsageWindow::new(id, WindowKind::FiveHour);
    window.used_pct = used;
    window.resets_at = resets_in_min.map(|min| NOW + min * 60);
    window.window_minutes = Some(300);
    window.label = Some("label-from-a-remote-service".to_owned());
    window.group = Some("group-from-a-remote-service".to_owned());
    window
}

/// A view carrying everything a collector may attach, secret-looking on purpose.
fn loaded(id: HarnessId, state: ViewState) -> HarnessView {
    let mut view = HarnessView::with_state(id, state);
    view.source = Some(Source { id: "oauth_usage".to_owned(), official: false });
    view.plan = Some("Max 5x".to_owned());
    view.read_at = Some(NOW - 5);
    view.error = Some(
        HarnessError::new(ErrorCode::Network, true).with_param("path", r"C:\Users\ana\token.txt"),
    );
    view.notices = vec![Notice {
        code: "n".to_owned(),
        severity: Severity::Warn,
        params: [("host".to_owned(), "internal.example".to_owned())].into(),
    }];
    view.meta = [("alias".to_owned(), "sk-secret-in-meta".to_owned())].into();
    view.next_attempt_at = Some(NOW + 60);
    view
}

fn render(views: Vec<HarnessView>) -> (String, Value) {
    let snapshot = Snapshot { generated_at: NOW, views };
    let bytes = limits(&snapshot, 25);
    let text = String::from_utf8(bytes).expect("utf-8");
    let json = serde_json::from_str(&text).expect("json");
    (text, json)
}

#[test]
fn the_shape_is_the_documented_one() {
    let mut claude = loaded(HarnessId::Claude, ViewState::Ready);
    claude.windows = vec![window("five_hour", Some(40.0), Some(120)), window("weekly", None, None)];
    claude.facts = vec![Fact {
        id: "rpm".to_owned(),
        number: Some(60.0),
        text: None,
        unit: Some("req".to_owned()),
    }];
    let (_, json) = render(vec![claude]);
    assert_eq!(json["version"], 1);
    assert_eq!(json["generatedAt"], "2026-10-10T12:00:00Z");
    let harness = &json["harnesses"][0];
    assert_eq!(harness["id"], "claude");
    assert_eq!(harness["name"], "Claude Code");
    assert_eq!(harness["state"], "ready");
    assert_eq!(harness["official"], false);
    assert_eq!(harness["plan"], "Max 5x");
    assert_eq!(harness["stale"], false);
    let five = &harness["windows"][0];
    assert_eq!(five["id"], "five_hour");
    assert_eq!(five["kind"], "five_hour");
    assert_eq!(five["usedPct"], 40.0);
    assert_eq!(five["availablePct"], 60.0);
    assert_eq!(five["resetsAt"], "2026-10-10T14:00:00Z");
    assert_eq!(five["status"], "ok");
    assert_eq!(
        harness["facts"][0],
        serde_json::json!({"id":"rpm","number":60.0,"text":null,"unit":"req"})
    );
}

#[test]
fn unknown_is_null_never_zero() {
    let mut view = loaded(HarnessId::Codex, ViewState::Ready);
    view.windows = vec![window("weekly", None, None)];
    let (_, json) = render(vec![view]);
    let window = &json["harnesses"][0]["windows"][0];
    assert_eq!(window["usedPct"], Value::Null);
    assert_eq!(window["availablePct"], Value::Null);
    assert_eq!(window["resetsAt"], Value::Null);
    assert_eq!(window["status"], Value::Null);
}

#[test]
fn an_expired_window_has_no_numbers_and_no_status() {
    let mut view = loaded(HarnessId::Claude, ViewState::Ready);
    view.windows =
        vec![window("five_hour", Some(97.0), Some(-1)), window("weekly", Some(97.0), Some(60))];
    let (_, json) = render(vec![view]);
    let expired = &json["harnesses"][0]["windows"][0];
    assert_eq!(expired["usedPct"], Value::Null);
    assert_eq!(expired["availablePct"], Value::Null);
    assert_eq!(expired["status"], Value::Null);
    assert_eq!(expired["resetsAt"], "2026-10-10T11:59:00Z", "the moment it passed stays visible");
    assert_eq!(json["harnesses"][0]["windows"][1]["status"], "crit");
}

#[test]
fn statuses_follow_the_shared_rules() {
    let mut view = loaded(HarnessId::Claude, ViewState::Ready);
    view.windows = vec![
        window("a", Some(100.0), Some(60)),
        window("b", Some(95.0), Some(60)),
        window("c", Some(80.0), Some(60)),
        window("d", Some(10.0), Some(60)),
    ];
    let (_, json) = render(vec![view]);
    let statuses: Vec<_> = json["harnesses"][0]["windows"]
        .as_array()
        .expect("array")
        .iter()
        .map(|w| w["status"].clone())
        .collect();
    assert_eq!(statuses, ["out", "crit", "warn", "ok"]);
}

#[test]
fn disabled_harnesses_are_left_out_and_the_rest_keep_their_order() {
    let views = vec![
        loaded(HarnessId::Claude, ViewState::Ready),
        HarnessView::disabled(HarnessId::Codex),
        HarnessView::needs_setup(HarnessId::Litellm),
        HarnessView::loading(HarnessId::Kimi),
    ];
    let (_, json) = render(views);
    let ids: Vec<_> = json["harnesses"]
        .as_array()
        .expect("array")
        .iter()
        .map(|h| h["id"].as_str().expect("id"))
        .collect();
    assert_eq!(ids, ["claude", "litellm", "kimi"]);
    assert_eq!(json["harnesses"][1]["state"], "needs_setup");
    assert_eq!(json["harnesses"][1]["official"], Value::Null);
}

const ALLOWED_TOP: &[&str] = &["version", "generatedAt", "harnesses"];
const ALLOWED_HARNESS: &[&str] =
    &["id", "name", "state", "official", "plan", "stale", "windows", "facts"];
const ALLOWED_WINDOW: &[&str] = &["id", "kind", "usedPct", "availablePct", "resetsAt", "status"];
const ALLOWED_FACT: &[&str] = &["id", "number", "text", "unit"];

fn keys(value: &Value) -> Vec<&str> {
    value.as_object().expect("object").keys().map(String::as_str).collect()
}

fn assert_only(value: &Value, allowed: &[&str]) {
    for key in keys(value) {
        assert!(allowed.contains(&key), "unexpected field {key}");
    }
}

#[test]
fn nothing_secret_or_remote_leaves_through_the_body() {
    let mut view = loaded(HarnessId::Litellm, ViewState::Ready);
    view.windows = vec![window("budget", Some(10.0), Some(60))];
    view.facts =
        vec![Fact { id: "rpm".to_owned(), number: None, text: Some("x".to_owned()), unit: None }];
    let mut errored = loaded(HarnessId::Kimi, ViewState::Error);
    errored.windows = vec![window("weekly", Some(10.0), Some(60))];
    let (text, json) = render(vec![view, errored]);
    for needle in [
        "sk-secret-in-meta",
        "token.txt",
        "internal.example",
        "label-from-a-remote-service",
        "group-from-a-remote-service",
        "oauth_usage",
        "alias",
        "notices",
        "meta",
        "params",
        "nextAttemptAt",
        "readAt",
        "windowMinutes",
        "label",
        "group",
        "amount",
    ] {
        assert!(!text.contains(needle), "{needle} leaked: {text}");
    }
    assert_only(&json, ALLOWED_TOP);
    for harness in json["harnesses"].as_array().expect("array") {
        assert_only(harness, ALLOWED_HARNESS);
        for window in harness["windows"].as_array().expect("array") {
            assert_only(window, ALLOWED_WINDOW);
        }
        for fact in harness["facts"].as_array().expect("array") {
            assert_only(fact, ALLOWED_FACT);
        }
    }
}

#[test]
fn rfc3339_matches_the_parser_across_eras() {
    for secs in [
        0,
        -1,
        86_399,
        951_782_400,
        1_709_208_000,
        NOW,
        4_102_444_800,
        -14_182_940,
        -62_167_219_200,
        253_402_300_799,
    ] {
        let text = rfc3339(secs);
        assert_eq!(usage_support::parse_rfc3339_secs(&text), Some(secs), "{text}");
    }
    assert_eq!(rfc3339(NOW), "2026-10-10T12:00:00Z");
    assert_eq!(rfc3339(1_709_164_800), "2024-02-29T00:00:00Z");
}
