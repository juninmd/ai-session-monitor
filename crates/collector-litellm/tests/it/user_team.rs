//! The user and team scopes: the id asked for must be the id answered, and only the record's own
//! budget fields are read.

use std::collections::BTreeMap;

use serde_json::json;
use usage_model::settings::LitellmScope;
use usage_model::{Amount, AmountUnit, ErrorCode};

use crate::common::{Fake, Reply, assert_ready, codes, error_of, meta, param, run, scoped, window};

#[tokio::test]
async fn the_user_scope_reads_the_user_record_by_id() {
    let fake = Fake::routes([("/user/info", Reply::json(fixture!("user_info.json")))]).await;
    let view = run(&scoped(&fake.base(), LitellmScope::User, "user-aurora")).await;
    assert_ready(&view);
    assert_eq!(fake.targets(), ["/user/info?user_id=user-aurora"]);

    let [budget] = view.windows.as_slice() else { panic!("one window: {view:?}") };
    assert_eq!(budget.id, "budget");
    assert_eq!(budget.label.as_deref(), Some("Aurora"));
    assert_eq!(budget.used_pct, Some(20.0));
    assert_eq!(
        budget.amount,
        Some(Amount { used: 40.0, limit: Some(200.0), unit: AmountUnit::Usd })
    );
    assert_eq!((budget.resets_at, budget.window_minutes), (Some(1_793_491_200), Some(43_200)));
    let facts: Vec<_> = view.facts.iter().map(|f| (f.id.as_str(), f.number)).collect();
    assert_eq!(facts, [("rpm_limit", Some(120.0))]);
    let expected = BTreeMap::from([
        ("alias", "Aurora"),
        ("host", "127.0.0.1"),
        ("level", "user"),
        ("scope", "user"),
    ]);
    assert_eq!(meta(&view), expected);
}

#[tokio::test]
async fn the_team_scope_reads_the_team_record_without_its_key_listing() {
    let fake = Fake::routes([("/team/info", Reply::json(fixture!("team_info.json")))]).await;
    let view = run(&scoped(&fake.base(), LitellmScope::Team, "team-borealis")).await;
    assert_ready(&view);
    assert_eq!(
        fake.targets(),
        ["/team/info?team_id=team-borealis&key_limit=1"],
        "the default listing carries every key of the team and can pass the body cap"
    );

    let [budget] = view.windows.as_slice() else { panic!("one window: {view:?}") };
    assert_eq!(budget.label.as_deref(), Some("Borealis"));
    assert!(budget.used_pct.is_some_and(|pct| (pct - 91.025).abs() < 1e-9), "{budget:?}");
    assert_eq!((budget.resets_at, budget.window_minutes), (Some(1_791_936_000), Some(10_080)));
    let facts: Vec<_> = view.facts.iter().map(|f| (f.id.as_str(), f.number)).collect();
    assert_eq!(facts, [("tpm_limit", Some(1_000_000.0))]);
    assert_eq!(meta(&view).get("level"), Some(&"team"));
    assert_eq!(meta(&view).get("scope"), Some(&"team"));
}

#[tokio::test]
async fn only_the_scoped_records_own_budget_becomes_a_window() {
    let user = Fake::routes([("/user/info", Reply::json(fixture!("user_info.json")))]).await;
    let view = run(&scoped(&user.base(), LitellmScope::User, "user-aurora")).await;
    assert_eq!(view.windows.len(), 1, "keys[] and teams[] of the listing are not windows");
    let team = Fake::routes([("/team/info", Reply::json(fixture!("team_info.json")))]).await;
    let view = run(&scoped(&team.base(), LitellmScope::Team, "team-borealis")).await;
    assert_eq!(view.windows.len(), 1, "keys[] of the listing are not windows");
}

#[tokio::test]
async fn a_user_or_team_without_a_budget_says_so_and_never_follows_anything() {
    let user =
        Fake::routes([("/user/info", Reply::json(fixture!("user_info_no_budget.json")))]).await;
    let view = run(&scoped(&user.base(), LitellmScope::User, "user-aurora")).await;
    assert_ready(&view);
    assert!(view.windows.is_empty());
    assert_eq!(codes(&view), ["litellm_no_budget"]);
    assert_eq!(user.requests().len(), 1);

    let team =
        Fake::routes([("/team/info", Reply::json(fixture!("team_info_no_budget.json")))]).await;
    let view = run(&scoped(&team.base(), LitellmScope::Team, "team-borealis")).await;
    assert_eq!(codes(&view), ["litellm_no_budget"]);
    assert_eq!(team.requests().len(), 1);
}

#[tokio::test]
async fn a_record_for_another_id_is_an_endpoint_change_and_shows_nothing() {
    let other = json!({
        "user_id": "someone-else",
        "user_info": { "user_id": "someone-else", "spend": 1.0, "max_budget": 2.0 },
    });
    let fake = Fake::routes([("/user/info", Reply::json(&other.to_string()))]).await;
    let view = run(&scoped(&fake.base(), LitellmScope::User, "user-aurora")).await;
    let error = error_of(&view);
    assert_eq!((error.code, error.retryable), (ErrorCode::EndpointChanged, false));
    assert_eq!(param(error, "reason"), Some("id_mismatch"));
    assert_eq!(param(error, "host"), Some("127.0.0.1"));
}

#[tokio::test]
async fn ids_compare_exactly_so_a_case_variant_is_another_record() {
    let body =
        json!({ "team_id": "TEAM-A", "team_info": { "team_id": "TEAM-A", "max_budget": 1 } });
    let fake = Fake::routes([("/team/info", Reply::json(&body.to_string()))]).await;
    let view = run(&scoped(&fake.base(), LitellmScope::Team, "team-a")).await;
    assert_eq!(param(error_of(&view), "reason"), Some("id_mismatch"));
}

#[tokio::test]
async fn a_half_matching_record_is_still_a_mismatch() {
    let body = json!({
        "user_id": "user-aurora",
        "user_info": { "user_id": "someone-else", "max_budget": 1 },
    });
    let fake = Fake::routes([("/user/info", Reply::json(&body.to_string()))]).await;
    let view = run(&scoped(&fake.base(), LitellmScope::User, "user-aurora")).await;
    assert_eq!(param(error_of(&view), "reason"), Some("id_mismatch"));
}

#[tokio::test]
async fn a_record_that_does_not_say_whose_it_is_cannot_be_trusted() {
    for body in [
        json!({ "user_info": { "spend": 1.0, "max_budget": 2.0 } }),
        json!({ "user_id": null, "user_info": { "user_id": null, "max_budget": 2.0 } }),
        json!({ "user_id": 7, "user_info": { "user_id": ["user-aurora"], "max_budget": 2.0 } }),
    ] {
        let fake = Fake::routes([("/user/info", Reply::json(&body.to_string()))]).await;
        let view = run(&scoped(&fake.base(), LitellmScope::User, "user-aurora")).await;
        assert_eq!(param(error_of(&view), "reason"), Some("id_missing"), "{body}");
    }
}

#[tokio::test]
async fn one_echoed_id_is_enough_when_the_other_copy_is_null() {
    let body = json!({
        "user_id": null,
        "user_info": { "user_id": "user-aurora", "spend": 1.0, "max_budget": 4.0 },
    });
    let fake = Fake::routes([("/user/info", Reply::json(&body.to_string()))]).await;
    let view = run(&scoped(&fake.base(), LitellmScope::User, "user-aurora")).await;
    assert_ready(&view);
    assert_eq!(window(&view, "budget").used_pct, Some(25.0));
}

#[tokio::test]
async fn the_id_is_trimmed_before_it_is_asked_for() {
    let fake = Fake::routes([("/user/info", Reply::json(fixture!("user_info.json")))]).await;
    let view = run(&scoped(&fake.base(), LitellmScope::User, "  user-aurora\t")).await;
    assert_ready(&view);
    assert_eq!(fake.targets(), ["/user/info?user_id=user-aurora"]);
}

#[tokio::test]
async fn a_wrong_container_is_an_endpoint_change() {
    // A key record answered where a user record was expected.
    let fake = Fake::routes([("/user/info", Reply::json(fixture!("key_with_budget.json")))]).await;
    let view = run(&scoped(&fake.base(), LitellmScope::User, "user-aurora")).await;
    let error = error_of(&view);
    assert_eq!(error.code, ErrorCode::EndpointChanged);
    assert_eq!(param(error, "reason"), Some("no_record"));
}
