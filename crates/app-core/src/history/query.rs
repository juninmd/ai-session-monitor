//! Reading the ledger for the UI: newest epochs per window, thinned to a drawable size.

use usage_model::history::MAX_POINTS_PER_EPOCH;
use usage_model::{HarnessHistory, HarnessId, HistoryEpoch, HistoryPoint};

use super::ledger::{Ledger, Series};

impl Ledger {
    /// At most `limit` newest epochs of each window, grouped by window id, oldest epoch first.
    pub(super) fn epochs(&self, harness: HarnessId, limit: usize) -> HarnessHistory {
        let mut mine: Vec<&Series> =
            self.series.iter().filter(|series| series.harness == harness).collect();
        mine.sort_by(|a, b| (&a.window, a.resets_at).cmp(&(&b.window, b.resets_at)));
        let mut epochs = Vec::new();
        let mut start = 0;
        while start < mine.len() {
            let end = mine[start..].iter().position(|s| s.window != mine[start].window);
            let end = end.map_or(mine.len(), |len| start + len);
            let keep_from = end.saturating_sub(limit).max(start);
            epochs.extend(mine[keep_from..end].iter().map(|series| epoch_of(series)));
            start = end;
        }
        HarnessHistory { harness, epochs }
    }
}

fn epoch_of(series: &Series) -> HistoryEpoch {
    HistoryEpoch {
        window_id: series.window.clone(),
        resets_at: series.resets_at,
        points: thin(&series.points, MAX_POINTS_PER_EPOCH),
    }
}

/// Keeps the first and last reading and the highest one of each slice in between, so the peak a
/// summary quotes is never thinned away. Usage only rises inside a window, so the max is the shape.
pub(super) fn thin(points: &[(i64, f64)], max: usize) -> Vec<HistoryPoint> {
    let as_point = |&(t, used_pct): &(i64, f64)| HistoryPoint { t, used_pct };
    if points.len() <= max || max < 3 {
        return points.iter().map(as_point).collect();
    }
    let middle = &points[1..points.len() - 1];
    let buckets = max - 2;
    let mut out = Vec::with_capacity(max);
    out.push(as_point(&points[0]));
    for bucket in 0..buckets {
        let from = bucket * middle.len() / buckets;
        let to = (bucket + 1) * middle.len() / buckets;
        let peak = middle[from..to].iter().max_by(|a, b| a.1.total_cmp(&b.1));
        out.extend(peak.map(as_point));
    }
    out.push(as_point(&points[points.len() - 1]));
    out
}
