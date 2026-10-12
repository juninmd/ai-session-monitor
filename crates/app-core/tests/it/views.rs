//! Builders for the views the fake collectors return. All values are fabricated.

use usage_model::{
    ErrorCode, HarnessError, HarnessId, HarnessView, UsageWindow, ViewState, WindowKind,
};

/// Epoch seconds at virtual time zero.
pub const BASE: i64 = 1_800_000_000;
pub const WEEK: i64 = 7 * 86_400;

pub fn window(used: f64, resets_in_secs: i64) -> UsageWindow {
    let mut window = UsageWindow::new("five_hour", WindowKind::FiveHour);
    window.used_pct = Some(used);
    window.resets_at = Some(BASE + resets_in_secs);
    window
}

/// A ready view with one window that stays current for a week.
pub fn ready(id: HarnessId, used: f64) -> HarnessView {
    ready_resetting(id, used, WEEK)
}

pub fn ready_resetting(id: HarnessId, used: f64, resets_in_secs: i64) -> HarnessView {
    let mut view = HarnessView::with_state(id, ViewState::Ready);
    view.windows = vec![window(used, resets_in_secs)];
    view
}

pub fn failing(id: HarnessId, code: ErrorCode, retryable: bool) -> HarnessView {
    HarnessView::error(id, HarnessError::new(code, retryable))
}

/// The view of `id` in a list of views.
pub fn of(views: &[HarnessView], id: HarnessId) -> &HarnessView {
    views.iter().find(|view| view.id == id).unwrap_or_else(|| panic!("no view for {id:?}"))
}
