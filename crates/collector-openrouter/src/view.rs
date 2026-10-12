//! What the two documents become: one percent window when the key has a limit, facts otherwise.

use usage_model::{
    Amount, AmountUnit, Fact, HarnessView, Source, UsageWindow, ViewState, WindowKind,
};

use crate::ID;
use crate::parse::KeyInfo;
use crate::remote::Fetched;

pub(crate) fn ready(fetched: &Fetched, now_secs: i64) -> HarnessView {
    let mut view = HarnessView::with_state(ID, ViewState::Ready);
    view.source = Some(Source { id: "key_credits".to_owned(), official: true });
    view.read_at = Some(now_secs);
    view.windows = limit_window(&fetched.key).into_iter().collect();
    view.facts = facts(fetched);
    view
}

/// Only a positive limit makes a percentage; `null` means unlimited and a zero cap has no ratio.
fn limit_window(key: &KeyInfo) -> Option<UsageWindow> {
    let limit = key.limit.filter(|limit| *limit > 0.0)?;
    let mut window = UsageWindow::new("key_limit", WindowKind::Budget);
    // The remaining figure already accounts for the cap's own resets; `usage` is lifetime spend.
    let used = key.limit_remaining.map(|left| limit - left).or(key.usage);
    if let Some(used) = used.map(|used| used.max(0.0)) {
        window.used_pct = Some(used / limit * 100.0).filter(|pct| pct.is_finite());
        window.amount = Some(Amount { used, limit: Some(limit), unit: AmountUnit::Usd });
    }
    Some(window)
}

fn facts(fetched: &Fetched) -> Vec<Fact> {
    let key = &fetched.key;
    let balance = fetched.credits.as_ref().map(|credits| credits.total - credits.used);
    [
        ("balance", balance),
        ("usage_daily", key.usage_daily),
        ("usage_weekly", key.usage_weekly),
        ("usage_monthly", key.usage_monthly),
        ("usage_total", key.usage),
    ]
    .into_iter()
    .filter_map(|(id, number)| {
        Some(Fact {
            id: id.to_owned(),
            number: Some(number?),
            text: None,
            unit: Some("usd".to_owned()),
        })
    })
    .collect()
}
