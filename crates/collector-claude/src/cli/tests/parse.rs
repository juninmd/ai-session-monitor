//! What counts as a limit line, and what never gets past the parser.

use serde_json::json;
use usage_model::ErrorCode;

use super::*;
use crate::cli::parse::envelope as parse;
use crate::cli::{ModelRow, windows};

fn rows(result: &str) -> Vec<ModelRow> {
    parse(&envelope(result)).expect("parses")
}

fn row(label: &str, used_pct: Option<f64>, same_reset: bool) -> ModelRow {
    ModelRow { label: label.to_owned(), used_pct, same_reset }
}

fn week(label: &str, number: &str, reset: &str) -> String {
    format!("Current week ({label}): {number}% used · resets {reset}")
}

fn block(lines: &[String]) -> String {
    format!("Header\n\n{}\n\nTail\n", lines.join("\n"))
}

#[test]
fn fable_opus_and_sonnet_come_out_with_their_share_and_whether_the_reset_matches() {
    assert_eq!(
        rows(REPORT),
        [
            row("Fable", Some(0.0), true),
            row("Opus", Some(12.5), true),
            row("Sonnet", Some(34.0), false),
        ]
    );
}

#[test]
fn a_plan_with_only_the_all_models_cap_has_no_per_model_rows_and_is_not_an_error() {
    let text = block(&[
        "Current session: 1% used · resets later".into(),
        week("all models", "78", "Oct 17"),
    ]);
    assert_eq!(rows(&text), []);
}

#[test]
fn a_percentage_outside_zero_to_one_hundred_is_unknown_not_clamped() {
    let lines = [
        week("A", "100.01", "x"),
        week("B", "250", "x"),
        week("C", "100", "x"),
        week("D", "0.25", "x"),
    ];
    let got = rows(&block(&lines));
    let pct: Vec<_> = got.iter().map(|r| r.used_pct).collect();
    assert_eq!(pct, [None, None, Some(100.0), Some(0.25)]);
}

#[test]
fn a_line_that_is_not_the_exact_shape_is_ignored() {
    let lines = [
        week("Good", "5", "Oct 17"),
        "Current week (Neg): -5% used · resets x".to_owned(),
        "Current week (Exp): 1e2% used · resets x".to_owned(),
        "Current week (Nan): nan% used · resets x".to_owned(),
        "Current week (Empty): % used · resets x".to_owned(),
        "Current week (NoReset): 5% used · resets ".to_owned(),
        "Current week (Left): 5% left · resets x".to_owned(),
        "Current week Missing: 5% used · resets x".to_owned(),
        "Current week (Dots): 1.2.3% used · resets x".to_owned(),
    ];
    let got = rows(&block(&lines));
    let shown: Vec<_> = got.iter().map(|r| (r.label.as_str(), r.used_pct)).collect();
    // A digits-and-dots token that is not a number is a line with an unknown share.
    assert_eq!(shown, [("Good", Some(5.0)), ("Dots", None)]);
}

#[test]
fn text_after_the_limits_block_is_never_read() {
    let text = format!(
        "{}\n\nWhat's contributing to your limits usage?\n{}\n{}",
        [week("all models", "78", "r"), week("Opus", "1", "r")].join("\n"),
        week("Evil", "99", "r"),
        "Current session: 99% used · resets r"
    );
    assert_eq!(rows(&text), [row("Opus", Some(1.0), true)]);
}

#[test]
fn labels_are_cleaned_and_bounded() {
    let long = "X".repeat(200);
    let got = rows(&block(&[week(&long, "1", "r"), week("Fa\u{7}ble", "2", "r")]));
    assert_eq!(got[0].label.chars().count(), 40);
    assert_eq!(got[1].label, "Fable");
}

#[test]
fn no_more_than_eight_rows() {
    let lines: Vec<_> = (0..20).map(|n| week(&format!("M{n}"), "1", "r")).collect();
    assert_eq!(rows(&block(&lines)).len(), 8);
}

#[test]
fn an_answer_that_is_not_the_usage_report_is_a_changed_endpoint() {
    let no_limits = "Hello! How can I help you today?";
    let bad = [
        envelope(no_limits),
        envelope(""),
        json!({"local_command": "usage"}),
        json!({"local_command": "usage", "result": 5}),
        json!({"result": REPORT}),
        json!({"local_command": "cost", "result": REPORT}),
        json!({"local_command": 1, "result": REPORT}),
        json!([1, 2]),
        json!("text"),
        json!(null),
    ];
    for body in bad {
        let error = parse(&body).expect_err(&body.to_string());
        assert_eq!((error.code, error.retryable), (ErrorCode::EndpointChanged, false));
    }
}

#[test]
fn the_epoch_of_the_statusline_applies_only_when_the_reset_text_matched() {
    let got = windows(&rows(REPORT), Some(WEEKLY_RESET));
    let by_id = |id: &str| got.iter().find(|w| w.id == id).unwrap();
    assert_eq!(by_id("fable:weekly").resets_at, Some(WEEKLY_RESET));
    assert_eq!(by_id("opus:weekly").resets_at, Some(WEEKLY_RESET));
    assert_eq!(by_id("sonnet:weekly").resets_at, None, "a different reset text is not a time");
    assert!(windows(&rows(REPORT), None).iter().all(|w| w.resets_at.is_none()));
}

#[test]
fn windows_look_exactly_like_the_ones_from_the_oauth_path() {
    let got = windows(&rows(REPORT), Some(WEEKLY_RESET));
    let shown: Vec<_> = got
        .iter()
        .map(|w| (w.id.as_str(), w.group.as_deref(), w.label.as_deref(), w.window_minutes))
        .collect();
    assert_eq!(
        shown,
        [
            ("fable:weekly", Some("fable"), Some("Fable"), Some(10_080)),
            ("opus:weekly", Some("opus"), None, Some(10_080)),
            ("sonnet:weekly", Some("sonnet"), None, Some(10_080)),
        ]
    );
}

#[test]
fn two_lines_for_one_family_keep_the_more_consumed_one() {
    let got = windows(&[row("Opus", Some(10.0), false), row("Opus 4.1", Some(30.0), false)], None);
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].used_pct, Some(30.0));
}
