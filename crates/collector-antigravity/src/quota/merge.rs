//! One list of pool windows out of what the two calls said.

use std::collections::BTreeSet;

use super::model::{Cadence, Pool, PoolWindow, Read};

/// Two pools with two windows each is the normal case; this only bounds a hostile response.
const MAX_WINDOWS: usize = 24;

/// Per pool, the summary's windows when any carries a share (it states the cadence, the status
/// only lets it be guessed), else the status' windows when any does, else whatever there is. The
/// plan comes from the status. Pools come out as Gemini, Claude and GPT, then the rest by name;
/// inside a pool the five hour window comes first.
pub(crate) fn merge(status: Option<Read>, summary: Option<Read>) -> Read {
    let plan = [&status, &summary].into_iter().flatten().find_map(|read| read.plan.clone());
    let stated = summary.map(|read| read.windows).unwrap_or_default();
    let guessed = status.map(|read| read.windows).unwrap_or_default();
    let pools: BTreeSet<Pool> = stated.iter().chain(&guessed).map(|w| w.pool.clone()).collect();
    let mut windows = Vec::new();
    for pool in pools {
        windows.extend(tidy(choose(of_pool(&stated, &pool), of_pool(&guessed, &pool))));
    }
    windows.truncate(MAX_WINDOWS);
    Read { plan, windows }
}

fn of_pool(windows: &[PoolWindow], pool: &Pool) -> Vec<PoolWindow> {
    windows.iter().filter(|window| &window.pool == pool).cloned().collect()
}

fn choose(stated: Vec<PoolWindow>, guessed: Vec<PoolWindow>) -> Vec<PoolWindow> {
    if has_share(&stated) || (!has_share(&guessed) && !stated.is_empty()) {
        stated
    } else {
        guessed
    }
}

fn has_share(windows: &[PoolWindow]) -> bool {
    windows.iter().any(|window| window.used_pct.is_some())
}

/// Five hour, weekly, then the rest; one window per known cadence, the most used one.
fn tidy(windows: Vec<PoolWindow>) -> Vec<PoolWindow> {
    let mut kept: Vec<PoolWindow> = Vec::new();
    for window in windows {
        let twin = window.cadence.and_then(|c| kept.iter_mut().find(|k| k.cadence == Some(c)));
        match twin {
            Some(twin) if used(&window) > used(twin) => *twin = window,
            Some(_) => {}
            None => kept.push(window),
        }
    }
    kept.sort_by_key(|window| match window.cadence {
        Some(Cadence::FiveHour) => 0,
        Some(Cadence::Weekly) => 1,
        None => 2,
    });
    kept
}

/// Unknown ranks below every known share, so a known window is never replaced by an unknown one.
fn used(window: &PoolWindow) -> f64 {
    window.used_pct.unwrap_or(-1.0)
}

#[cfg(test)]
mod tests;
