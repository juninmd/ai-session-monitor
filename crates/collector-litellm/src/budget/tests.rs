use crate::entries::{LimitBudget, ModelBudget};
use crate::record::Caps;

use super::*;

const SECRET: &str = "lk-fixture-secret-0001";

fn capped(spend: Option<f64>, max: f64) -> Level {
    Level { spend, max_budget: Some(max), ..Level::default() }
}

fn model(name: &str, spend: f64, limit: f64) -> ModelBudget {
    ModelBudget { name: name.into(), spend, limit, period: Some("30d".into()) }
}

#[test]
fn the_headline_carries_amount_percentage_reset_and_length() {
    let level = Level {
        alias: Some("ci-bot".into()),
        duration: Some("30d".into()),
        resets_at: Some(1_793_491_200),
        ..capped(Some(12.5), 50.0)
    };
    let built = windows(&level, SECRET);
    let [w] = built.list.as_slice() else { panic!("one window") };
    assert_eq!((w.id.as_str(), w.kind, w.group.as_deref()), ("budget", WindowKind::Budget, None));
    assert_eq!(w.label.as_deref(), Some("ci-bot"));
    assert_eq!(
        (w.used_pct, w.window_minutes, w.resets_at),
        (Some(25.0), Some(43_200), Some(1_793_491_200))
    );
    assert_eq!(w.amount, Some(Amount { used: 12.5, limit: Some(50.0), unit: AmountUnit::Usd }));
    assert_eq!(built.dropped, 0);
}

#[test]
fn no_cap_means_no_window_even_when_spend_is_known() {
    let level = Level { spend: Some(99.0), ..Level::default() };
    assert!(windows(&level, SECRET).list.is_empty());
}

#[test]
fn unknown_spend_keeps_the_cap_but_never_becomes_zero_percent() {
    let built = windows(&capped(None, 50.0), SECRET);
    let w = &built.list[0];
    assert_eq!((w.used_pct, w.amount.as_ref()), (None, None));
}

#[test]
fn over_budget_percentage_is_kept_above_one_hundred() {
    let built = windows(&capped(Some(130.0), 50.0), SECRET);
    assert_eq!(built.list[0].used_pct, Some(260.0));
}

#[test]
fn a_ratio_that_overflows_is_unknown_but_the_amounts_stay() {
    let built = windows(&capped(Some(9e15), 1e-300), SECRET);
    let w = &built.list[0];
    assert_eq!(w.used_pct, None);
    assert!(w.amount.is_some());
}

#[test]
fn a_credential_shaped_alias_is_not_used_as_the_label() {
    let level = Level { alias: Some(SECRET.into()), ..capped(Some(1.0), 2.0) };
    assert_eq!(windows(&level, SECRET).list[0].label, None);
}

#[test]
fn extras_follow_the_headline_most_consumed_first() {
    let level = Level {
        models: vec![model("light", 0.1, 10.0), model("heavy", 9.0, 10.0)],
        limits: vec![LimitBudget {
            duration: "1h".into(),
            spend: 2.0,
            limit: 4.0,
            resets_at: Some(5),
        }],
        ..capped(Some(1.0), 10.0)
    };
    let ids: Vec<_> = windows(&level, SECRET).list.iter().map(|w| w.id.clone()).collect();
    assert_eq!(ids, ["budget", "model:heavy", "limit:1h", "model:light"]);
}

#[test]
fn ties_and_unknown_use_are_ordered_by_id_after_the_known_ones() {
    let overflow = |name: &str| model(name, 9e15, 1e-300);
    let models = vec![overflow("z"), model("b", 1.0, 2.0), overflow("y"), model("a", 1.0, 2.0)];
    let level = Level { models, ..Level::default() };
    let ids: Vec<_> = windows(&level, SECRET).list.iter().map(|w| w.id.clone()).collect();
    assert_eq!(ids, ["model:a", "model:b", "model:y", "model:z"]);
}

#[test]
fn model_windows_are_grouped_and_named_by_model() {
    let level = Level { models: vec![model("gpt-5.6-terra", 0.0002, 0.0001)], ..Level::default() };
    let w = &windows(&level, SECRET).list[0];
    assert_eq!(
        (w.group.as_deref(), w.label.as_deref()),
        (Some("gpt-5.6-terra"), Some("gpt-5.6-terra"))
    );
    assert_eq!((w.window_minutes, w.resets_at), (Some(43_200), None));
    assert!((w.used_pct.expect("pct") - 200.0).abs() < 1e-9);
}

#[test]
fn limit_windows_take_length_and_reset_from_their_own_entry() {
    let level = Level {
        limits: vec![LimitBudget {
            duration: "1h".into(),
            spend: 1.0,
            limit: 4.0,
            resets_at: Some(7),
        }],
        ..Level::default()
    };
    let w = &windows(&level, SECRET).list[0];
    assert_eq!((w.group.as_deref(), w.label.as_deref()), (None, Some("1h")));
    assert_eq!((w.window_minutes, w.resets_at, w.used_pct), (Some(60), Some(7), Some(25.0)));
}

#[test]
fn extras_are_capped_and_the_cut_is_reported() {
    let models = (0..40).map(|i| model(&format!("m{i:02}"), f64::from(i), 100.0)).collect();
    let built = windows(&Level { models, ..Level::default() }, SECRET);
    assert_eq!(built.list.len(), MAX_EXTRA_WINDOWS);
    assert_eq!(built.dropped, 40 - MAX_EXTRA_WINDOWS);
    assert_eq!(built.list[0].id, "model:m39", "the most consumed survive the cut");
}

#[test]
fn duplicate_ids_collapse_to_one_window() {
    let level =
        Level { models: vec![model("a", 1.0, 2.0), model("a", 1.5, 2.0)], ..Level::default() };
    assert_eq!(windows(&level, SECRET).list.len(), 1);
}

#[test]
fn static_caps_become_facts_in_a_fixed_order() {
    let level = Level {
        caps: Caps {
            rpm: Some(60.0),
            tpm: Some(200_000.0),
            parallel: Some(4.0),
            expires: Some(1_798_761_600),
        },
        ..Level::default()
    };
    let got: Vec<_> = facts(&level).into_iter().map(|f| (f.id, f.number, f.unit)).collect();
    let expect = |id: &str, n: f64, unit: &str| (id.to_owned(), Some(n), Some(unit.to_owned()));
    assert_eq!(
        got,
        [
            expect("rpm_limit", 60.0, "requests"),
            expect("tpm_limit", 200_000.0, "tokens"),
            expect("max_parallel_requests", 4.0, "requests"),
            expect("expires", 1_798_761_600.0, EPOCH_UNIT),
        ]
    );
    assert!(facts(&Level::default()).is_empty());
}
