//! Throttle of the CLI source: when it last ran and what it said, in a small file next to the
//! shim's data (collectors are stateless). Nothing secret is stored: labels, percentages and an
//! error code.

use std::path::Path;

use serde::{Deserialize, Serialize};
use usage_model::HarnessError;

use super::{CliReport, ModelRow};
use crate::lenient::{clean_text, percent};

/// A run costs about five seconds of CPU and may fire the user's hooks.
const OK_WAIT_SECS: i64 = 600;
const FAILED_WAIT_SECS: i64 = 1800;
const CACHE_VERSION: u32 = 2;
const MAX_FILE_BYTES: usize = 64 * 1024;
const MAX_ROWS: usize = 8;
const MAX_LABEL_CHARS: usize = 40;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CacheFile {
    v: u32,
    attempted_at: i64,
    outcome: Stored,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum Stored {
    #[serde(rename_all = "camelCase")]
    Ok {
        #[serde(default)]
        session_pct: Option<f64>,
        #[serde(default)]
        weekly_pct: Option<f64>,
        rows: Vec<ModelRow>,
    },
    Failed {
        error: HarnessError,
    },
}

/// The last outcome when it is recent enough to stand in for a new run, dropping a cached `Ok`
/// recorded before `min_attempted_at` (for instance before a statusLine window's `resets_at`).
pub(super) fn fresh_since(
    path: &Path,
    now: i64,
    min_attempted_at: Option<i64>,
) -> Option<Result<CliReport, HarnessError>> {
    let file: CacheFile = usage_support::read_json_capped(path, MAX_FILE_BYTES).ok()?;
    let wait = match file.outcome {
        Stored::Ok { .. } => {
            if min_attempted_at.is_some_and(|r| file.attempted_at < r) {
                return None;
            }
            OK_WAIT_SECS
        }
        Stored::Failed { .. } => FAILED_WAIT_SECS,
    };
    // A timestamp in the future (clock moved back) proves nothing about how long ago it was.
    let age = now.checked_sub(file.attempted_at)?;
    if file.v != CACHE_VERSION || !(0..wait).contains(&age) {
        return None;
    }
    Some(match file.outcome {
        Stored::Ok { session_pct, weekly_pct, rows } => {
            Ok(clean(CliReport { session_pct, weekly_pct, rows }))
        }
        Stored::Failed { error } => Err(HarnessError::new(error.code, error.retryable)),
    })
}

/// Best effort: without the file the CLI is simply run again on the next tick.
pub(super) fn store(path: &Path, now: i64, outcome: &Result<CliReport, HarnessError>) {
    let outcome = match outcome {
        Ok(report) => Stored::Ok {
            session_pct: report.session_pct,
            weekly_pct: report.weekly_pct,
            rows: report.rows.clone(),
        },
        Err(error) => Stored::Failed { error: HarnessError::new(error.code, error.retryable) },
    };
    let file = CacheFile { v: CACHE_VERSION, attempted_at: now, outcome };
    if let Ok(bytes) = serde_json::to_vec(&file) {
        let _ = usage_support::atomic_write(path, &bytes);
    }
}

/// The file is editable by anyone with access to the data directory.
fn clean(report: CliReport) -> CliReport {
    let pct = |val: Option<f64>| percent(val.map(serde_json::Value::from).as_ref(), 100.0);
    let rows = report
        .rows
        .into_iter()
        .filter_map(|row| {
            Some(ModelRow {
                label: clean_text(&row.label, MAX_LABEL_CHARS)?,
                used_pct: pct(row.used_pct),
                same_reset: row.same_reset,
            })
        })
        .take(MAX_ROWS)
        .collect();
    CliReport { session_pct: pct(report.session_pct), weekly_pct: pct(report.weekly_pct), rows }
}
