//! What the documents become: a percent window only when there is a spending limit, facts otherwise.

use usage_model::{
    Amount, AmountUnit, Fact, HarnessView, Source, UsageWindow, ViewState, WindowKind,
};

use crate::ID;
use crate::parse::Checklist;
use crate::remote::Fetched;

pub(crate) fn ready(fetched: &Fetched, now_secs: i64) -> HarnessView {
    let mut view = HarnessView::with_state(ID, ViewState::Ready);
    view.source = Some(Source { id: "payment_checklist".to_owned(), official: true });
    view.read_at = Some(now_secs);
    view.windows = limit_window(&fetched.checklist).into_iter().collect();
    view.facts = facts(fetched);
    view
}

/// The limit caps the spend since the last invoice; when that is unknown there is no ratio.
fn limit_window(checklist: &Checklist) -> Option<UsageWindow> {
    let limit = checklist.limit?;
    let used = checklist.recent?;
    let mut window = UsageWindow::new("spend_limit", WindowKind::Budget);
    window.used_pct = Some(used / limit * 100.0).filter(|pct| pct.is_finite());
    window.amount = Some(Amount { used, limit: Some(limit), unit: AmountUnit::Usd });
    Some(window)
}

fn facts(fetched: &Fetched) -> Vec<Fact> {
    let balance = fetched.checklist.stripe_balance;
    // Credit and debt are different facts: a negative "balance" would read as a bug.
    let (id, amount) = if balance <= 0.0 { ("balance", -balance) } else { ("owed", balance) };
    [
        (id, Some(amount)),
        ("usage_recent", fetched.checklist.recent),
        ("usage_month", fetched.month_spend),
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
