//! When to poll: the cadence of each harness and how failures stretch it. Pure functions.

use std::time::Duration;

use usage_model::{HarnessId, Settings};

mod adaptive;

pub(crate) use adaptive::{adaptive_wait, hot_pct};

/// One collector call never runs longer than this; it then becomes a `Timeout` error.
pub(crate) const COLLECT_TIMEOUT: Duration = Duration::from_secs(30);
/// A manual refresh is dropped when the harness started a collection less than this ago.
pub(crate) const REFRESH_MIN_GAP: Duration = Duration::from_secs(10);
/// Snapshot events are at least this far apart (about four a second at most).
pub(crate) const EMIT_MIN_GAP: Duration = Duration::from_millis(250);
/// Rate-limit cool-down: the wait when the source names none, and the longest wait we honour.
const COOLDOWN_DEFAULT: Duration = Duration::from_secs(5 * 60);
const COOLDOWN_MAX: Duration = Duration::from_secs(30 * 60);
const MAX_BACKOFF: Duration = Duration::from_secs(15 * 60);
/// The OpenCode, Kimi and z.ai endpoints are remote, undocumented and rate-sensitive, and their
/// collectors leave the pacing to the caller. (The Claude OAuth endpoint paces itself in its collector, so Claude is not slowed.)
const REMOTE_MIN_CADENCE: Duration = Duration::from_secs(300);
/// 2^10 times any cadence already exceeds the cap, so the shift never needs to grow past it.
const MAX_DOUBLINGS: u32 = 10;

/// How often a healthy harness is collected.
pub(crate) fn cadence(id: HarnessId, settings: &Settings) -> Duration {
    let local = Duration::from_secs(u64::from(settings.general.interval_secs));
    match id {
        HarnessId::Claude | HarnessId::Codex | HarnessId::Antigravity => local,
        HarnessId::Opencode | HarnessId::Kimi | HarnessId::Zai => local.max(REMOTE_MIN_CADENCE),
        HarnessId::Litellm => Duration::from_secs(u64::from(settings.litellm.poll_interval_secs)),
    }
}

/// The wait after the `failures`-th failure in a row: double per failure up to 15 minutes, straight
/// to the cap when the error is not retryable. Never shorter than the normal cadence.
pub(crate) fn backoff(cadence: Duration, failures: u32, retryable: bool) -> Duration {
    if failures == 0 {
        return cadence;
    }
    let grown = if retryable {
        cadence.saturating_mul(1_u32 << failures.min(MAX_DOUBLINGS))
    } else {
        MAX_BACKOFF
    };
    grown.min(MAX_BACKOFF).max(cadence)
}

/// The wait after a rate-limit answer: the source's `Retry-After` (5 min when absent), held to
/// 30 minutes, at least the ordinary backoff and never below the normal cadence.
pub(crate) fn cooldown(
    cadence: Duration,
    failures: u32,
    retry_after_secs: Option<u64>,
) -> Duration {
    let asked = retry_after_secs.map_or(COOLDOWN_DEFAULT, Duration::from_secs);
    asked.max(backoff(cadence, failures, true)).min(COOLDOWN_MAX).max(cadence)
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn secs(n: u64) -> Duration {
        Duration::from_secs(n)
    }

    #[test]
    fn local_sources_follow_the_general_interval_and_litellm_its_own() {
        let mut settings = Settings::default();
        settings.general.interval_secs = 45;
        settings.litellm.poll_interval_secs = 600;
        assert_eq!(cadence(HarnessId::Claude, &settings), secs(45));
        assert_eq!(cadence(HarnessId::Codex, &settings), secs(45));
        assert_eq!(cadence(HarnessId::Antigravity, &settings), secs(45));
        assert_eq!(cadence(HarnessId::Litellm, &settings), secs(600));
    }

    #[test]
    fn remote_sources_never_poll_faster_than_five_minutes() {
        let mut settings = Settings::default();
        settings.general.interval_secs = 30;
        for id in [HarnessId::Opencode, HarnessId::Kimi, HarnessId::Zai] {
            assert_eq!(cadence(id, &settings), secs(300), "{id:?}");
        }
        assert_eq!(cadence(HarnessId::Claude, &settings), secs(30), "statusLine file is local");
        settings.claude.oauth_usage = true;
        assert_eq!(cadence(HarnessId::Claude, &settings), secs(30), "OAuth paces itself");
        settings.general.interval_secs = 900;
        for id in [HarnessId::Opencode, HarnessId::Kimi, HarnessId::Zai] {
            assert_eq!(cadence(id, &settings), secs(900), "{id:?}: a slower interval wins");
        }
        assert_eq!(cadence(HarnessId::Claude, &settings), secs(900));
    }

    #[test]
    fn retryable_failures_double_up_to_fifteen_minutes() {
        let steps: Vec<u64> = (0..=7).map(|n| backoff(secs(60), n, true).as_secs()).collect();
        assert_eq!(steps, [60, 120, 240, 480, 900, 900, 900, 900]);
        let steps: Vec<u64> = (0..=4).map(|n| backoff(secs(300), n, true).as_secs()).collect();
        assert_eq!(steps, [300, 600, 900, 900, 900]);
    }

    #[test]
    fn a_failure_that_needs_the_user_goes_straight_to_the_cap() {
        assert_eq!(backoff(secs(60), 1, false), secs(900));
        assert_eq!(backoff(secs(30), 1, false), secs(900));
    }

    #[test]
    fn backoff_never_polls_faster_than_the_normal_cadence() {
        assert_eq!(backoff(secs(3600), 1, true), secs(3600));
        assert_eq!(backoff(secs(3600), 5, false), secs(3600));
        assert_eq!(backoff(secs(1000), 3, true), secs(1000));
    }

    #[test]
    fn a_rate_limit_waits_five_minutes_unless_the_source_says_otherwise() {
        let wait = |after: Option<u64>| cooldown(secs(60), 1, after).as_secs();
        assert_eq!(wait(None), 300);
        assert_eq!(wait(Some(900)), 900);
        assert_eq!(wait(Some(86_400)), 1800, "capped at 30 minutes");
        assert_eq!(wait(Some(30)), 120, "a short ask is still held to the ordinary backoff");
        assert_eq!(wait(Some(u64::MAX)), 1800);
    }

    #[test]
    fn a_rate_limit_never_waits_less_than_the_cadence_or_the_backoff() {
        assert_eq!(cooldown(secs(3600), 1, None), secs(3600));
        assert_eq!(cooldown(secs(60), 6, None), secs(900), "repeated limits keep growing");
        assert_eq!(cooldown(secs(60), u32::MAX, None), secs(900));
        assert_eq!(cooldown(Duration::MAX, 1, None), Duration::MAX);
    }

    #[test]
    fn a_huge_failure_count_cannot_overflow() {
        assert_eq!(backoff(secs(30), u32::MAX, true), secs(900));
        assert_eq!(backoff(Duration::MAX, u32::MAX, true), Duration::MAX);
    }
}
