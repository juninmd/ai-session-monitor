//! A read of the language server as the view the UI renders.

use std::collections::{BTreeMap, BTreeSet};

use usage_model::{
    HarnessId, HarnessView, Notice, Severity, Source, UsageWindow, ViewState, WindowKind,
};

use crate::quota::{Cadence, PoolWindow, Read};

/// Notice for a pool whose share the server did not state; the window shows as unknown, not 0 %.
pub(crate) const POOL_UNKNOWN: &str = "antigravity_pool_unknown";

/// Where a read came from; the id names the source badge in the UI.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Origin {
    LanguageServer,
    Cli,
}

impl Origin {
    const fn id(self) -> &'static str {
        match self {
            Self::LanguageServer => "language_server",
            Self::Cli => "agy_cli",
        }
    }
}

const FIVE_HOUR_MINUTES: u32 = 300;
const WEEKLY_MINUTES: u32 = 10_080;

/// `now` is when the response was produced, so it is also when the data is from.
pub(crate) fn build(read: Read, origin: Origin, now: i64) -> HarnessView {
    let windows = windows_of(&read.windows);
    let state = if windows.is_empty() { ViewState::NoData } else { ViewState::Ready };
    let mut view = HarnessView::with_state(HarnessId::Antigravity, state);
    view.source = Some(Source { id: origin.id().to_owned(), official: false });
    view.read_at = Some(now);
    view.plan = read.plan;
    // The server can keep serving last cycle's numbers after every window has reset.
    view.stale = !windows.is_empty()
        && windows.iter().all(|window| window.resets_at.is_some_and(|reset| reset <= now));
    view.notices = notices_of(&read.windows);
    view.windows = windows;
    view
}

fn windows_of(windows: &[PoolWindow]) -> Vec<UsageWindow> {
    let mut taken = BTreeSet::new();
    windows
        .iter()
        .map(|source| {
            let (kind, minutes, part) = match source.cadence {
                Some(Cadence::FiveHour) => {
                    (WindowKind::FiveHour, Some(FIVE_HOUR_MINUTES), "five_hour".to_owned())
                }
                Some(Cadence::Weekly) => {
                    (WindowKind::Weekly, Some(WEEKLY_MINUTES), "weekly".to_owned())
                }
                None => (
                    WindowKind::Other,
                    None,
                    source.slug.clone().unwrap_or_else(|| "quota".to_owned()),
                ),
            };
            let id = unique(&mut taken, format!("{}:{part}", source.pool.id()));
            let mut window = UsageWindow::new(id, kind);
            window.group = Some(source.pool.id().to_owned());
            window.label.clone_from(&source.label);
            // A guessed cadence is not a stated length: it must not steer the pace estimate.
            window.window_minutes = if source.inferred { None } else { minutes };
            window.used_pct = source.used_pct;
            window.resets_at = source.resets_at;
            window
        })
        .collect()
}

/// Ids stay unique inside the harness even when the server lists a name twice.
fn unique(taken: &mut BTreeSet<String>, id: String) -> String {
    let mut candidate = id.clone();
    let mut copy = 1;
    while !taken.insert(candidate.clone()) {
        copy += 1;
        candidate = format!("{id}_{copy}");
    }
    candidate
}

/// One notice per pool with an unknown window, in the order the pools appear.
fn notices_of(windows: &[PoolWindow]) -> Vec<Notice> {
    let mut pools: Vec<&str> = Vec::new();
    for window in windows.iter().filter(|window| window.used_pct.is_none()) {
        let pool = window.pool.id();
        if !pools.contains(&pool) {
            pools.push(pool);
        }
    }
    pools
        .into_iter()
        .map(|pool| Notice {
            code: POOL_UNKNOWN.to_owned(),
            severity: Severity::Warn,
            params: BTreeMap::from([("pool".to_owned(), pool.to_owned())]),
        })
        .collect()
}

#[cfg(test)]
mod tests;
