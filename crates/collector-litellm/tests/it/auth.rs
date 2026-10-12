//! The key travels only in the configured header: as a bearer token in `Authorization`, raw in a
//! custom header, and nowhere else.

use serde_json::json;
use usage_model::settings::{LitellmScope, LitellmSettings};
use usage_support::Secret;

use crate::common::{Fake, NOW, Reply, SECRET, assert_ready, run, run_with, scoped, settings};

fn carriers(fake: &Fake) -> Vec<Vec<String>> {
    fake.requests()
        .iter()
        .map(|seen| {
            let names = seen.headers.iter().filter(|(_, value)| value.contains(SECRET));
            names.map(|(name, _)| name.to_ascii_lowercase()).collect()
        })
        .collect()
}

async fn proxy() -> Fake {
    Fake::routes([("/key/info", Reply::json(fixture!("key_with_budget.json")))]).await
}

#[tokio::test]
async fn authorization_carries_the_key_as_a_bearer_token_in_any_capitalisation() {
    for name in ["Authorization", "authorization", "AUTHORIZATION", "  Authorization \t"] {
        let fake = proxy().await;
        let view =
            run(&LitellmSettings { auth_header: name.to_owned(), ..settings(&fake.base()) }).await;
        assert_ready(&view);
        let requests = fake.requests();
        let [seen] = requests.as_slice() else { panic!("one request: {requests:?}") };
        let bearer = format!("Bearer {SECRET}");
        assert_eq!(seen.header("authorization"), Some(bearer.as_str()), "{name:?}");
        assert_eq!(carriers(&fake), [["authorization"]], "{name:?}: only one header holds the key");
    }
}

#[tokio::test]
async fn a_custom_header_carries_the_raw_key_and_authorization_is_not_sent() {
    for name in ["X-Litellm-Api-Key", "x-litellm-api-key", "api-key"] {
        let fake = proxy().await;
        let view =
            run(&LitellmSettings { auth_header: name.to_owned(), ..settings(&fake.base()) }).await;
        assert_ready(&view);
        let seen = &fake.requests()[0];
        assert_eq!(seen.header(name), Some(SECRET), "{name}");
        assert_eq!(seen.header("authorization"), None, "{name}");
        assert_eq!(carriers(&fake), [[name.to_ascii_lowercase()]]);
    }
}

#[tokio::test]
async fn follow_ups_use_the_same_header_as_the_first_request() {
    let key = fixture!("key_follow_up.json");
    let fake = Fake::routes([
        ("/key/info", Reply::json(key)),
        ("/user/info", Reply::json(fixture!("user_info_no_budget.json"))),
        ("/team/info", Reply::json(fixture!("team_info.json"))),
    ])
    .await;
    let settings =
        LitellmSettings { auth_header: "X-Litellm-Api-Key".to_owned(), ..settings(&fake.base()) };
    assert_ready(&run(&settings).await);
    assert_eq!(fake.requests().len(), 3);
    assert_eq!(carriers(&fake), [["x-litellm-api-key"]; 3]);
}

#[tokio::test]
async fn the_key_is_never_part_of_a_url() {
    let fake = Fake::routes([
        ("/key/info", Reply::json(fixture!("key_follow_up.json"))),
        ("/user/info", Reply::json(fixture!("user_info_no_budget.json"))),
        ("/team/info", Reply::json(fixture!("team_info_no_budget.json"))),
    ])
    .await;
    run(&settings(&fake.base())).await;
    run(&scoped(&fake.base(), LitellmScope::User, "user-aurora")).await;
    run(&scoped(&fake.base(), LitellmScope::Team, "team-borealis")).await;
    assert_eq!(fake.requests().len(), 3 + 1 + 1);
    assert!(fake.targets().iter().all(|target| !target.contains(SECRET)), "{:?}", fake.targets());
}

#[tokio::test]
async fn an_id_that_holds_the_key_is_not_asked_about() {
    let key =
        json!({ "info": { "spend": 1.0, "user_id": SECRET, "team_id": format!("t-{SECRET}") } });
    let fake = Fake::routes([
        ("/key/info", Reply::json(&key.to_string())),
        ("/user/info", Reply::json(fixture!("user_info.json"))),
        ("/team/info", Reply::json(fixture!("team_info.json"))),
    ])
    .await;
    let view = run(&settings(&fake.base())).await;
    assert_ready(&view);
    assert_eq!(fake.targets(), ["/key/info"], "following would write the key into a URL");
}

#[tokio::test]
async fn a_key_saved_with_edge_spaces_is_still_kept_out_of_urls() {
    // A proxy sees the key without its edge spaces, so ids are compared against that form.
    let padded = Secret::new(format!("  {SECRET}  "));
    let key = json!({ "info": {
        "spend": 1.0, "user_id": format!("u-{SECRET}"), "team_id": format!("t-{SECRET}"),
    }});
    let fake = Fake::routes([
        ("/key/info", Reply::json(&key.to_string())),
        ("/user/info", Reply::json(fixture!("user_info.json"))),
        ("/team/info", Reply::json(fixture!("team_info.json"))),
    ])
    .await;
    let view = run_with(&settings(&fake.base()), Some(&padded), NOW).await;
    assert_ready(&view);
    assert_eq!(fake.targets(), ["/key/info"], "following would write the key into a URL");
}
