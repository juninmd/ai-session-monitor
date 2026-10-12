use usage_model::ErrorCode;

use super::*;
use crate::fixtures::NOW;
use crate::quota::{Pool, parse_summary};

fn window(
    pool: &Pool,
    cadence: Option<Cadence>,
    used: Option<f64>,
    reset: Option<i64>,
) -> PoolWindow {
    let mut window = PoolWindow::new(pool.clone());
    window.cadence = cadence;
    window.used_pct = used;
    window.resets_at = reset;
    window
}

fn read(windows: Vec<PoolWindow>) -> Read {
    Read { plan: Some("Pro".to_owned()), windows }
}

const FIVE: Option<Cadence> = Some(Cadence::FiveHour);
const WEEK: Option<Cadence> = Some(Cadence::Weekly);
const LATER: Option<i64> = Some(NOW + 3_600);

#[test]
fn a_read_becomes_a_ready_view_with_the_source_marked_unofficial() {
    let view = build(read(vec![window(&Pool::Gemini, FIVE, Some(54.0), LATER)]), NOW);
    assert_eq!(view.id, HarnessId::Antigravity);
    assert_eq!(view.state, ViewState::Ready);
    assert_eq!(view.source, Some(Source { id: "language_server".to_owned(), official: false }));
    assert_eq!(view.read_at, Some(NOW));
    assert_eq!(view.plan.as_deref(), Some("Pro"));
    assert!(!view.stale && view.error.is_none() && view.notices.is_empty());
    assert!(view.facts.is_empty() && view.meta.is_empty());
}

#[test]
fn ids_kinds_groups_and_lengths_follow_the_contract() {
    let summary = parse_summary(&crate::fixtures::json("summary_full"), NOW).unwrap();
    let view = build(crate::quota::merge(None, Some(summary)), NOW);
    let rows: Vec<_> = view
        .windows
        .iter()
        .map(|w| (w.id.as_str(), w.kind, w.group.as_deref(), w.window_minutes, w.used_pct))
        .collect();
    assert_eq!(
        rows,
        [
            ("gemini:five_hour", WindowKind::FiveHour, Some("gemini"), Some(300), Some(54.0)),
            ("gemini:weekly", WindowKind::Weekly, Some("gemini"), Some(10_080), Some(29.0)),
            (
                "claude_gpt:five_hour",
                WindowKind::FiveHour,
                Some("claude_gpt"),
                Some(300),
                Some(39.0)
            ),
            ("claude_gpt:weekly", WindowKind::Weekly, Some("claude_gpt"), Some(10_080), Some(12.0)),
        ]
    );
    assert!(view.windows.iter().all(|w| w.label.is_none() && w.amount.is_none()));
    assert_eq!(view.windows[0].resets_at, Some(1_791_641_520));
}

#[test]
fn a_guessed_cadence_names_the_window_but_gives_no_length() {
    let mut guessed = window(&Pool::Gemini, FIVE, Some(30.0), LATER);
    guessed.inferred = true;
    let view = build(read(vec![guessed]), NOW);
    assert_eq!(view.windows[0].kind, WindowKind::FiveHour);
    assert_eq!(view.windows[0].window_minutes, None);
}

#[test]
fn a_window_without_a_cadence_is_other_and_keeps_the_provider_name() {
    let mut named = window(&Pool::Other("image_models".to_owned()), None, Some(10.0), None);
    named.label = Some("Monthly Limit".to_owned());
    named.slug = Some("monthly".to_owned());
    let view = build(read(vec![named]), NOW);
    let shown = &view.windows[0];
    assert_eq!(shown.id, "image_models:monthly");
    assert_eq!(
        (shown.kind, shown.group.as_deref(), shown.label.as_deref()),
        (WindowKind::Other, Some("image_models"), Some("Monthly Limit"))
    );
    assert_eq!(shown.window_minutes, None);
}

#[test]
fn duplicate_names_get_distinct_ids() {
    let mut first = window(&Pool::Gemini, None, Some(1.0), None);
    first.slug = Some("quota".to_owned());
    let mut second = first.clone();
    second.used_pct = Some(2.0);
    let third = window(&Pool::Gemini, None, Some(3.0), None);
    let view = build(read(vec![first, second, third]), NOW);
    let ids: Vec<&str> = view.windows.iter().map(|w| w.id.as_str()).collect();
    assert_eq!(ids, ["gemini:quota", "gemini:quota_2", "gemini:quota_3"]);
}

#[test]
fn an_unknown_share_stays_null_and_raises_one_notice_per_pool() {
    let view = build(
        read(vec![
            window(&Pool::ClaudeGpt, FIVE, None, LATER),
            window(&Pool::ClaudeGpt, WEEK, None, LATER),
            window(&Pool::Gemini, FIVE, Some(10.0), LATER),
            window(&Pool::Gemini, WEEK, None, LATER),
        ]),
        NOW,
    );
    assert_eq!(view.state, ViewState::Ready);
    let used: Vec<Option<f64>> = view.windows.iter().map(|w| w.used_pct).collect();
    assert_eq!(used, [None, None, Some(10.0), None]);
    let pools: Vec<&str> = view.notices.iter().map(|n| n.params["pool"].as_str()).collect();
    assert_eq!(pools, ["claude_gpt", "gemini"]);
    for notice in &view.notices {
        assert_eq!((notice.code.as_str(), notice.severity), (POOL_UNKNOWN, Severity::Warn));
        assert_eq!(notice.params.len(), 1);
    }
}

#[test]
fn a_view_is_stale_only_when_every_window_has_already_reset() {
    let stale = |resets: &[Option<i64>]| {
        let windows = resets.iter().map(|reset| window(&Pool::Gemini, FIVE, Some(50.0), *reset));
        build(read(windows.collect()), NOW).stale
    };
    assert!(stale(&[Some(NOW - 1)]));
    assert!(stale(&[Some(NOW)]));
    assert!(stale(&[Some(NOW - 9_000), Some(NOW - 5)]));
    assert!(!stale(&[Some(NOW + 1)]));
    assert!(!stale(&[Some(NOW - 1), Some(NOW + 60)]));
    // Not knowing a reset is not the same as having seen it pass.
    assert!(!stale(&[Some(NOW - 1), None]));
    assert!(!stale(&[None]));
}

#[test]
fn stale_data_stays_visible_and_ready() {
    let view = build(read(vec![window(&Pool::Gemini, FIVE, Some(54.0), Some(NOW - 60))]), NOW);
    assert!(view.stale);
    assert_eq!(view.state, ViewState::Ready);
    assert_eq!(view.windows.len(), 1);
    assert_eq!(view.windows[0].used_pct, Some(54.0));
    assert_eq!(view.read_at, Some(NOW));
}

#[test]
fn a_server_without_quota_is_no_data_not_an_error() {
    let view = build(read(Vec::new()), NOW);
    assert_eq!(view.state, ViewState::NoData);
    assert!(view.windows.is_empty() && !view.stale && view.notices.is_empty());
    assert_eq!(view.plan.as_deref(), Some("Pro"));
    assert_eq!(view.read_at, Some(NOW));
    assert!(view.source.is_some_and(|source| !source.official));
    assert_eq!(view.error.map(|error| error.code), None::<ErrorCode>);
}
