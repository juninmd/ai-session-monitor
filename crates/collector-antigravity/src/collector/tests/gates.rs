//! A source that is off, or not consented to, touches nothing: no discovery, no connection, no file.

use usage_model::{HarnessId, ViewState};

use super::*;

/// Everything a poll could reach is armed: a live server, a discovery that would return it and a
/// real directory the context points at. None of it may be used.
async fn assert_untouched(settings: AntigravitySettings, expected: ViewState) {
    let server = serve("status_ide", Some("summary_full")).await;
    let discovery = StaticDiscovery::new(Ok(vec![server.target(TOKEN)]));
    let cli = ScriptedCli::new(Ok(crate::fixtures::json("cli_usage")));
    let dir = tempfile::tempdir().unwrap();
    let ctx = Ctx::new(NOW, dir.path(), dir.path());

    let view = collect_with(&settings, &ctx, &discovery, &cli, &config()).await;

    assert_eq!(view.id, HarnessId::Antigravity);
    assert_eq!(view.state, expected);
    assert!(view.windows.is_empty() && view.notices.is_empty() && view.facts.is_empty());
    assert!(view.source.is_none() && view.error.is_none() && view.read_at.is_none());
    assert_eq!(discovery.calls(), 0, "discovery must not run");
    assert_eq!(cli.calls(), 0, "the CLI must not run");
    tokio::time::sleep(Duration::from_millis(150)).await;
    assert_eq!(server.connections(), 0, "no connection may be opened");
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0, "no file may be written");
}

#[tokio::test]
async fn a_disabled_source_is_disabled_whatever_else_is_set() {
    for consent in [true, false] {
        assert_untouched(AntigravitySettings { enabled: false, consent }, ViewState::Disabled)
            .await;
    }
}

#[tokio::test]
async fn an_enabled_source_without_consent_asks_for_it_and_reads_nothing() {
    assert_untouched(
        AntigravitySettings { enabled: true, consent: false },
        ViewState::NeedsConsent,
    )
    .await;
}

#[tokio::test]
async fn consent_opens_the_gate() {
    let server = serve("status_ide", Some("summary_full")).await;
    let view = poll(vec![server.target(TOKEN)]).await;
    assert_eq!(view.state, ViewState::Ready);
    assert!(server.connections() > 0);
}

#[tokio::test]
async fn the_future_can_be_sent_between_threads() {
    fn assert_send<T: Send>(_: &T) {}
    let (discovery, cli) = (crate::discovery::SystemDiscovery::new(), AbsentCli);
    let (settings, ctx, config) = (on(), ctx(), config());
    let future = collect_with(&settings, &ctx, &discovery, &cli, &config);
    assert_send(&future);
    drop(future);
}
