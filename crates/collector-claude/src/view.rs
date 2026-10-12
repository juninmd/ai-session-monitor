//! Turns what the shim recorded into a `HarnessView`. Pure: the clock comes from the caller.

use usage_model::status::is_expired;
use usage_model::{HarnessError, HarnessId, HarnessView, Severity, Source, UsageWindow, ViewState};

use crate::datafile::DataFile;
use crate::notice;

pub(crate) const SOURCE_ID: &str = "statusline";
/// Claude Code drops a window once it resets, and the longest one lasts a week, so a recording
/// older than that (plus a day) cannot describe any window that is still open.
const MAX_AGE_SECS: i64 = 8 * 24 * 3600;
/// The shim and the app read the same clock; more than this ahead means it was set back.
const MAX_FUTURE_SKEW_SECS: i64 = 300;

fn base(state: ViewState) -> HarnessView {
    let mut view = HarnessView::with_state(HarnessId::Claude, state);
    view.source = Some(Source { id: SOURCE_ID.to_owned(), official: true });
    view
}

/// The shim is installed but no Claude Code session has reported yet.
pub(crate) fn waiting_first_session() -> HarnessView {
    let mut view = base(ViewState::NoData);
    view.notices.push(notice::plain(notice::WAITING_FIRST_SESSION, Severity::Info));
    view
}

pub(crate) fn from_data(file: DataFile, now: i64) -> HarnessView {
    let DataFile { captured_at, windows } = file;
    let Some(windows) = windows else {
        // Claude Code reported no `rate_limits`: API-key billing, or a plan without limits.
        let mut view = base(ViewState::NoData);
        view.read_at = Some(captured_at);
        view.notices.push(notice::plain(notice::NO_RATE_LIMITS, Severity::Warn));
        return view;
    };
    let mut view = base(ViewState::Ready);
    view.read_at = Some(captured_at);
    view.stale = is_stale(&windows, captured_at, now);
    view.windows = windows;
    view
}

/// The shim is missing, moved or somebody else's statusLine is in the way.
pub(crate) fn needs_setup(outdated: bool) -> HarnessView {
    let mut view = HarnessView::needs_setup(HarnessId::Claude);
    if outdated {
        view.notices.push(notice::plain(notice::SHIM_OUTDATED, Severity::Warn));
    }
    view
}

pub(crate) fn failed(error: HarnessError) -> HarnessView {
    HarnessView::error(HarnessId::Claude, error)
}

/// Every window has reset, or the recording is older than any window can be.
fn is_stale(windows: &[UsageWindow], captured_at: i64, now: i64) -> bool {
    let all_reset = !windows.is_empty() && windows.iter().all(|w| is_expired(w, now));
    all_reset
        || now.saturating_sub(captured_at) > MAX_AGE_SECS
        || captured_at.saturating_sub(now) > MAX_FUTURE_SKEW_SECS
}

#[cfg(test)]
mod tests {
    use usage_model::WindowKind;

    use super::*;

    const NOW: i64 = 1_800_000_000;

    fn window(id: &str, resets_at: Option<i64>) -> UsageWindow {
        let mut w = UsageWindow::new(id, WindowKind::FiveHour);
        w.used_pct = Some(10.0);
        w.resets_at = resets_at;
        w
    }

    fn file(windows: Option<Vec<UsageWindow>>, captured_at: i64) -> DataFile {
        DataFile { captured_at, windows }
    }

    #[test]
    fn ready_keeps_the_windows_and_reports_when_they_were_recorded() {
        let view = from_data(file(Some(vec![window("five_hour", Some(NOW + 60))]), NOW - 30), NOW);
        assert_eq!(
            (view.state, view.stale, view.read_at),
            (ViewState::Ready, false, Some(NOW - 30))
        );
        assert_eq!(view.source, Some(Source { id: "statusline".into(), official: true }));
        assert_eq!(view.windows.len(), 1);
        assert!(view.notices.is_empty() && view.error.is_none() && view.plan.is_none());
    }

    #[test]
    fn a_window_resetting_between_two_polls_flips_the_view_to_stale() {
        let data = || file(Some(vec![window("five_hour", Some(NOW + 1))]), NOW - 60);
        assert!(!from_data(data(), NOW).stale, "one second before the reset");
        assert!(from_data(data(), NOW + 1).stale, "the reset instant counts as reset");
        assert!(from_data(data(), NOW + 2).stale);
        assert_eq!(from_data(data(), NOW + 2).windows.len(), 1, "the windows stay visible");
    }

    #[test]
    fn one_open_window_keeps_the_view_fresh() {
        let windows = vec![window("five_hour", Some(NOW - 5)), window("weekly", Some(NOW + 3600))];
        assert!(!from_data(file(Some(windows), NOW - 600), NOW).stale);
    }

    #[test]
    fn a_window_without_a_reset_time_never_counts_as_reset() {
        let windows = vec![window("five_hour", None), window("weekly", Some(NOW - 5))];
        assert!(!from_data(file(Some(windows), NOW - 600), NOW).stale);
    }

    #[test]
    fn a_recording_older_than_any_window_is_stale_even_if_nothing_says_so() {
        let old =
            |age: i64| from_data(file(Some(vec![window("weekly", None)]), NOW - age), NOW).stale;
        assert!(!old(8 * 24 * 3600), "the bound itself is still fine");
        assert!(old(8 * 24 * 3600 + 1));
    }

    #[test]
    fn a_recording_from_the_future_is_stale_beyond_a_small_skew() {
        let ahead =
            |secs: i64| from_data(file(Some(vec![window("weekly", None)]), NOW + secs), NOW).stale;
        assert!(!ahead(300));
        assert!(ahead(301));
    }

    #[test]
    fn no_rate_limits_is_no_data_with_a_warning_and_the_capture_time() {
        let view = from_data(file(None, NOW - 90), NOW);
        assert_eq!(
            (view.state, view.read_at, view.stale),
            (ViewState::NoData, Some(NOW - 90), false)
        );
        assert!(view.windows.is_empty());
        assert_eq!(view.notices.len(), 1);
        assert_eq!(
            (view.notices[0].code.as_str(), view.notices[0].severity),
            ("claude_no_rate_limits", Severity::Warn)
        );
    }

    #[test]
    fn waiting_for_the_first_session_is_no_data_with_an_info_notice() {
        let view = waiting_first_session();
        assert_eq!((view.state, view.read_at), (ViewState::NoData, None));
        assert_eq!(view.notices[0].code, "claude_waiting_first_session");
        assert_eq!(view.notices[0].severity, Severity::Info);
    }

    #[test]
    fn needs_setup_flags_an_outdated_shim_only_when_it_is_one() {
        assert!(needs_setup(false).notices.is_empty());
        let outdated = needs_setup(true);
        assert_eq!(outdated.state, ViewState::NeedsSetup);
        assert_eq!(
            (outdated.notices[0].code.as_str(), outdated.notices[0].severity),
            ("claude_shim_outdated", Severity::Warn)
        );
    }
}
