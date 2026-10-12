//! The rollout reader on awkward files: huge, cut off, empty of events, multibyte, mixed buckets.
//! Every file is fabricated; the app-server is made unavailable by pointing at no binary.

use usage_model::{Ctx, ErrorCode, ViewState};

use crate::common::rollout::{
    chatter, event, event_with, rate_limits, session_path, write_session,
};
use crate::common::{FIVE_HOUR_RESET, NOW, PATIENT, Rig, WEEKLY_RESET, assert_failure};

const OLD: &str = "2026-10-10T18:00:00.000Z";
const NEWER: &str = "2026-10-10T19:30:00.000Z";
const NAME: &str = "rollout-2026-10-10T18-00-00-a.jsonl";
const DAY: &str = "2026/10/10";

fn five_hour_used(view: &usage_model::HarnessView) -> Option<f64> {
    view.windows.iter().find(|w| w.id == "five_hour").and_then(|w| w.used_pct)
}

#[tokio::test]
async fn the_last_event_of_the_newest_file_wins() {
    let rig = Rig::without_binary();
    let lines = [
        chatter("first"),
        event(OLD, Some((10.0, FIVE_HOUR_RESET)), None),
        chatter("second"),
        event(NEWER, Some((25.0, FIVE_HOUR_RESET)), Some((4.0, WEEKLY_RESET))),
        chatter("third"),
    ];
    write_session(rig.home.path(), DAY, NAME, &lines);
    let view = rig.collect().await;
    assert_eq!(five_hour_used(&view), Some(25.0));
    assert_eq!(view.read_at, Some(1_791_660_600), "dated by the winning event");
}

#[tokio::test]
async fn a_cut_off_last_line_gives_way_to_the_event_before_it() {
    let rig = Rig::without_binary();
    let whole = event(NEWER, Some((99.0, FIVE_HOUR_RESET)), None);
    let lines =
        [event(OLD, Some((10.0, FIVE_HOUR_RESET)), None), whole[..whole.len() / 2].to_owned()];
    let path = write_session(rig.home.path(), DAY, NAME, &lines);
    // The writer was interrupted mid-line: no closing newline either.
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::write(&path, text.trim_end()).unwrap();
    assert_eq!(five_hour_used(&rig.collect().await), Some(10.0));
}

#[tokio::test]
async fn files_without_any_rate_limit_event_are_not_data() {
    let rig = Rig::without_binary();
    let without = rate_limits(None, None);
    let lines = [
        chatter("hello"),
        event_with(OLD, serde_json::Value::Null),
        event_with(NEWER, without),
        r#"{"type":"session_meta","payload":{"id":"fabricated"}}"#.to_owned(),
    ];
    write_session(rig.home.path(), DAY, NAME, &lines);
    let view = rig.collect().await;
    assert_failure(&view, ErrorCode::BinaryNotFound, false);
}

#[tokio::test]
async fn a_newer_file_without_events_does_not_hide_an_older_one() {
    let rig = Rig::without_binary();
    let old = write_session(
        rig.home.path(),
        "2026/10/09",
        NAME,
        &[event(OLD, Some((12.0, FIVE_HOUR_RESET)), None)],
    );
    let new = write_session(
        rig.home.path(),
        DAY,
        "rollout-2026-10-10T19-00-00-b.jsonl",
        &[chatter("only talk")],
    );
    crate::common::rollout::set_modified(&old, 1_791_600_000);
    crate::common::rollout::set_modified(&new, 1_791_660_000);
    assert_eq!(five_hour_used(&rig.collect().await), Some(12.0));
}

#[tokio::test]
async fn a_resumed_session_beats_a_newer_named_one() {
    let rig = Rig::without_binary();
    let resumed = write_session(
        rig.home.path(),
        "2026/09/01",
        NAME,
        &[event(OLD, Some((10.0, FIVE_HOUR_RESET)), None)],
    );
    let fresher_name = write_session(
        rig.home.path(),
        DAY,
        "rollout-2026-10-10T19-00-00-b.jsonl",
        &[event(OLD, Some((50.0, FIVE_HOUR_RESET)), None)],
    );
    crate::common::rollout::set_modified(&resumed, 1_791_661_000);
    crate::common::rollout::set_modified(&fresher_name, 1_791_650_000);
    assert_eq!(five_hour_used(&rig.collect().await), Some(10.0));
}

#[tokio::test]
async fn the_main_bucket_is_preferred_over_a_model_specific_one() {
    let rig = Rig::without_binary();
    let mut other = rate_limits(Some((77.0, FIVE_HOUR_RESET)), None);
    other["limit_id"] = serde_json::json!("codex_other");
    let lines = [event(OLD, Some((10.0, FIVE_HOUR_RESET)), None), event_with(NEWER, other)];
    write_session(rig.home.path(), DAY, NAME, &lines);
    assert_eq!(five_hour_used(&rig.collect().await), Some(10.0));
}

#[tokio::test]
async fn unicode_around_the_event_is_harmless() {
    let rig = Rig::without_binary();
    let lines = [
        chatter("ação 🔑 日本語 עברית \u{202e}reversed\u{202c} \u{0}nul"),
        event(NEWER, Some((25.0, FIVE_HOUR_RESET)), None),
        chatter("última linha com acentuação — e emoji 🚀"),
    ];
    write_session(rig.home.path(), DAY, NAME, &lines);
    assert_eq!(five_hour_used(&rig.collect().await), Some(25.0));
}

#[tokio::test]
async fn a_huge_file_is_read_from_its_tail_only() {
    let filler = chatter(&"x".repeat(900));
    let padding = || vec![filler.clone(); 3_000]; // about 2.8 MB, far beyond the 512 KiB tail
    // An event at the end of a huge file is found.
    let tail = Rig::without_binary();
    let mut lines = padding();
    lines.push(event(NEWER, Some((25.0, FIVE_HOUR_RESET)), None));
    let path = write_session(tail.home.path(), DAY, NAME, &lines);
    assert!(std::fs::metadata(&path).unwrap().len() > 2_000_000);
    assert_eq!(five_hour_used(&tail.collect().await), Some(25.0));

    // An event only at the start is out of reach: proof that the whole file is not scanned.
    let head = Rig::without_binary();
    let mut lines = vec![event(OLD, Some((10.0, FIVE_HOUR_RESET)), None)];
    lines.extend(padding());
    write_session(head.home.path(), DAY, NAME, &lines);
    let view = head.collect().await;
    assert_failure(&view, ErrorCode::BinaryNotFound, false);
}

#[tokio::test]
async fn only_the_sessions_directory_is_read() {
    let rig = Rig::without_binary();
    let line = event(NEWER, Some((25.0, FIVE_HOUR_RESET)), None);
    let home = rig.home.path();
    for stray in [
        home.join(".codex").join("rollout-top.jsonl"),
        home.join(".codex").join("archived").join("rollout-old.jsonl"),
        home.join("sessions").join("rollout-no-dot-codex.jsonl"),
        home.join(".codex").join("sessions").join("notes.jsonl"),
    ] {
        std::fs::create_dir_all(stray.parent().unwrap()).unwrap();
        std::fs::write(stray, format!("{line}\n")).unwrap();
    }
    assert_failure(&rig.collect().await, ErrorCode::BinaryNotFound, false);
}

#[tokio::test]
async fn a_very_deep_tree_is_not_walked_to_the_bottom() {
    let rig = Rig::without_binary();
    let deep = "a/b/c/d/e/f/g/h/i/j";
    write_session(
        rig.home.path(),
        deep,
        NAME,
        &[event(NEWER, Some((25.0, FIVE_HOUR_RESET)), None)],
    );
    assert_failure(&rig.collect().await, ErrorCode::BinaryNotFound, false);
    assert!(session_path(rig.home.path(), deep, NAME).exists());
}

#[tokio::test]
async fn an_event_without_a_timestamp_is_dated_by_the_files_modification_time() {
    let rig = Rig::without_binary();
    let mut undated: serde_json::Value =
        serde_json::from_str(&event(NEWER, Some((25.0, FIVE_HOUR_RESET)), None)).unwrap();
    undated.as_object_mut().unwrap().remove("timestamp");
    let path = write_session(rig.home.path(), DAY, NAME, &[undated.to_string()]);
    crate::common::rollout::set_modified(&path, 1_791_600_000);
    let view = rig.collect().await;
    assert_eq!(view.state, ViewState::Ready);
    assert_eq!(view.read_at, Some(1_791_600_000));
}

#[tokio::test]
async fn a_home_directory_that_is_not_absolute_reads_nothing() {
    // An empty or relative home would otherwise point at the directory the app runs from.
    let rig = Rig::without_binary();
    for home in ["", "relative"] {
        let ctx = Ctx::new(NOW, "data", home);
        let view = collector_codex::collect_with_budget(&rig.settings(), &ctx, PATIENT).await;
        assert_failure(&view, ErrorCode::BinaryNotFound, false);
    }
}
