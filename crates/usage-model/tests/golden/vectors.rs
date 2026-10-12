//! Shared status vectors: the pace and summary rules must agree with `contract/status-vectors.json`.

use serde::Deserialize;
use usage_model::status::{Pace, Status, pace_of, status_of, summarize};
use usage_model::view::WindowKind;
use usage_model::{HarnessId, HarnessView, UsageWindow, ViewState};

use super::read_json;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Vectors {
    now: i64,
    windows: Vec<WindowCase>,
    summaries: Vec<SummaryCase>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VectorWindow {
    #[serde(default)]
    id: String,
    window_minutes: Option<u32>,
    used_pct: Option<f64>,
    resets_in_min: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct WindowCase {
    name: String,
    #[serde(flatten)]
    window: VectorWindow,
    warn_at: f64,
    expected: ExpectedPace,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExpectedPace {
    pace: String,
    runway_min: Option<f64>,
    early_min: Option<f64>,
    status: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SummaryCase {
    name: String,
    state: ViewState,
    stale: bool,
    warn_at: f64,
    windows: Vec<VectorWindow>,
    expected: Option<ExpectedSummary>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExpectedSummary {
    window_id: String,
    available_pct: f64,
    status: String,
}

fn build(v: &VectorWindow, now: i64) -> UsageWindow {
    let mut w = UsageWindow::new(if v.id.is_empty() { "w" } else { &v.id }, WindowKind::FiveHour);
    w.window_minutes = v.window_minutes;
    w.used_pct = v.used_pct;
    w.resets_at = v.resets_in_min.map(|m| now + m * 60);
    w
}

fn status_name(status: Status) -> &'static str {
    match status {
        Status::Ok => "ok",
        Status::Warn => "warn",
        Status::Crit => "crit",
        Status::Out => "out",
    }
}

fn close(actual: Option<f64>, expected: Option<f64>, what: &str) {
    match (actual, expected) {
        (None, None) => {}
        (Some(a), Some(e)) => assert!((a - e).abs() < 1e-6, "{what}: {a} != {e}"),
        other => panic!("{what}: {other:?}"),
    }
}

#[test]
fn status_vectors_hold() {
    let vectors: Vectors = serde_json::from_value(read_json("status-vectors.json"))
        .unwrap_or_else(|err| panic!("status-vectors.json: {err}"));
    for case in &vectors.windows {
        let w = build(&case.window, vectors.now);
        let (pace, runway, early) = match pace_of(&w, vectors.now) {
            Pace::Unknown => ("unknown", None, None),
            Pace::Early => ("early", None, None),
            Pace::Out => ("out", None, None),
            Pace::Fine { runway_min } => ("fine", Some(runway_min), None),
            Pace::Risk { runway_min, early_min } => ("risk", Some(runway_min), Some(early_min)),
        };
        assert_eq!(pace, case.expected.pace, "{}: pace", case.name);
        close(runway, case.expected.runway_min, &format!("{}: runway", case.name));
        close(early, case.expected.early_min, &format!("{}: early", case.name));
        let status = status_of(&w, vectors.now, case.warn_at).map(status_name);
        assert_eq!(status, case.expected.status.as_deref(), "{}: status", case.name);
    }
    for case in &vectors.summaries {
        let mut view = HarnessView::with_state(HarnessId::Claude, case.state);
        view.stale = case.stale;
        view.windows = case.windows.iter().map(|w| build(w, vectors.now)).collect();
        let got = summarize(&view, vectors.now, case.warn_at);
        match (&got, &case.expected) {
            (None, None) => {}
            (Some(got), Some(want)) => {
                assert_eq!(got.window_id, want.window_id, "{}: window", case.name);
                close(Some(got.available_pct), Some(want.available_pct), &case.name);
                assert_eq!(status_name(got.status), want.status, "{}: status", case.name);
            }
            other => panic!("{}: {other:?}", case.name),
        }
    }
}
