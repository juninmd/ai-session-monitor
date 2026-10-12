//! The key scope: `GET /key/info` and what the view makes of fabricated key records.

use std::collections::BTreeMap;

use serde_json::json;
use usage_model::{Amount, AmountUnit, Severity, ViewState, WindowKind};

use crate::common::{
    Fake, NOW, Reply, SECRET, assert_ready, codes, ids, meta, run, settings, window,
};

async fn serving(body: &str) -> Fake {
    Fake::routes([("/key/info", Reply::json(body))]).await
}

#[tokio::test]
async fn a_key_with_a_budget_and_a_reset_is_one_budget_window() {
    let fake = serving(fixture!("key_with_budget.json")).await;
    let view = run(&settings(&fake.base())).await;
    assert_ready(&view);
    assert_eq!(view.read_at, Some(NOW));
    assert!(!view.stale);
    assert!(view.notices.is_empty());

    let [budget] = view.windows.as_slice() else { panic!("one window: {view:?}") };
    assert_eq!((budget.id.as_str(), budget.kind), ("budget", WindowKind::Budget));
    assert_eq!((budget.group.as_deref(), budget.label.as_deref()), (None, Some("ci-bot")));
    assert_eq!(budget.used_pct, Some(25.0));
    assert_eq!(
        budget.amount,
        Some(Amount { used: 12.5, limit: Some(50.0), unit: AmountUnit::Usd })
    );
    assert_eq!(budget.resets_at, Some(1_793_491_200), "2026-11-01T00:00:00Z");
    assert_eq!(budget.window_minutes, Some(43_200), "30d");

    let facts: Vec<_> =
        view.facts.iter().map(|f| (f.id.as_str(), f.number, f.unit.as_deref())).collect();
    assert_eq!(
        facts,
        [
            ("rpm_limit", Some(60.0), Some("requests")),
            ("tpm_limit", Some(200_000.0), Some("tokens")),
            ("max_parallel_requests", Some(4.0), Some("requests")),
            ("expires", Some(1_798_761_600.0), Some("epoch_s")),
        ]
    );
    let expected = BTreeMap::from([
        ("alias", "ci-bot"),
        ("host", "127.0.0.1"),
        ("level", "key"),
        ("scope", "key"),
    ]);
    assert_eq!(meta(&view), expected);
}

#[tokio::test]
async fn the_poll_is_one_get_without_a_query_and_names_only_the_key_path() {
    let fake = serving(fixture!("key_with_budget.json")).await;
    run(&settings(&fake.base())).await;
    let seen = fake.requests();
    let [only] = seen.as_slice() else { panic!("exactly one request: {seen:?}") };
    assert_eq!((only.method.as_str(), only.target.as_str()), ("GET", "/key/info"));
    assert_eq!(only.header("accept"), Some("application/json"));
    assert_eq!(only.header("authorization"), Some(format!("Bearer {SECRET}").as_str()));
}

#[tokio::test]
async fn a_key_without_a_budget_has_no_window_and_says_so() {
    let fake = serving(fixture!("key_no_budget.json")).await;
    let view = run(&settings(&fake.base())).await;
    assert_ready(&view);
    assert!(view.windows.is_empty(), "spend alone is not a budget: {view:?}");
    assert!(view.facts.is_empty());
    assert_eq!(codes(&view), ["litellm_no_budget"]);
    assert_eq!(view.notices[0].severity, Severity::Info);
    assert!(!view.stale);
    assert_eq!(fake.requests().len(), 1, "no user or team to ask");
    assert_eq!(meta(&view).get("alias"), Some(&"scratch"));
}

#[tokio::test]
async fn going_over_budget_keeps_the_percentage_above_one_hundred() {
    let fake = serving(fixture!("over_budget.json")).await;
    let view = run(&settings(&fake.base())).await;
    assert_ready(&view);
    let budget = window(&view, "budget");
    assert_eq!(budget.used_pct, Some(260.0));
    assert_eq!(
        budget.amount,
        Some(Amount { used: 130.0, limit: Some(50.0), unit: AmountUnit::Usd })
    );
    assert_eq!((budget.window_minutes, budget.resets_at), (Some(1_440), Some(1_791_763_200)));
    assert!(!view.stale);
}

#[tokio::test]
async fn model_and_limit_budgets_follow_the_headline_most_consumed_first() {
    let fake = serving(fixture!("model_budgets.json")).await;
    let view = run(&settings(&fake.base())).await;
    assert_ready(&view);
    assert_eq!(
        ids(&view),
        [
            "budget",
            "limit:1h",
            "model:gpt-fixture-a",
            "limit:30d",
            "model:gpt-fixture-b",
            "model:gpt-fixture-c",
        ],
        "a model without recorded spend has no window"
    );
    let a = window(&view, "model:gpt-fixture-a");
    assert_eq!(
        (a.group.as_deref(), a.label.as_deref()),
        (Some("gpt-fixture-a"), Some("gpt-fixture-a"))
    );
    assert_eq!((a.used_pct, a.window_minutes, a.resets_at), (Some(80.0), Some(1_440), None));
    let hour = window(&view, "limit:1h");
    assert_eq!((hour.group.as_deref(), hour.label.as_deref()), (None, Some("1h")));
    assert_eq!(
        (hour.used_pct, hour.window_minutes, hour.resets_at),
        (Some(90.0), Some(60), Some(1_791_723_600))
    );
    assert_eq!(window(&view, "model:gpt-fixture-c").used_pct, Some(0.0), "known zero stays zero");
    assert!(view.windows.iter().all(|w| w.kind == WindowKind::Budget));
}

#[tokio::test]
async fn unusual_durations_leave_the_window_length_unknown() {
    let fake = serving(fixture!("odd_durations.json")).await;
    let view = run(&settings(&fake.base())).await;
    assert_ready(&view);
    let lengths: Vec<_> = view.windows.iter().map(|w| (w.id.as_str(), w.window_minutes)).collect();
    assert_eq!(
        lengths,
        [
            ("budget", Some(90)),
            ("model:model-garbage", None),
            ("model:model-monthly", None),
            ("model:model-two-hours", Some(120)),
            ("model:model-zero", None),
            ("model:model-no-period", None),
        ]
    );
}

#[tokio::test]
async fn null_everything_is_unknown_never_zero() {
    let fake = serving(fixture!("null_everything.json")).await;
    let view = run(&settings(&fake.base())).await;
    assert_ready(&view);
    assert!(view.windows.is_empty() && view.facts.is_empty());
    assert_eq!(codes(&view), ["litellm_no_budget"]);
    assert_eq!(meta(&view).get("alias"), None);
    assert_eq!(fake.requests().len(), 1);
}

#[tokio::test]
async fn hostile_values_are_unknown_and_never_clamped_into_something_plausible() {
    let fake = serving(fixture!("hostile_numbers.json")).await;
    let view = run(&settings(&fake.base())).await;
    assert_ready(&view);
    assert!(view.windows.is_empty(), "{view:?}");
    assert!(view.facts.is_empty(), "{view:?}");
    assert_eq!(codes(&view), ["litellm_no_budget"]);
    assert_eq!(meta(&view).get("alias"), Some(&"hostile"));
    assert_eq!(fake.requests().len(), 1, "a numeric user id is not a link to follow");
}

#[tokio::test]
async fn a_zero_or_negative_cap_is_no_cap() {
    for max in [json!(0), json!(0.0), json!(-5), json!(-0.0)] {
        let body = json!({ "info": { "spend": 4.0, "max_budget": max } }).to_string();
        let fake = serving(&body).await;
        let view = run(&settings(&fake.base())).await;
        assert_ready(&view);
        assert!(view.windows.is_empty(), "{max}: {view:?}");
        assert_eq!(codes(&view), ["litellm_no_budget"], "{max}");
    }
}

#[tokio::test]
async fn a_cap_with_unknown_spend_shows_the_cap_but_no_percentage() {
    let fake = serving(&json!({ "info": { "spend": null, "max_budget": 25.0 } }).to_string()).await;
    let view = run(&settings(&fake.base())).await;
    assert_ready(&view);
    let budget = window(&view, "budget");
    assert_eq!((budget.used_pct, budget.amount.as_ref()), (None, None));
    assert!(view.notices.is_empty(), "a cap exists, so this is not 'no budget'");
}

#[tokio::test]
async fn a_tiny_cap_cannot_turn_the_percentage_into_infinity() {
    let fake =
        serving(&json!({ "info": { "spend": 1e15, "max_budget": 1e-300 } }).to_string()).await;
    let view = run(&settings(&fake.base())).await;
    let budget = window(&view, "budget");
    assert_eq!(budget.used_pct, None);
    assert!(budget.amount.is_some());
}

#[tokio::test]
async fn aliases_are_cleaned_and_a_credential_shaped_alias_is_dropped() {
    let cases = [
        ("  spaced  ", Some("spaced")),
        ("a\u{0}b\nc", Some("abc")),
        (SECRET, None),
        ("prod-sk-fixture-7f3a9c1e5b2d4a60-eu", None),
        ("   ", None),
    ];
    for (alias, shown) in cases {
        let body = json!({ "info": { "key_alias": alias, "spend": 1.0, "max_budget": 2.0 } });
        let fake = serving(&body.to_string()).await;
        let view = run(&settings(&fake.base())).await;
        assert_eq!(window(&view, "budget").label.as_deref(), shown, "{alias:?}");
        assert_eq!(meta(&view).get("alias").copied(), shown, "{alias:?}");
    }
}

#[tokio::test]
async fn very_long_aliases_are_cut_to_a_displayable_length() {
    let body = json!({ "info": { "key_alias": "é".repeat(500), "spend": 1.0, "max_budget": 2.0 } });
    let fake = serving(&body.to_string()).await;
    let view = run(&settings(&fake.base())).await;
    let label = window(&view, "budget").label.clone().unwrap();
    assert_eq!(label.chars().count(), 81);
    assert!(label.ends_with('…'));
}

#[tokio::test]
async fn a_large_body_under_the_cap_is_still_read() {
    let pad = "p".repeat(900 * 1024);
    let body = json!({ "info": { "spend": 1.0, "max_budget": 4.0, "padding": pad } });
    let fake = serving(&body.to_string()).await;
    let view = run(&settings(&fake.base())).await;
    assert_ready(&view);
    assert_eq!(window(&view, "budget").used_pct, Some(25.0));
}

#[tokio::test]
async fn an_empty_record_is_a_ready_view_without_windows() {
    let fake = serving(r#"{"info": {}}"#).await;
    let view = run(&settings(&fake.base())).await;
    assert_eq!(view.state, ViewState::Ready);
    assert!(view.windows.is_empty());
    assert_eq!(codes(&view), ["litellm_no_budget"]);
}
