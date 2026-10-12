//! Nothing the proxy knows but the user did not ask to see may reach a view, and a hostile or
//! careless proxy cannot turn the user's own key into displayed text.

use serde_json::json;
use usage_model::settings::LitellmScope;

use crate::common::{
    Fake, Reply, SECRET, assert_ready, codes, error_of, meta, run, scoped, settings,
};

const SENSITIVE_FIXTURES: [&str; 8] = [
    fixture!("key_with_budget.json"),
    fixture!("key_follow_up.json"),
    fixture!("user_info.json"),
    fixture!("user_info_no_budget.json"),
    fixture!("team_info.json"),
    fixture!("over_budget.json"),
    fixture!("error_page.html"),
    fixture!("model_budgets.json"),
];

#[test]
fn the_leak_check_is_not_vacuous() {
    // `run` rejects any view containing "decoy"; that only means something if the fixtures do.
    let planted = SENSITIVE_FIXTURES.iter().filter(|f| f.to_lowercase().contains("decoy")).count();
    assert_eq!(planted, SENSITIVE_FIXTURES.len(), "every sensitive fixture carries a decoy");
}

#[tokio::test]
async fn private_fields_of_every_record_stay_out_of_the_view() {
    let fake = Fake::routes([
        ("/key/info", Reply::json(fixture!("key_with_budget.json"))),
        ("/user/info", Reply::json(fixture!("user_info.json"))),
        ("/team/info", Reply::json(fixture!("team_info.json"))),
    ])
    .await;
    // `run` checks every view for the key and for decoys; the point here is to cover all scopes.
    assert_ready(&run(&settings(&fake.base())).await);
    assert_ready(&run(&scoped(&fake.base(), LitellmScope::User, "user-aurora")).await);
    assert_ready(&run(&scoped(&fake.base(), LitellmScope::Team, "team-borealis")).await);
}

#[tokio::test]
async fn a_proxy_that_echoes_the_key_into_every_name_shows_none_of_it() {
    let echo = json!({ "info": {
        "key_alias": SECRET, "spend": 1.0, "max_budget": 2.0, "budget_duration": SECRET,
        "model_max_budget_usage": {
            SECRET: { "current_spend": 1.0, "budget_limit": 2.0, "time_period": SECRET },
            format!("gpt-{SECRET}"): { "current_spend": 1.0, "budget_limit": 2.0 },
            "gpt-fixture-a": { "current_spend": 1.0, "budget_limit": 2.0, "time_period": SECRET },
        },
        "budget_limits": [{ "budget_duration": SECRET, "max_budget": 2.0 }],
        "budget_limits_usage": { SECRET: { "current_spend": 1.0 } },
    }});
    let fake = Fake::routes([("/key/info", Reply::json(&echo.to_string()))]).await;
    let view = run(&settings(&fake.base())).await;
    assert_ready(&view);
    let shown: Vec<_> = view.windows.iter().map(|w| (w.id.as_str(), w.label.as_deref())).collect();
    assert_eq!(shown, [("budget", None), ("model:gpt-fixture-a", Some("gpt-fixture-a"))]);
    assert_eq!(meta(&view).get("alias"), None);
}

#[tokio::test]
async fn errors_never_repeat_what_the_proxy_said() {
    let hostile = format!(
        r#"{{"error":{{"message":"bad key {SECRET}","type":"auth_error","detail":"decoy"}}}}"#
    );
    let replies = [
        Reply::json_status(401, &hostile),
        Reply::json_status(500, &hostile),
        Reply::typed(200, "text/html", &hostile),
        Reply::json(&hostile),
        Reply::status(429, &[("Retry-After", SECRET), ("X-Detail", "decoy")]),
        Reply::status(
            302,
            &[("Location", "https://decoy.example.test/?k=sk-fixture-7f3a9c1e5b2d4a60")],
        ),
        Reply::status(401, &[("WWW-Authenticate", "Bearer realm=\"decoy\"")]),
    ];
    for reply in replies {
        let fake = Fake::routes([("/key/info", reply)]).await;
        let view = run(&settings(&fake.base())).await;
        error_of(&view);
    }
}

#[tokio::test]
async fn a_key_pasted_into_the_address_is_refused_and_not_shown() {
    let fake = Fake::routes([("/key/info", Reply::json(fixture!("key_with_budget.json")))]).await;
    for url in [
        format!("{}/?api_key={SECRET}", fake.base()),
        format!("{}#{SECRET}", fake.base()),
        fake.base().replace("http://", &format!("http://{SECRET}@")),
    ] {
        let view = run(&settings(&url)).await;
        let error = error_of(&view);
        assert_eq!(error.code, usage_model::ErrorCode::InvalidUrl);
        assert_eq!(error.params.keys().map(String::as_str).collect::<Vec<_>>(), ["host"]);
    }
    fake.assert_untouched().await;
}

#[tokio::test]
async fn a_ready_view_never_reports_the_proxy_as_having_no_budget_when_it_could_not_look() {
    let fake = Fake::routes([
        ("/key/info", Reply::json(fixture!("key_follow_up.json"))),
        ("/user/info", Reply::json_status(500, "{}")),
        ("/team/info", Reply::json_status(500, "{}")),
    ])
    .await;
    let view = run(&settings(&fake.base())).await;
    assert_ready(&view);
    assert!(!codes(&view).contains(&"litellm_no_budget"), "{:?}", view.notices);
}
