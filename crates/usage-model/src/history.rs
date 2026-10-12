//! Wire shape of the quota burndown history: the past windows ("epochs") of one harness. Mirrored
//! by Zod in `src/lib/contract/history.ts`; `contract/ipc.json` pins both sides to the same sample.

use serde::{Deserialize, Serialize};

use crate::HarnessId;

/// Newest epochs returned per window, and the largest `limit` a caller may ask for.
pub const MAX_EPOCHS_PER_WINDOW: u8 = 8;
/// Points per epoch on the wire; the store keeps more and thins them on the way out.
pub const MAX_POINTS_PER_EPOCH: usize = 120;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryPoint {
    /// Epoch seconds (UTC) of the reading.
    pub t: i64,
    /// 0..=100; an unknown reading is never stored, so there is no null here.
    pub used_pct: f64,
}

/// One pass of one window, from its first reading to its reset (or to now, for the current one).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEpoch {
    /// Matches `UsageWindow::id` of the same harness.
    pub window_id: String,
    /// Epoch seconds (UTC) at which this pass resets; identifies the epoch.
    pub resets_at: i64,
    /// Oldest first, at most `MAX_POINTS_PER_EPOCH`.
    pub points: Vec<HistoryPoint>,
}

/// Epochs of one harness, ordered by window id then reset time (oldest first).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HarnessHistory {
    pub harness: HarnessId,
    pub epochs: Vec<HistoryEpoch>,
}

impl HarnessHistory {
    pub fn empty(harness: HarnessId) -> Self {
        Self { harness, epochs: Vec::new() }
    }
}
