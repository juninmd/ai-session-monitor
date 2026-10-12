//! Several addresses to try: the first that answers ends the poll, the others only cost time, and
//! when none answers the error that says the most is the one reported.

use usage_model::{ErrorCode, ViewState};
use usage_support::Secret;

use super::*;
use crate::discovery::Scheme;
use crate::fake::{Reply, closed_port};

/// Short enough that a silent address is cheap, long enough for a loaded CI machine.
fn quick() -> Config {
    timed(Duration::from_millis(400), Duration::from_secs(20))
}

async fn polled(targets: Vec<Target>) -> HarnessView {
    poll_with(&on(), &StaticDiscovery::new(Ok(targets)), &quick()).await
}

/// The address of `server`, spoken to with `scheme` whatever the server really speaks.
fn aimed(server: &FakeServer, scheme: Scheme) -> Target {
    Target { scheme, ..server.target(TOKEN) }
}

async fn answering(reply: Reply) -> FakeServer {
    FakeServer::http(move |_| reply.clone()).await
}

#[tokio::test]
async fn the_first_server_that_answers_ends_the_probing() {
    let first = serve("status_ide", Some("summary_full")).await;
    let second = serve("status_ide", Some("summary_full")).await;
    let view = polled(vec![first.target(TOKEN), second.target(TOKEN)]).await;
    assert_eq!(view.state, ViewState::Ready);
    assert_eq!(first.requests().len(), 2);
    assert_eq!(second.connections(), 0, "no address after the answer is tried");
}

#[tokio::test]
async fn addresses_that_cannot_answer_are_skipped_in_favour_of_the_one_that_can() {
    let good = serve("status_ide", Some("summary_full")).await;
    let hangs_up = answering(Reply::Hangup).await;
    let silent = answering(Reply::Silent).await;
    let web_page = answering(Reply::Html(200, "<h1>not it</h1>".to_owned())).await;
    let not_json = answering(Reply::json(200, r#"{"hello":"world"}"#)).await;
    let junk = answering(Reply::Raw(b"\x00\x01 not http".to_vec())).await;
    let targets = [&hangs_up, &silent, &web_page, &not_json, &junk, &good]
        .map(|server| server.target(TOKEN))
        .to_vec();

    let view = polled(targets).await;

    assert_eq!(view.state, ViewState::Ready);
    assert_eq!(view.windows.len(), 4);
    assert_eq!(view.plan.as_deref(), Some("Google AI Pro"));
    assert!(view.error.is_none());
}

#[tokio::test]
async fn a_server_that_refuses_the_token_does_not_stop_the_next_address() {
    let script = Script::new("someone-elses-token", text("status_ide"), Some(text("summary_full")));
    let other = script.serve_http().await;
    let ours = serve("status_ide", Some("summary_full")).await;
    let view = polled(vec![other.target(TOKEN), ours.target(TOKEN)]).await;
    assert_eq!(view.state, ViewState::Ready);
    assert_eq!(other.requests().len(), 1, "one refused question, no more");
}

#[tokio::test]
async fn tls_spoken_to_a_plain_port_and_plain_http_spoken_to_a_tls_port_both_move_on() {
    let plain = serve("status_ide", Some("summary_full")).await;
    let tls =
        Script::new(TOKEN, text("status_ide"), Some(text("summary_full"))).serve_https().await;
    let good = serve("status_ide", Some("summary_full")).await;
    let targets = vec![aimed(&plain, Scheme::Https), aimed(&tls, Scheme::Http), good.target(TOKEN)];

    let view = polled(targets).await;

    assert_eq!(view.state, ViewState::Ready);
    assert!(plain.requests().is_empty(), "a TLS hello is not an HTTP request");
    assert!(tls.requests().is_empty(), "a failed handshake serves nothing");
}

#[tokio::test]
async fn a_port_nobody_listens_on_is_skipped() {
    let dead =
        Target { scheme: Scheme::Https, port: closed_port().await, token: Secret::new(TOKEN) };
    let good = serve("status_ide", Some("summary_full")).await;
    let view = polled(vec![dead, good.target(TOKEN)]).await;
    assert_eq!(view.state, ViewState::Ready);
}

#[tokio::test]
async fn when_every_address_fails_the_error_that_says_most_wins_in_either_order() {
    let html = || Reply::Html(200, "<p>x</p>".to_owned());
    let cases = [
        (Reply::Hangup, Reply::status(401), ErrorCode::Unauthorized, false),
        (Reply::status(500), Reply::Hangup, ErrorCode::ServerError, true),
        (Reply::status(429), Reply::status(500), ErrorCode::RateLimited, true),
        (html(), Reply::status(404), ErrorCode::BadResponse, false),
        (Reply::status(404), Reply::Hangup, ErrorCode::EndpointChanged, false),
        (Reply::json(200, "{}"), Reply::Hangup, ErrorCode::EndpointChanged, false),
        (Reply::Hangup, Reply::Hangup, ErrorCode::Network, true),
    ];
    for (a, b, code, retryable) in cases {
        for (first, second) in [(a.clone(), b.clone()), (b, a)] {
            let label = format!("{first:?} then {second:?}");
            let (one, two) = (answering(first).await, answering(second).await);
            let view = polled(vec![one.target(TOKEN), two.target(TOKEN)]).await;
            let error = view.error.as_ref().expect("every address failed");
            assert_eq!(
                (view.state, error.code, error.retryable),
                (ViewState::Error, code, retryable),
                "{label}"
            );
        }
    }
}

#[tokio::test]
async fn nothing_to_answer_on_the_only_port_is_retryable() {
    let dead =
        Target { scheme: Scheme::Http, port: closed_port().await, token: Secret::new(TOKEN) };
    let view = polled(vec![dead]).await;
    let error = view.error.as_ref().expect("nothing answered");
    // A refused connection is `Network`; a platform that stalls the refusal makes it `Timeout`.
    assert!(matches!(error.code, ErrorCode::Network | ErrorCode::Timeout), "{:?}", error.code);
    assert!(error.retryable);
    assert!(view.windows.is_empty());
}
