//! Ids come from settings and from the server; whatever they hold must arrive at the proxy as the
//! same text, without changing the shape of the request.

use serde_json::json;
use usage_model::settings::LitellmScope;

use crate::common::{Fake, Reply, assert_ready, run, scoped, settings, window};

/// A proxy that answers any user or team id with a record for exactly that id.
async fn echoing_proxy() -> Fake {
    Fake::start(|seen| {
        let (field, container) = match seen.path() {
            "/user/info" => ("user_id", "user_info"),
            "/team/info" => ("team_id", "team_info"),
            _ => return Reply::json_status(404, "{}"),
        };
        let Some(id) = seen.param(field) else { return Reply::json_status(400, "{}") };
        let record =
            json!({ field: id, container: { field: id, "spend": 1.0, "max_budget": 4.0 } });
        Reply::json(&record.to_string())
    })
    .await
}

fn hard_ids() -> Vec<String> {
    let mut ids: Vec<String> = [
        "plain-id",
        "ana@example.test",
        "équipe",
        "a/b",
        "a+b",
        "a b",
        "a&b=c#d?e",
        "日本語",
        "🙂 smile",
        "%41",
        "100%",
        "../etc/passwd",
        "semi;colon",
        "quote\"and'tick",
        "back\\slash",
        "key=value&other=1",
    ]
    .iter()
    .map(|id| (*id).to_owned())
    .collect();
    ids.push("🙂".repeat(256));
    ids
}

#[tokio::test]
async fn user_ids_arrive_exactly_as_typed() {
    let fake = echoing_proxy().await;
    for id in hard_ids() {
        let view = run(&scoped(&fake.base(), LitellmScope::User, &id)).await;
        assert_ready(&view);
        assert_eq!(window(&view, "budget").used_pct, Some(25.0), "{id}");
        let seen = fake.requests().pop().unwrap();
        assert_eq!(seen.param("user_id").as_deref(), Some(id.as_str()), "{id}");
        assert_eq!(seen.path(), "/user/info", "{id}: the id must not change the path");
        assert_eq!(seen.query().matches('&').count(), 0, "{id}: one parameter, one separator");
        assert!(!seen.target.contains([' ', '#', '+', '"', '\\', ';']), "{}", seen.target);
    }
}

#[tokio::test]
async fn team_ids_arrive_exactly_as_typed_next_to_the_key_limit() {
    let fake = echoing_proxy().await;
    for id in hard_ids() {
        let view = run(&scoped(&fake.base(), LitellmScope::Team, &id)).await;
        assert_ready(&view);
        let seen = fake.requests().pop().unwrap();
        assert_eq!(seen.param("team_id").as_deref(), Some(id.as_str()), "{id}");
        assert_eq!(seen.param("key_limit").as_deref(), Some("1"));
        assert_eq!(seen.path(), "/team/info");
        assert_eq!(seen.query().matches('&').count(), 1, "{id}");
    }
}

#[tokio::test]
async fn ids_the_server_names_are_encoded_the_same_way_in_follow_ups() {
    let user = "équipe/α β+c";
    let team = "100% & more #1";
    let key = json!({ "info": { "spend": 1.0, "user_id": user, "team_id": team } });
    let fake = Fake::start(move |seen| match seen.path() {
        "/key/info" => Reply::json(&key.to_string()),
        "/user/info" => {
            let id = seen.param("user_id").unwrap_or_default();
            Reply::json(&json!({ "user_id": id, "user_info": { "user_id": id } }).to_string())
        }
        "/team/info" => {
            let id = seen.param("team_id").unwrap_or_default();
            let record = json!({ "team_id": id, "team_info": { "team_id": id, "max_budget": 8 } });
            Reply::json(&record.to_string())
        }
        _ => Reply::json_status(404, "{}"),
    })
    .await;
    let view = run(&settings(&fake.base())).await;
    assert_ready(&view);
    let requests = fake.requests();
    assert_eq!(requests.len(), 3);
    assert_eq!(requests[1].param("user_id").as_deref(), Some(user));
    assert_eq!(requests[2].param("team_id").as_deref(), Some(team));
    assert!(view.notices.is_empty(), "both ids round-tripped: {:?}", view.notices);
}

#[tokio::test]
async fn an_id_over_the_length_limit_is_refused_without_being_sent() {
    let fake = echoing_proxy().await;
    let view = run(&scoped(&fake.base(), LitellmScope::User, &"x".repeat(257))).await;
    assert_eq!(view.state, usage_model::ViewState::Error);
    fake.assert_untouched().await;
}
