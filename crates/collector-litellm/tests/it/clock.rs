//! The clock edges: `read_at` is the poll's own clock, and a view is stale once every window has
//! already reset (at the very second included), while its windows stay visible.

use serde_json::json;
use usage_model::settings::LitellmScope;
use usage_support::Secret;

use crate::common::{
    Fake, NOW, Reply, SECRET, assert_ready, ids, run, run_with, scoped, settings, window,
};

async fn resetting_at(reset: &str, now: i64) -> usage_model::HarnessView {
    let key = json!({ "info": {
        "spend": 5.0, "max_budget": 10.0, "budget_duration": "1d", "budget_reset_at": reset,
    }});
    let fake = Fake::routes([("/key/info", Reply::json(&key.to_string()))]).await;
    run_with(&settings(&fake.base()), Some(&Secret::new(SECRET)), now).await
}

#[tokio::test]
async fn a_window_is_stale_from_the_second_it_resets() {
    let cases = [
        ("2026-10-11T12:00:01Z", false, "one second to go"),
        ("2026-10-11T12:00:00Z", true, "resetting right now"),
        ("2026-10-11T11:59:59Z", true, "one second ago"),
        ("2026-10-12T00:00:00Z", false, "tomorrow"),
        ("2026-10-11T14:00:00+02:00", true, "now, written in UTC+2"),
        ("2026-10-11T14:00:01+02:00", false, "one second to go, in UTC+2"),
        ("2026-10-11T06:00:00-06:00", true, "now, written in UTC-6"),
        ("2026-10-11T12:00:01.000000+00:00", false, "python isoformat with microseconds"),
        ("0001-01-01T00:00:00Z", true, "the earliest representable date"),
        ("9999-12-31T23:59:59Z", false, "the latest representable date"),
    ];
    for (reset, stale, why) in cases {
        let view = resetting_at(reset, NOW).await;
        assert_ready(&view);
        assert_eq!(view.stale, stale, "{reset}: {why}");
        assert_eq!(view.windows.len(), 1, "a stale view keeps its windows");
        assert_eq!(window(&view, "budget").used_pct, Some(50.0));
    }
}

#[tokio::test]
async fn an_unreadable_reset_time_never_makes_a_view_stale() {
    for reset in ["tomorrow", "", "2026-13-45T99:99:99Z", "1793491200", "2026-10-11"] {
        let view = resetting_at(reset, NOW).await;
        assert!(!view.stale, "{reset:?}");
        assert_eq!(window(&view, "budget").resets_at, None, "{reset:?}");
    }
    let fake = Fake::routes([(
        "/key/info",
        Reply::json(r#"{"info":{"spend":1,"max_budget":2,"budget_reset_at":1793491200}}"#),
    )])
    .await;
    let view = run(&settings(&fake.base())).await;
    assert!(!view.stale && window(&view, "budget").resets_at.is_none(), "a number is not a date");
}

#[tokio::test]
async fn read_at_is_the_poll_clock_not_the_servers() {
    for now in [NOW, 0, 1, -86_400, 32_503_680_000, i64::MAX] {
        let view = resetting_at("2026-11-01T00:00:00Z", now).await;
        assert_eq!(view.read_at, Some(now), "now = {now}");
    }
}

#[tokio::test]
async fn extreme_clocks_neither_panic_nor_invent_a_reset() {
    // The epoch, a negative clock and the far past: the reset is in the future.
    for now in [0, -1, i64::MIN / 2] {
        assert!(!resetting_at("2026-11-01T00:00:00Z", now).await.stale, "now = {now}");
    }
    // The far future and the largest clock: every reset has happened.
    for now in [32_503_680_000, i64::MAX] {
        assert!(resetting_at("2026-11-01T00:00:00Z", now).await.stale, "now = {now}");
    }
}

#[tokio::test]
async fn only_when_every_window_has_reset_is_the_view_stale() {
    let key = json!({ "info": {
        "spend": 5.0, "max_budget": 10.0, "budget_reset_at": "2026-10-11T11:00:00Z",
        "budget_limits": [{ "budget_duration": "1h", "max_budget": 4.0, "reset_at": "2026-10-11T13:00:00Z" }],
        "budget_limits_usage": { "1h": { "current_spend": 1.0 } },
        "model_max_budget_usage": { "gpt-fixture-a": { "current_spend": 1.0, "budget_limit": 2.0 } },
    }});
    let fake = Fake::routes([("/key/info", Reply::json(&key.to_string()))]).await;
    let view = run(&settings(&fake.base())).await;
    assert_eq!(ids(&view), ["budget", "model:gpt-fixture-a", "limit:1h"]);
    assert!(!view.stale, "the hourly window has not reset and the model cap never does");

    let all_past = json!({ "info": {
        "spend": 5.0, "max_budget": 10.0, "budget_reset_at": "2026-10-11T11:00:00Z",
        "budget_limits": [{ "budget_duration": "1h", "max_budget": 4.0, "reset_at": "2026-10-11T11:30:00Z" }],
        "budget_limits_usage": { "1h": { "current_spend": 1.0 } },
    }});
    let fake = Fake::routes([("/key/info", Reply::json(&all_past.to_string()))]).await;
    let view = run(&settings(&fake.base())).await;
    assert_eq!(ids(&view), ["budget", "limit:1h"]);
    assert!(view.stale);
}

#[tokio::test]
async fn a_followed_level_is_judged_by_its_own_windows() {
    // The key has no budget and no reset; the team's budget reset an hour ago.
    let team = json!({
        "team_id": "team-borealis",
        "team_info": {
            "team_id": "team-borealis", "spend": 9.0, "max_budget": 10.0,
            "budget_reset_at": "2026-10-11T11:00:00Z",
        },
    });
    let fake = Fake::routes([
        ("/key/info", Reply::json(&json!({ "info": { "team_id": "team-borealis" } }).to_string())),
        ("/team/info", Reply::json(&team.to_string())),
    ])
    .await;
    let view = run(&settings(&fake.base())).await;
    assert_ready(&view);
    assert!(view.stale);
    assert_eq!(window(&view, "budget").used_pct, Some(90.0));
}

#[tokio::test]
async fn the_user_and_team_scopes_use_the_same_clock_rules() {
    let user = json!({
        "user_id": "user-aurora",
        "user_info": { "user_id": "user-aurora", "spend": 1.0, "max_budget": 2.0, "budget_reset_at": "2026-10-11T12:00:00Z" },
    });
    let fake = Fake::routes([("/user/info", Reply::json(&user.to_string()))]).await;
    let view = run(&scoped(&fake.base(), LitellmScope::User, "user-aurora")).await;
    assert!(view.stale, "reset at this very second");
    assert_eq!(view.read_at, Some(NOW));
}
