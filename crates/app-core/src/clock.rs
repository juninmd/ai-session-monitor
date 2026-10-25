//! The wall-clock seam. Collectors, snapshots and notifications take "now" from here instead of
//! the OS, so tests drive time and `Ctx::now_secs` always agrees with what the hub shows.

use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Local;

pub trait Clock: Send + Sync {
    /// Seconds since the Unix epoch (UTC).
    fn now_secs(&self) -> i64;

    /// Seconds the local zone is ahead of UTC right now. UTC unless a clock says otherwise.
    fn utc_offset_secs(&self) -> i32 {
        0
    }
}

/// The operating system clock.
#[derive(Clone, Copy, Debug, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_secs(&self) -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| i64::try_from(elapsed.as_secs()).unwrap_or(i64::MAX))
    }

    fn utc_offset_secs(&self) -> i32 {
        Local::now().offset().local_minus_utc()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_clock_is_after_2025() {
        // 2025-01-01T00:00:00Z; guards against the 0 fallback silently becoming the norm.
        assert!(SystemClock.now_secs() > 1_735_689_600);
    }
}
