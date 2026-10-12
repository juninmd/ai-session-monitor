//! The in-memory record of past windows: pure data and rules, no clock and no disk.

use serde::{Deserialize, Serialize};
use usage_model::{HarnessId, UsageWindow};

pub(super) const VERSION: u32 = 1;
/// Samples closer than this and equal in value add nothing a line chart can show.
pub(super) const SAMPLE_GAP_SECS: i64 = 10 * 60;
pub(super) const RETENTION_SECS: i64 = 35 * 24 * 3600;
pub(super) const MAX_POINTS: usize = 20_000;
/// Providers re-state a reset a few seconds apart between reads; windows are far longer than this.
const EPOCH_TOLERANCE_SECS: i64 = 120;
/// A window id comes from a collector; the store only keeps short, plain ones.
const MAX_ID_CHARS: usize = 64;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct Series {
    pub(super) harness: HarnessId,
    pub(super) window: String,
    pub(super) resets_at: i64,
    /// `(epoch seconds, used percentage)`, oldest first.
    pub(super) points: Vec<(i64, f64)>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct Ledger {
    pub(super) version: u32,
    pub(super) series: Vec<Series>,
}

impl Default for Ledger {
    fn default() -> Self {
        Self { version: VERSION, series: Vec::new() }
    }
}

fn plain_id(id: &str) -> bool {
    !id.is_empty()
        && id.chars().count() <= MAX_ID_CHARS
        && id.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | ':' | '.' | '-'))
}

impl Ledger {
    #[cfg(test)]
    pub(super) fn is_empty(&self) -> bool {
        self.series.is_empty()
    }

    pub(super) fn point_count(&self) -> usize {
        self.series.iter().map(|series| series.points.len()).sum()
    }

    /// Samples every window with a known percentage; unknown is skipped, never stored as 0.
    /// Returns whether anything was added.
    pub(super) fn record(&mut self, harness: HarnessId, windows: &[UsageWindow], now: i64) -> bool {
        let mut added = false;
        for window in windows {
            let (Some(used), Some(resets_at)) = (window.used_pct, window.resets_at) else {
                continue;
            };
            if !used.is_finite() || resets_at <= now || !plain_id(&window.id) {
                continue;
            }
            added |= self.sample(harness, &window.id, resets_at, used.clamp(0.0, 100.0), now);
        }
        added
    }

    fn sample(
        &mut self,
        harness: HarnessId,
        id: &str,
        resets_at: i64,
        used: f64,
        now: i64,
    ) -> bool {
        let used = (used * 10.0).round() / 10.0;
        let found = self.series.iter_mut().rev().find(|series| {
            series.harness == harness
                && series.window == id
                && (series.resets_at - resets_at).abs() <= EPOCH_TOLERANCE_SECS
        });
        let Some(series) = found else {
            self.series.push(Series {
                harness,
                window: id.to_owned(),
                resets_at,
                points: vec![(now, used)],
            });
            return true;
        };
        series.resets_at = resets_at;
        let due = series.points.last().is_none_or(|&(at, last)| {
            now > at && ((used - last).abs() > f64::EPSILON || now - at >= SAMPLE_GAP_SECS)
        });
        if due {
            series.points.push((now, used));
        }
        due
    }

    /// Applies the retention window, then the point cap, oldest first.
    pub(super) fn prune(&mut self, now: i64) {
        let oldest = now - RETENTION_SECS;
        for series in &mut self.series {
            series.points.retain(|&(at, _)| at >= oldest);
        }
        self.series.retain(|series| !series.points.is_empty());
        self.cap_points(MAX_POINTS);
    }

    pub(super) fn cap_points(&mut self, max: usize) {
        let excess = self.point_count().saturating_sub(max);
        if excess == 0 {
            return;
        }
        let mut times: Vec<i64> =
            self.series.iter().flat_map(|s| s.points.iter().map(|p| p.0)).collect();
        times.sort_unstable();
        let cutoff = times[excess - 1];
        for series in &mut self.series {
            series.points.retain(|&(at, _)| at > cutoff);
        }
        self.series.retain(|series| !series.points.is_empty());
    }

    /// Drops what a hand-edited or damaged file could smuggle in; keeps the rest.
    pub(super) fn sanitize(&mut self) {
        self.series.retain(|series| plain_id(&series.window));
        for series in &mut self.series {
            series.points.retain(|&(_, used)| used.is_finite() && (0.0..=100.0).contains(&used));
            series.points.sort_by_key(|&(at, _)| at);
        }
        self.series.retain(|series| !series.points.is_empty());
    }
}
