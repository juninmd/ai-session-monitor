//! A key with no budget of its own asks its user, then its team: at most two more requests, with
//! the same credential and host, and "no budget" is only claimed when every level was read.

use std::time::Duration;

use serde_json::json;
use usage_model::settings::LitellmSettings;
use usage_model::{HarnessView, Notice, Severity};

use crate::common::{Fake, Reply, assert_ready, codes, ids, meta, run, settings, window};

const USER: &str = fixture!("user_info.json");
const USER_NONE: &str = fixture!("user_info_no_budget.json");
const TEAM: &str = fixture!("team_info.json");
const TEAM_NONE: &str = fixture!("team_info_no_budget.json");

/// A key without a budget that belongs to a user and a team, with the given answers for them.
async fn walk(user: Reply, team: Reply) -> (Fake, HarnessView) {
    let fake = Fake::routes([
        ("/key/info", Reply::json(fixture!("key_follow_up.json"))),
        ("/user/info", user),
        ("/team/info", team),
    ])
    .await;
    let view = run(&settings(&fake.base())).await;
    (fake, view)
}

fn detail<'a>(notice: &'a Notice, key: &str) -> &'a str {
    notice.params.get(key).map_or("?", String::as_str)
}

/// `(level, reason)` of every "could not read this level" notice, in order.
fn unreadable(view: &HarnessView) -> Vec<(&str, &str)> {
    let unreadable = view.notices.iter().filter(|n| n.code == "litellm_level_unreadable");
    unreadable.map(|n| (detail(n, "level"), detail(n, "reason"))).collect()
}

#[tokio::test]
async fn the_team_budget_is_found_when_neither_key_nor_user_has_one() {
    let (fake, view) = walk(Reply::json(USER_NONE), Reply::json(TEAM)).await;
    assert_ready(&view);
    assert_eq!(
        fake.targets(),
        [
            "/key/info",
            "/user/info?user_id=user-aurora",
            "/team/info?team_id=team-borealis&key_limit=1"
        ]
    );
    let [budget] = view.windows.as_slice() else { panic!("one window: {view:?}") };
    assert_eq!(budget.label.as_deref(), Some("Borealis"));
    assert!(budget.used_pct.is_some_and(|pct| (pct - 91.025).abs() < 1e-9), "{budget:?}");
    assert_eq!(budget.window_minutes, Some(10_080));
    assert!(view.notices.is_empty(), "{:?}", view.notices);
    // The scope stays the one the user configured; the level says where the windows came from.
    assert_eq!(meta(&view).get("scope"), Some(&"key"));
    assert_eq!(meta(&view).get("level"), Some(&"team"));
    assert_eq!(meta(&view).get("alias"), Some(&"laptop-key"), "the alias is the key's own");
    let caps: Vec<_> = view.facts.iter().map(|f| f.id.as_str()).collect();
    assert_eq!(caps, ["rpm_limit"], "static caps stay those of the key that is polled");
}

#[tokio::test]
async fn the_user_budget_wins_and_the_team_is_never_asked() {
    let (fake, view) = walk(Reply::json(USER), Reply::json(TEAM)).await;
    assert_ready(&view);
    assert_eq!(fake.targets(), ["/key/info", "/user/info?user_id=user-aurora"]);
    assert_eq!(window(&view, "budget").label.as_deref(), Some("Aurora"));
    assert_eq!(meta(&view).get("level"), Some(&"user"));
}

#[tokio::test]
async fn a_key_with_its_own_budget_never_follows_anything() {
    let fake = Fake::routes([
        ("/key/info", Reply::json(fixture!("key_with_budget.json"))),
        ("/user/info", Reply::json(USER)),
        ("/team/info", Reply::json(TEAM)),
    ])
    .await;
    let view = run(&settings(&fake.base())).await;
    assert_ready(&view);
    assert_eq!(fake.targets(), ["/key/info"]);
    assert_eq!(meta(&view).get("level"), Some(&"key"));
}

#[tokio::test]
async fn follow_ups_reuse_the_credential_and_the_host() {
    let (fake, _) = walk(Reply::json(USER_NONE), Reply::json(TEAM)).await;
    let seen = fake.requests();
    assert_eq!(seen.len(), 3);
    for request in &seen {
        assert_eq!(request.method, "GET");
        assert_eq!(request.header("authorization"), seen[0].header("authorization"));
        assert_eq!(request.header("host"), seen[0].header("host"));
    }
    assert_eq!(fake.connections(), 3, "one fresh connection per request, none kept for reuse");
}

#[tokio::test]
async fn an_unreadable_user_is_a_notice_and_never_a_claim_of_no_budget() {
    let (_, view) = walk(Reply::json_status(403, "{}"), Reply::json(TEAM_NONE)).await;
    assert_ready(&view);
    assert!(view.windows.is_empty());
    assert_eq!(codes(&view), ["litellm_level_unreadable"]);
    assert_eq!(view.notices[0].severity, Severity::Warn);
    assert_eq!(unreadable(&view), [("user", "unauthorized")]);
}

#[tokio::test]
async fn every_unreadable_level_gets_its_own_notice_in_walk_order() {
    let (fake, view) = walk(Reply::json_status(500, "{}"), Reply::json_status(404, "{}")).await;
    assert_ready(&view);
    assert_eq!(codes(&view), ["litellm_level_unreadable", "litellm_level_unreadable"]);
    assert_eq!(unreadable(&view), [("user", "server_error"), ("team", "not_found")]);
    assert_eq!(fake.requests().len(), 3);
}

#[tokio::test]
async fn a_readable_team_still_shows_even_when_the_user_could_not_be_read() {
    let (_, view) = walk(Reply::json_status(429, "{}"), Reply::json(TEAM)).await;
    assert_ready(&view);
    assert_eq!(ids(&view), ["budget"]);
    assert_eq!(meta(&view).get("level"), Some(&"team"));
    assert_eq!(unreadable(&view), [("user", "rate_limited")]);
}

#[tokio::test]
async fn a_follow_up_answered_for_another_id_is_unreadable_not_trusted() {
    let stranger = json!({
        "user_id": "someone-else",
        "user_info": { "user_id": "someone-else", "spend": 1.0, "max_budget": 2.0 },
    });
    let (_, view) = walk(Reply::json(&stranger.to_string()), Reply::json(TEAM_NONE)).await;
    assert!(view.windows.is_empty());
    assert_eq!(unreadable(&view), [("user", "endpoint_changed")]);
}

#[tokio::test]
async fn a_follow_up_that_redirects_is_unreadable_and_never_followed() {
    let elsewhere = Fake::routes([("/user/info", Reply::json(USER))]).await;
    let target = format!("{}/user/info", elsewhere.base());
    let moved = Reply::status(302, &[("Location", target.as_str())]);
    let (_, view) = walk(moved, Reply::json(TEAM_NONE)).await;
    assert_eq!(unreadable(&view), [("user", "bad_response")]);
    elsewhere.assert_untouched().await;
}

#[tokio::test]
async fn a_hanging_follow_up_times_out_and_the_walk_goes_on() {
    let fake = Fake::routes([
        ("/key/info", Reply::json(fixture!("key_follow_up.json"))),
        ("/user/info", Reply::json(USER).after(Duration::from_secs(30))),
        ("/team/info", Reply::json(TEAM)),
    ])
    .await;
    let settings = LitellmSettings { timeout_secs: 1, ..settings(&fake.base()) };
    let view = run(&settings).await;
    assert_ready(&view);
    assert_eq!(unreadable(&view), [("user", "timeout")]);
    assert_eq!(meta(&view).get("level"), Some(&"team"));
}

#[tokio::test]
async fn ids_that_cannot_be_asked_about_are_skipped_without_a_request() {
    let key = json!({ "info": {
        "key_alias": "odd-ids", "spend": 1.0, "max_budget": null,
        "user_id": "line\nbreak", "team_id": "x".repeat(300),
    }});
    let fake = Fake::routes([("/key/info", Reply::json(&key.to_string()))]).await;
    let view = run(&settings(&fake.base())).await;
    assert_ready(&view);
    assert_eq!(fake.targets(), ["/key/info"]);
    assert_eq!(codes(&view), ["litellm_no_budget"]);
}

#[tokio::test]
async fn a_key_with_only_a_team_skips_the_user_lookup() {
    let key = json!({ "info": { "spend": 1.0, "user_id": null, "team_id": "team-borealis" } });
    let fake = Fake::routes([
        ("/key/info", Reply::json(&key.to_string())),
        ("/team/info", Reply::json(TEAM)),
    ])
    .await;
    let view = run(&settings(&fake.base())).await;
    assert_ready(&view);
    assert_eq!(fake.targets(), ["/key/info", "/team/info?team_id=team-borealis&key_limit=1"]);
}

#[tokio::test]
async fn a_key_budget_only_in_a_model_cap_still_counts_as_its_own() {
    let key = json!({ "info": {
        "spend": 1.0, "max_budget": null, "user_id": "user-aurora", "team_id": "team-borealis",
        "model_max_budget_usage": { "gpt-fixture-a": { "current_spend": 1.0, "budget_limit": 4.0 } },
    }});
    let fake = Fake::routes([
        ("/key/info", Reply::json(&key.to_string())),
        ("/user/info", Reply::json(USER)),
    ])
    .await;
    let view = run(&settings(&fake.base())).await;
    assert_eq!(ids(&view), ["model:gpt-fixture-a"]);
    assert_eq!(fake.targets(), ["/key/info"]);
}
