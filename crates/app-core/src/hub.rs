//! The latest view per harness, in tab order. Pure state: the scheduler feeds it results and the
//! publisher reads it, so the rules for failures and expiry are testable without a runtime.

use usage_model::status::is_expired;
use usage_model::{HarnessId, HarnessView, Settings, ViewState};

/// One counter per harness, in tab order.
pub(crate) type Counts = [u32; HarnessId::ALL.len()];

pub(crate) struct Hub {
    views: Vec<HarnessView>,
    /// How often a switch-on put each view back to `Loading`. The scheduler compares it with what
    /// it has seen, so an off/on pair it never observed still gets the read the tab is waiting for.
    switch_ons: Counts,
}

impl Hub {
    /// Every enabled harness starts in `Loading`, the others `Disabled`.
    pub(crate) fn new(settings: &Settings) -> Self {
        let views = HarnessId::ALL
            .iter()
            .map(|&id| {
                if settings.is_enabled(id) {
                    HarnessView::loading(id)
                } else {
                    HarnessView::disabled(id)
                }
            })
            .collect();
        Self { views, switch_ons: [0; HarnessId::ALL.len()] }
    }

    /// Brings the enabled/disabled views in line with `settings`; idempotent. A re-enabled
    /// harness goes back to `Loading`, a disabled one drops its data.
    pub(crate) fn reconcile_enabled(&mut self, settings: &Settings) {
        for &id in &HarnessId::ALL {
            let view = &mut self.views[id as usize];
            match (settings.is_enabled(id), view.state == ViewState::Disabled) {
                (false, false) => *view = HarnessView::disabled(id),
                (true, true) => {
                    *view = HarnessView::loading(id);
                    self.switch_ons[id as usize] = self.switch_ons[id as usize].wrapping_add(1);
                }
                _ => {}
            }
        }
    }

    pub(crate) fn switch_ons(&self) -> Counts {
        self.switch_ons
    }

    /// Stores a collector result. A failed collection never blanks a tab that has data: the
    /// previous good view stays `Ready`, flagged stale, with the new error attached.
    pub(crate) fn apply(&mut self, incoming: HarnessView) {
        let incoming = sanitize(incoming);
        let current = &mut self.views[incoming.id as usize];
        if incoming.state == ViewState::Error && current.state == ViewState::Ready {
            current.stale = true;
            current.error = incoming.error;
        } else {
            *current = incoming;
        }
    }

    /// The views as the UI should see them at `now_secs`.
    pub(crate) fn views_at(&self, now_secs: i64) -> Vec<HarnessView> {
        self.views.iter().map(|view| present(view, now_secs)).collect()
    }

    /// The first moment after `now_secs` at which a ready view's windows have all expired, i.e.
    /// when `views_at` starts to differ without any new collection.
    pub(crate) fn next_expiry(&self, now_secs: i64) -> Option<i64> {
        self.views
            .iter()
            .filter(|view| counts_for_expiry(view))
            .filter_map(|view| view.windows.iter().map(|w| w.resets_at).collect::<Option<Vec<_>>>())
            .filter_map(|resets| resets.into_iter().max())
            .filter(|&at| at > now_secs)
            .min()
    }
}

fn counts_for_expiry(view: &HarnessView) -> bool {
    view.state == ViewState::Ready && !view.stale && !view.windows.is_empty()
}

/// A ready view whose windows have all reset describes a window that no longer exists.
fn present(view: &HarnessView, now_secs: i64) -> HarnessView {
    let mut shown = view.clone();
    if counts_for_expiry(view) && view.windows.iter().all(|w| is_expired(w, now_secs)) {
        shown.stale = true;
    }
    shown
}

/// Unknown is `null`, never a number: a non-finite value from a collector (0/0 in a ratio) is
/// dropped, and it would also make every snapshot compare unequal to itself.
fn sanitize(mut view: HarnessView) -> HarnessView {
    for window in &mut view.windows {
        window.used_pct = window.used_pct.filter(|pct| pct.is_finite());
        window.amount =
            window.amount.take().filter(|amount| amount.used.is_finite()).map(|mut a| {
                a.limit = a.limit.filter(|limit| limit.is_finite());
                a
            });
    }
    for fact in &mut view.facts {
        fact.number = fact.number.filter(|number| number.is_finite());
    }
    view
}

#[cfg(test)]
mod tests;
