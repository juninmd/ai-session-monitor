//! Clock edges, concurrent polls and secret hygiene across whole polls.

use serde_json::{Value, json};
use usage_model::{Ctx, ViewState};

use super::fake_db::NOW;
use super::fake_server::{FakeServer, Reply};
use super::flow_support::{loopback, ok_server, run, run_at, window_ids};
use super::local_support::populated;
use super::{KEY, ctx, iso, secret, settings};
use crate::collect_with;

/// A usage body with the given windows; `None` leaves the reset time out.
fn body(windows: &[(&str, Option<i64>)]) -> String {
    let mut usage = serde_json::Map::new();
    for (name, reset) in windows {
        let mut window = json!({ "percent": 40 });
        if let Some(secs) = reset {
            window["resetsAt"] = Value::String(iso(*secs));
        }
        usage.insert((*name).to_owned(), window);
    }
    json!({ "usage": usage }).to_string()
}

fn serving_body(text: String) -> FakeServer {
    FakeServer::always(move || Reply::json(200, &text))
}

#[tokio::test]
async fn a_view_is_stale_only_when_every_window_is_known_to_have_reset() {
    let (rolling, weekly, monthly) = ("rolling", "weekly", "monthly");
    let cases = [
        (
            vec![
                (rolling, Some(NOW + 60)),
                (weekly, Some(NOW + 3_600)),
                (monthly, Some(NOW + 86_400)),
            ],
            false,
        ),
        (
            vec![
                (rolling, Some(NOW - 60)),
                (weekly, Some(NOW - 3_600)),
                (monthly, Some(NOW - 86_400)),
            ],
            true,
        ),
        (vec![(rolling, Some(NOW)), (weekly, Some(NOW)), (monthly, Some(NOW))], true),
        (
            vec![(rolling, Some(NOW - 60)), (weekly, Some(NOW - 60)), (monthly, Some(NOW + 1))],
            false,
        ),
        (vec![(rolling, Some(NOW - 60)), (weekly, None), (monthly, Some(NOW - 60))], false),
        (vec![(rolling, Some(NOW - 1))], true),
        (vec![(rolling, None)], false),
    ];
    for (windows, stale) in cases {
        let (server, home) = (serving_body(body(&windows)), tempfile::tempdir().unwrap());
        let view = run(&settings(false, None), Some(&secret()), &server, home.path()).await;
        assert_eq!(view.state, ViewState::Ready, "{windows:?}");
        assert_eq!(view.stale, stale, "{windows:?}");
        assert_eq!(view.windows.len(), windows.len(), "windows stay visible even when stale");
        assert!(view.windows.iter().all(|w| w.used_pct == Some(40.0)));
    }
}

#[tokio::test]
async fn a_window_that_expires_between_two_polls_turns_the_view_stale_keeping_its_numbers() {
    let reset = NOW + 1_000;
    let (server, home) =
        (serving_body(body(&[("rolling", Some(reset))])), tempfile::tempdir().unwrap());
    let mut seen = Vec::new();
    for now in [reset - 1, reset, reset + 1] {
        let at = Ctx::new(now, home.path().join("app-data"), home.path());
        let view = run_at(&settings(false, None), Some(&secret()), &server, &at).await;
        assert_eq!((view.read_at, window_ids(&view)), (Some(now), vec!["rolling"]));
        assert_eq!(view.windows[0].used_pct, Some(40.0));
        seen.push(view.stale);
    }
    assert_eq!(seen, [false, true, true]);
}

#[tokio::test]
async fn concurrent_polls_do_not_interfere() {
    let (server, fake, home) = (ok_server(), populated(), tempfile::tempdir().unwrap());
    let (key_only, consent_only, both) =
        (settings(false, None), settings(true, Some(fake.dir())), settings(true, Some(fake.dir())));
    let (key, at) = (secret(), ctx(home.path()));
    let endpoint = loopback(&server);
    let (first, second, third) = tokio::join!(
        collect_with(&key_only, Some(&key), &at, &endpoint),
        collect_with(&consent_only, None, &at, &endpoint),
        collect_with(&both, Some(&key), &at, &endpoint),
    );
    assert_eq!(server.requests().len(), 2, "only the two polls with a key reach the network");
    assert_eq!(first, collect_with(&key_only, Some(&key), &at, &endpoint).await);
    assert_eq!(second, collect_with(&consent_only, None, &at, &endpoint).await);
    assert_eq!(third, collect_with(&both, Some(&key), &at, &endpoint).await);
    assert_eq!((first.facts.len(), second.facts.len(), third.facts.len()), (0, 5, 5));
}

/// Server replies that echo the key and database text wherever they can.
fn leaky_servers() -> Vec<FakeServer> {
    let json_error = format!(r#"{{"error":"rejected {KEY}","key":"{KEY}"}}"#);
    let echoing = format!(
        r#"{{"usage":{{"rolling":{{"percent":10,"status":"ok {KEY}","resetsAt":"{}"}}}},"echo":"{KEY}"}}"#,
        iso(NOW + 600)
    );
    let page = format!("<html><body>{KEY}</body></html>");
    let (err_401, err_403, err_500, err_429) =
        (json_error.clone(), json_error.clone(), json_error.clone(), json_error);
    let location = format!("http://127.0.0.1:9/{KEY}");
    vec![
        FakeServer::always(move || Reply::json(401, &err_401)),
        FakeServer::always(move || Reply::json(403, &err_403)),
        FakeServer::always(move || Reply::json(500, &err_500)),
        FakeServer::always(move || {
            let echo = format!("echo {KEY}");
            Reply::with_headers(
                429,
                &[("Retry-After", "30"), ("X-Echo", &echo), ("Content-Type", "application/json")],
                &err_429,
            )
        }),
        FakeServer::always(move || Reply::json(200, &echoing)),
        FakeServer::always(move || {
            Reply::with_headers(200, &[("Content-Type", "text/html")], &page)
        }),
        FakeServer::always(move || Reply::with_headers(302, &[("Location", &location)], "")),
    ]
}

#[tokio::test]
async fn nothing_a_server_echoes_reaches_the_view() {
    for server in leaky_servers() {
        let (fake, home) = (populated(), tempfile::tempdir().unwrap());
        // `run` fails the test if the key or a database canary shows up anywhere in the view.
        let view =
            run(&settings(true, Some(fake.dir())), Some(&secret()), &server, home.path()).await;
        assert!(matches!(view.state, ViewState::Ready | ViewState::Error), "{:?}", view.state);
        assert_eq!(server.requests().len(), 1);
    }
}
