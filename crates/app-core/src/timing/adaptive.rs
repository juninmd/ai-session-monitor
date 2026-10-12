//! Adaptive pacing: poll a calm source less often, never faster than its base cadence.

use std::time::Duration;

use usage_model::status::is_expired;
use usage_model::{Settings, UsageWindow};

/// A source never waits longer than this just for being calm (unless its base cadence is longer).
const MAX_STRETCH: Duration = Duration::from_secs(30 * 60);
/// A reset this close (or closer) puts the source back on its base cadence.
const NEAR_RESET_SECS: i64 = 10 * 60;
/// Used percentage at or above which a window is worth watching closely.
const HOT_PCT: f64 = 70.0;
/// A threshold alert needs a reading this many points before it, or the alert would come late.
const ALERT_MARGIN: f64 = 10.0;
/// Bounds the multiplication; far above what `MAX_STRETCH` allows for any real cadence.
const MAX_IDLE: u32 = 64;

/// Used percentage from which a window counts as close to its limit: 70, or lower when the user
/// asked for an earlier alert.
pub(crate) fn hot_pct(settings: &Settings) -> f64 {
    match settings.general.alert_thresholds.first() {
        Some(&lowest) => HOT_PCT.min(f64::from(lowest) - ALERT_MARGIN).max(1.0),
        None => HOT_PCT,
    }
}

/// The wait after a healthy read. `base` when anything is close to its limit, close to its reset
/// or unknown; otherwise `base * (2 + idle_ticks)`, held to 30 minutes and to 10 minutes before
/// the nearest reset, and never below `base`. `idle_ticks` counts reads in a row with no change.
pub(crate) fn adaptive_wait(
    base: Duration,
    windows: &[UsageWindow],
    idle_ticks: u32,
    hot_pct: f64,
    now_secs: i64,
) -> Duration {
    let mut ceiling = MAX_STRETCH;
    let mut live = false;
    for window in windows.iter().filter(|w| !is_expired(w, now_secs)) {
        live = true;
        match window.used_pct {
            Some(used) if used < hot_pct => {}
            _ => return base,
        }
        if let Some(reset) = window.resets_at {
            let calm_for = reset.saturating_sub(now_secs).saturating_sub(NEAR_RESET_SECS);
            if calm_for <= 0 {
                return base;
            }
            ceiling = ceiling.min(Duration::from_secs(calm_for.unsigned_abs()));
        }
    }
    if !live {
        return base;
    }
    base.saturating_mul(2 + idle_ticks.min(MAX_IDLE)).min(ceiling).max(base)
}

#[cfg(test)]
mod tests;
