//! Assembles the final view from what the remote and local reads produced.

use usage_model::{
    Fact, HarnessError, HarnessId, HarnessView, Notice, Severity, Source, UsageWindow, ViewState,
};

use crate::local::{self, Facts, Outcome};
use crate::notice;
use crate::remote::Parsed;

const ID: HarnessId = HarnessId::Opencode;

pub(crate) fn compose(
    remote: Option<Result<Parsed, HarnessError>>,
    local: Option<Outcome>,
    now_secs: i64,
) -> HarnessView {
    match (remote, local) {
        // A failed remote read hides the local numbers: the UI shows the error, not partial data.
        (Some(Err(error)), _) => HarnessView::error(ID, error),
        (Some(Ok(parsed)), local) => with_windows(parsed, local, now_secs),
        (None, Some(outcome)) => local_only(outcome),
        (None, None) => HarnessView::needs_setup(ID),
    }
}

fn source(id: &str) -> Option<Source> {
    Some(Source { id: id.to_owned(), official: false })
}

/// Windows from the Go endpoint, with the local estimate attached when it was read as well.
fn with_windows(parsed: Parsed, local: Option<Outcome>, now_secs: i64) -> HarnessView {
    let mut view = HarnessView::with_state(ID, ViewState::Ready);
    view.source = source("go_usage");
    view.read_at = Some(now_secs);
    view.stale = all_reset(&parsed.windows, now_secs);
    view.windows = parsed.windows;
    view.notices = parsed.notices;
    match local {
        Some(Outcome::Facts(facts)) => {
            view.facts = facts_of(&facts);
            view.notices.push(estimate_notice());
            view.meta.insert("local_estimate".to_owned(), "true".to_owned());
        }
        Some(Outcome::Unreadable(reason)) => view.notices.push(unreadable_notice(reason)),
        None => {}
    }
    view
}

/// The local estimate alone: facts only, no windows. Without usable facts there is nothing to
/// show yet, so the view is `NoData` and the notices say why.
fn local_only(outcome: Outcome) -> HarnessView {
    let (state, read_at, facts, extra) = match outcome {
        Outcome::Facts(found) => {
            (ViewState::Ready, found.read_at, facts_of(&found), estimate_notice())
        }
        Outcome::Unreadable(reason) => {
            (ViewState::NoData, None, Vec::new(), unreadable_notice(reason))
        }
    };
    let mut view = HarnessView::with_state(ID, state);
    view.source = source("local_db");
    view.read_at = read_at;
    view.facts = facts;
    view.notices = vec![notice::make(notice::NO_KEY, Severity::Info, &[]), extra];
    view
}

/// Every window has a known reset time that already passed: the numbers are history. One window
/// without a reset time keeps the view fresh, since it cannot be shown to have expired.
fn all_reset(windows: &[UsageWindow], now_secs: i64) -> bool {
    !windows.is_empty()
        && windows.iter().all(|window| window.resets_at.is_some_and(|reset| reset <= now_secs))
}

fn estimate_notice() -> Notice {
    notice::make(notice::LOCAL_ESTIMATE, Severity::Info, &[])
}

fn unreadable_notice(reason: local::Reason) -> Notice {
    notice::make(notice::DB_UNREADABLE, Severity::Warn, &[("reason", reason.as_str())])
}

/// Unknown facts are left out rather than shown as a number.
fn facts_of(facts: &Facts) -> Vec<Fact> {
    let rows = [
        ("local_cost_5h", facts.cost_5h, "usd"),
        ("local_cost_7d", facts.cost_7d, "usd"),
        ("local_cost_30d", facts.cost_30d, "usd"),
        ("local_tokens_5h", facts.tokens_5h, "tokens"),
        ("local_tokens_7d", facts.tokens_7d, "tokens"),
    ];
    rows.into_iter()
        .filter_map(|(id, number, unit)| {
            let number = number?;
            Some(Fact {
                id: id.to_owned(),
                number: Some(number),
                text: None,
                unit: Some(unit.to_owned()),
            })
        })
        .collect()
}
