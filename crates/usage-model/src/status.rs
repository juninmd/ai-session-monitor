//! Pace and status rules. The TypeScript twin lives in `src/lib/usage.ts`; both are pinned to the
//! same vectors in `contract/status-vectors.json` so the tray icon and the flyout never disagree.

use crate::view::{HarnessView, UsageWindow, ViewState};

/// A window younger than this has no meaningful burn rate yet.
const EARLY_ELAPSED_MIN: f64 = 10.0;
/// Below this much consumption the burn rate is noise.
const EARLY_USED_PCT: f64 = 3.0;
/// Runway shorter than this while the window still has time left is critical.
const CRIT_RUNWAY_MIN: f64 = 30.0;
/// Slack before "runs out before the reset" is reported.
const RISK_SLACK_MIN: f64 = 5.0;
const CRIT_AVAILABLE_PCT: f64 = 10.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Ok,
    Warn,
    Crit,
    Out,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Pace {
    /// No window length or no known percentage: nothing to say.
    Unknown,
    /// Too early in the window to extrapolate.
    Early,
    Out,
    /// At the current burn rate the window lasts `runway_min` minutes, beyond the reset.
    Fine {
        runway_min: f64,
    },
    /// At the current burn rate the window runs out `early_min` minutes before the reset.
    Risk {
        runway_min: f64,
        early_min: f64,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Summary {
    pub window_id: String,
    pub available_pct: f64,
    pub status: Status,
}

/// Percentage still available, `None` while unknown.
pub fn available_pct(window: &UsageWindow) -> Option<f64> {
    window.used_pct.map(|used| (100.0 - used).clamp(0.0, 100.0))
}

/// A window whose reset moment has passed no longer describes reality.
pub fn is_expired(window: &UsageWindow, now_secs: i64) -> bool {
    window.resets_at.is_some_and(|reset| reset <= now_secs)
}

/// Minutes until the reset, floored at zero. `None` when the source gave no reset time.
pub fn minutes_left(window: &UsageWindow, now_secs: i64) -> Option<f64> {
    window.resets_at.map(|reset| ((reset - now_secs) as f64 / 60.0).max(0.0))
}

pub fn pace_of(window: &UsageWindow, now_secs: i64) -> Pace {
    // A window that already reset has nothing left to extrapolate.
    if is_expired(window, now_secs) {
        return Pace::Unknown;
    }
    let (Some(avail), Some(length), Some(left)) =
        (available_pct(window), window.window_minutes, minutes_left(window, now_secs))
    else {
        return Pace::Unknown;
    };
    if avail <= 0.0 {
        return Pace::Out;
    }
    let elapsed = f64::from(length) - left;
    let used = 100.0 - avail;
    if elapsed < EARLY_ELAPSED_MIN || used < EARLY_USED_PCT {
        return Pace::Early;
    }
    let runway_min = avail / (used / elapsed);
    if runway_min < left - RISK_SLACK_MIN {
        Pace::Risk { runway_min, early_min: left - runway_min }
    } else {
        Pace::Fine { runway_min }
    }
}

/// `None` for unknown or already-reset windows: they must not colour the UI or the tray.
pub fn status_of(window: &UsageWindow, now_secs: i64, warn_at_pct: f64) -> Option<Status> {
    let avail = available_pct(window)?;
    if is_expired(window, now_secs) {
        return None;
    }
    if avail <= 0.0 {
        return Some(Status::Out);
    }
    let pace = pace_of(window, now_secs);
    let crit_pace = matches!(pace, Pace::Risk { runway_min, .. } if runway_min < CRIT_RUNWAY_MIN);
    if avail <= CRIT_AVAILABLE_PCT || crit_pace {
        return Some(Status::Crit);
    }
    if avail <= warn_at_pct || matches!(pace, Pace::Risk { .. }) {
        return Some(Status::Warn);
    }
    Some(Status::Ok)
}

/// The most constrained usable window of a view; drives the tab chip and the tray ring.
pub fn summarize(view: &HarnessView, now_secs: i64, warn_at_pct: f64) -> Option<Summary> {
    if view.state != ViewState::Ready || view.stale {
        return None;
    }
    view.windows
        .iter()
        .filter_map(|window| {
            let status = status_of(window, now_secs, warn_at_pct)?;
            Some((window, available_pct(window)?, status))
        })
        .min_by(|a, b| {
            a.1.total_cmp(&b.1).then_with(|| {
                a.0.resets_at.unwrap_or(i64::MAX).cmp(&b.0.resets_at.unwrap_or(i64::MAX))
            })
        })
        .map(|(window, available_pct, status)| Summary {
            window_id: window.id.clone(),
            available_pct,
            status,
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HarnessId;
    use crate::view::WindowKind;

    const NOW: i64 = 1_800_000_000;

    fn window(length_min: u32, used: f64, left_min: i64) -> UsageWindow {
        let mut w = UsageWindow::new("five_hour", WindowKind::FiveHour);
        w.window_minutes = Some(length_min);
        w.used_pct = Some(used);
        w.resets_at = Some(NOW + left_min * 60);
        w
    }

    #[test]
    fn unknown_percentage_is_not_a_status() {
        let mut w = window(300, 50.0, 100);
        w.used_pct = None;
        assert_eq!(status_of(&w, NOW, 25.0), None);
        assert_eq!(pace_of(&w, NOW), Pace::Unknown);
    }

    #[test]
    fn expired_window_is_not_a_status() {
        let w = window(300, 90.0, -1);
        assert_eq!(status_of(&w, NOW, 25.0), None);
        assert_eq!(pace_of(&w, NOW), Pace::Unknown);
    }

    #[test]
    fn exhausted_window_is_out() {
        let w = window(300, 100.0, 47);
        assert_eq!(pace_of(&w, NOW), Pace::Out);
        assert_eq!(status_of(&w, NOW, 25.0), Some(Status::Out));
    }

    #[test]
    fn over_budget_clamps_to_out() {
        let w = window(43_200, 130.0, 8_000);
        assert_eq!(available_pct(&w), Some(0.0));
        assert_eq!(status_of(&w, NOW, 25.0), Some(Status::Out));
    }

    #[test]
    fn early_window_has_no_pace_and_stays_ok() {
        // 5 minutes into a 5 hour window with 20 % used.
        let w = window(300, 20.0, 295);
        assert_eq!(pace_of(&w, NOW), Pace::Early);
        assert_eq!(status_of(&w, NOW, 25.0), Some(Status::Ok));
    }

    #[test]
    fn healthy_pace_lasts_beyond_reset() {
        // 187 min elapsed, 48 % used: runway = 52 / (48 / 187) = 202.6 min > 113 left.
        let w = window(300, 48.0, 113);
        assert!(matches!(pace_of(&w, NOW), Pace::Fine { .. }));
        assert_eq!(status_of(&w, NOW, 25.0), Some(Status::Ok));
    }

    #[test]
    fn risky_pace_warns_even_with_plenty_available() {
        // Weekly: 4100 min elapsed, 45 % used: runway = 55 / (45 / 4100) = 5011 min < 5980 left.
        let w = window(10_080, 45.0, 5_980);
        let Pace::Risk { early_min, .. } = pace_of(&w, NOW) else { panic!("expected risk") };
        assert!((early_min - 968.8).abs() < 1.0, "early_min = {early_min}");
        assert_eq!(status_of(&w, NOW, 25.0), Some(Status::Warn));
    }

    #[test]
    fn short_runway_is_critical() {
        // 187 min elapsed, 88 % used: 12 % left (above the 10 % floor) but runway = 12 / (88 / 187)
        // = 25.5 min < 30 with 113 min to go, so the pace alone makes it critical.
        let w = window(300, 88.0, 113);
        assert_eq!(status_of(&w, NOW, 25.0), Some(Status::Crit));
    }

    #[test]
    fn low_availability_thresholds() {
        let low = window(300, 90.0, 20); // 10 % available, little time left: critical by threshold
        assert_eq!(status_of(&low, NOW, 25.0), Some(Status::Crit));
        let mid = window(300, 76.0, 20); // 24 % available, runway ok because few minutes remain
        assert_eq!(status_of(&mid, NOW, 25.0), Some(Status::Warn));
    }

    #[test]
    fn summary_picks_the_lowest_usable_window() {
        let mut view = HarnessView::with_state(HarnessId::Antigravity, ViewState::Ready);
        let mut gem = window(300, 54.0, 132);
        gem.id = "gemini:five_hour".into();
        let mut cg = window(300, 39.0, 132);
        cg.id = "claude_gpt:five_hour".into();
        let mut expired = window(300, 99.0, -5);
        expired.id = "stale".into();
        view.windows = vec![cg, gem, expired];
        let summary = summarize(&view, NOW, 25.0).expect("summary");
        assert_eq!(summary.window_id, "gemini:five_hour");
        assert!((summary.available_pct - 46.0).abs() < 1e-9);
    }

    #[test]
    fn summary_ignores_stale_and_non_ready_views() {
        let mut view = HarnessView::with_state(HarnessId::Claude, ViewState::Ready);
        view.windows = vec![window(300, 40.0, 100)];
        view.stale = true;
        assert_eq!(summarize(&view, NOW, 25.0), None);
        view.stale = false;
        view.state = ViewState::NoData;
        assert_eq!(summarize(&view, NOW, 25.0), None);
    }
}
