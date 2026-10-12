//! The gates run before any I/O: switched off, or neither a key nor local consent. A gated call
//! must not open a socket or touch a file; the fake server and a directory snapshot prove it.

use std::path::Path;

use usage_model::settings::OpencodeSettings;
use usage_model::{HarnessId, HarnessView};
use usage_support::Secret;

use super::fake_db::FakeDb;
use super::flow_support::{loopback, ok_server};
use super::local_support::populated;
use super::{ctx, secret, settings, snapshot};
use crate::{Plan, collect_with, plan};

const BLANK_KEYS: [&str; 5] = ["", " ", "\t\r\n", "\u{a0}\u{2003}", "   \n   "];

fn gate(settings: &OpencodeSettings, key: Option<&Secret>) -> Option<HarnessView> {
    match plan(settings, key) {
        Plan::Done(view) => Some(view),
        Plan::Read { .. } => None,
    }
}

fn opens(settings: &OpencodeSettings, key: Option<&Secret>) -> Option<(bool, bool)> {
    match plan(settings, key) {
        Plan::Done(_) => None,
        Plan::Read { key, local } => Some((key.is_some(), local)),
    }
}

#[test]
fn a_disabled_harness_is_never_read_whatever_else_is_set() {
    let key = secret();
    for (key, consent) in [(None, false), (None, true), (Some(&key), false), (Some(&key), true)] {
        let mut off = settings(consent, None);
        off.enabled = false;
        assert_eq!(gate(&off, key), Some(HarnessView::disabled(HarnessId::Opencode)));
    }
}

#[test]
fn neither_a_key_nor_consent_needs_setup() {
    let needs_setup = Some(HarnessView::needs_setup(HarnessId::Opencode));
    assert_eq!(gate(&settings(false, None), None), needs_setup);
    for blank in BLANK_KEYS {
        assert_eq!(
            gate(&settings(false, None), Some(&Secret::new(blank))),
            needs_setup,
            "{blank:?}"
        );
    }
}

#[test]
fn a_key_or_consent_each_open_the_gate_for_their_own_source() {
    let key = secret();
    assert_eq!(opens(&settings(false, None), Some(&key)), Some((true, false)));
    assert_eq!(opens(&settings(true, None), None), Some((false, true)));
    assert_eq!(opens(&settings(true, None), Some(&key)), Some((true, true)));
}

#[test]
fn a_blank_key_is_no_key_so_consent_reads_the_estimate_alone() {
    for blank in BLANK_KEYS {
        assert_eq!(
            opens(&settings(true, None), Some(&Secret::new(blank))),
            Some((false, true)),
            "{blank:?}"
        );
    }
}

#[tokio::test]
async fn gated_calls_open_no_socket_and_touch_no_file() {
    let (server, fake, home) = (ok_server(), populated(), tempfile::tempdir().unwrap());
    let (data_before, home_before) = (snapshot(fake.dir()), snapshot(home.path()));
    let key = secret();

    let mut off = settings(true, Some(fake.dir()));
    off.enabled = false;
    let blank = Secret::new("  ");
    let cases = [
        (off, Some(&key), HarnessView::disabled(HarnessId::Opencode)),
        (settings(false, Some(fake.dir())), None, HarnessView::needs_setup(HarnessId::Opencode)),
        (
            settings(false, Some(fake.dir())),
            Some(&blank),
            HarnessView::needs_setup(HarnessId::Opencode),
        ),
    ];
    for (gated, api_key, want) in cases {
        assert_eq!(
            collect_with(&gated, api_key, &ctx(home.path()), &loopback(&server)).await,
            want
        );
    }
    server.assert_untouched();
    assert_eq!(snapshot(fake.dir()), data_before);
    assert_eq!(snapshot(home.path()), home_before);
}

#[tokio::test]
async fn the_public_entry_point_applies_the_same_gates() {
    let home = tempfile::tempdir().unwrap();
    let mut off = settings(true, None);
    off.enabled = false;
    let view = crate::collect(&off, Some(&secret()), &ctx(home.path())).await;
    assert_eq!(view, HarnessView::disabled(HarnessId::Opencode));
    let view = crate::collect(&settings(false, None), None, &ctx(home.path())).await;
    assert_eq!(view, HarnessView::needs_setup(HarnessId::Opencode));
}

#[tokio::test]
async fn consent_without_a_key_never_opens_a_socket() {
    let (server, fake, home) = (ok_server(), populated(), tempfile::tempdir().unwrap());
    let view = collect_with(
        &settings(true, Some(fake.dir())),
        None,
        &ctx(home.path()),
        &loopback(&server),
    )
    .await;
    assert_eq!(view.facts.len(), 5);
    server.assert_untouched();
}

#[tokio::test]
async fn a_key_without_consent_never_resolves_or_opens_the_database() {
    let (server, home) = (ok_server(), tempfile::tempdir().unwrap());
    let empty = FakeDb::bare();
    // Any access would surface as a notice: a bad path, or a missing file.
    for dir in [Some(empty.dir()), Some(Path::new("relative/dir")), None] {
        let view = collect_with(
            &settings(false, dir),
            Some(&secret()),
            &ctx(home.path()),
            &loopback(&server),
        )
        .await;
        assert_eq!(view.windows.len(), 3);
        assert!(view.notices.is_empty() && view.facts.is_empty() && view.meta.is_empty());
    }
    assert!(snapshot(empty.dir()).is_empty());
}
