//! Where the request goes: a path prefix on the address is kept, the host is shown as the user
//! typed it, and surrounding whitespace does not matter.

use usage_model::settings::LitellmScope;

use crate::common::{Fake, Reply, assert_ready, meta, run, scoped, settings};

#[tokio::test]
async fn a_path_prefix_in_the_address_is_kept_in_front_of_the_endpoint() {
    let cases = [
        ("", "/key/info"),
        ("/", "/key/info"),
        ("/litellm", "/litellm/key/info"),
        ("/litellm/", "/litellm/key/info"),
        ("/a/b/c", "/a/b/c/key/info"),
        ("/team-é/ß", "/team-%C3%A9/%C3%9F/key/info"),
    ];
    for (prefix, path) in cases {
        let fake = Fake::routes([(path, Reply::json(fixture!("key_with_budget.json")))]).await;
        let view = run(&settings(&format!("{}{prefix}", fake.base()))).await;
        assert_ready(&view);
        assert_eq!(fake.targets(), [path], "prefix {prefix:?}");
    }
}

#[tokio::test]
async fn every_scope_and_follow_up_lives_under_the_prefix() {
    let fake = Fake::routes([
        ("/gw/v1/key/info", Reply::json(fixture!("key_follow_up.json"))),
        ("/gw/v1/user/info", Reply::json(fixture!("user_info_no_budget.json"))),
        ("/gw/v1/team/info", Reply::json(fixture!("team_info.json"))),
    ])
    .await;
    let base = format!("{}/gw/v1/", fake.base());
    assert_ready(&run(&settings(&base)).await);
    assert_eq!(
        fake.targets(),
        [
            "/gw/v1/key/info",
            "/gw/v1/user/info?user_id=user-aurora",
            "/gw/v1/team/info?team_id=team-borealis&key_limit=1",
        ]
    );
    assert_ready(&run(&scoped(&base, LitellmScope::User, "user-aurora")).await);
    assert_ready(&run(&scoped(&base, LitellmScope::Team, "team-borealis")).await);
    assert_eq!(fake.requests().len(), 5);
}

#[tokio::test]
async fn whitespace_around_the_address_is_ignored() {
    let fake = Fake::routes([("/key/info", Reply::json(fixture!("key_with_budget.json")))]).await;
    let view = run(&settings(&format!("  {}\t\r\n", fake.base()))).await;
    assert_ready(&view);
}

#[tokio::test]
async fn localhost_works_over_plain_http_and_is_shown_lowercase() {
    let fake = Fake::routes([("/key/info", Reply::json(fixture!("key_with_budget.json")))]).await;
    let port = fake.base().rsplit(':').next().unwrap().to_owned();
    let view = run(&settings(&format!("http://LocalHost:{port}"))).await;
    assert_ready(&view);
    assert_eq!(meta(&view).get("host"), Some(&"localhost"));
    assert_eq!(fake.requests()[0].header("host"), Some(format!("localhost:{port}").as_str()));
}

#[tokio::test]
async fn the_shown_host_has_no_port_path_or_credentials() {
    let fake =
        Fake::routes([("/litellm/key/info", Reply::json(fixture!("key_with_budget.json")))]).await;
    let view = run(&settings(&format!("{}/litellm", fake.base()))).await;
    assert_eq!(meta(&view).get("host"), Some(&"127.0.0.1"));
}
