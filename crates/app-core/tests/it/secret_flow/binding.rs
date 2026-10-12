//! A stored LiteLLM key belongs to the connection it was entered for. Saving settings that would
//! send the same read somewhere else (server, header, certificate trust) drops the key first, so a
//! typo or a misbehaving webview cannot hand it to another host. Edits that keep the destination
//! keep the key. A speed bump, not a boundary: whoever can type a key can type it for any server.

use std::sync::Arc;

use usage_model::settings::{LitellmScope, TlsMode};
use usage_model::{ErrorCode, HarnessId, SecretKind, Settings};

use crate::doubles::StuckDelete;
use crate::rig::{Rig, RigBuilder, rig, seconds, settle};

/// Fabricated; it only has to be recognisable in a dump.
const KEY: &str = "sk-fake-4f9a0c2e7b1d";
const LITELLM: HarnessId = HarnessId::Litellm;
const HOME: &str = "https://litellm.example.test";
const ELSEWHERE: &str = "https://collector.example.test";

type Edit = fn(&mut Settings);

fn url(settings: &mut Settings, address: &str) {
    settings.litellm.base_url = Some(address.to_owned());
}

fn pem(name: &str) -> String {
    std::env::temp_dir().join(name).to_string_lossy().into_owned()
}

/// A rig whose saved settings already name `address`, as after a first save.
fn at(address: &'static str) -> RigBuilder {
    rig().settings(move |s| url(s, address))
}

/// The order the form uses: the connection is saved first, the key after it.
async fn connected(builder: RigBuilder) -> Rig {
    let rig = builder.start().await;
    rig.at(seconds(1)).await;
    let stored = rig.service.set_secret(SecretKind::Litellm, KEY.to_owned());
    assert!(stored.is_ok_and(|status| status.litellm), "control: the key was stored");
    settle().await;
    assert_eq!(last_key(&rig).as_deref(), Some(KEY), "control: the key is read for its own server");
    rig
}

fn last_key(rig: &Rig) -> Option<String> {
    rig.fake(LITELLM).calls().last().and_then(|call| call.litellm_key.clone())
}

fn key_is_stored(rig: &Rig) -> bool {
    rig.service.secret_status().unwrap_or_else(|e| panic!("secret_status: {e:?}")).litellm
}

/// Reads made under a connection other than `before` ran without the key, and the key is gone.
fn assert_dropped(rig: &Rig, before: &Settings) {
    let calls = rig.fake(LITELLM).calls();
    let later: Vec<_> =
        calls.iter().filter(|call| call.settings.litellm != before.litellm).collect();
    assert!(!later.is_empty(), "control: the changed connection was read");
    let carrying = later.iter().filter(|call| call.litellm_key.is_some()).count();
    assert_eq!(carrying, 0, "reads of the new connection that carried the key");
    assert!(!key_is_stored(rig), "the key is still stored");
}

fn assert_kept(rig: &Rig, what: &str) {
    assert!(key_is_stored(rig), "{what}: the key was dropped");
    assert_eq!(last_key(rig).as_deref(), Some(KEY), "{what}: the next read lost the key");
}

async fn dropped_by(builder: RigBuilder, edit: impl FnOnce(&mut Settings)) {
    let rig = connected(builder).await;
    let before = rig.service.settings();
    rig.edit(edit);
    rig.after(seconds(900)).await;
    assert_dropped(&rig, &before);
}

#[tokio::test(start_paused = true)]
async fn the_key_does_not_follow_the_url_to_another_server() {
    dropped_by(at(HOME), |s| url(s, ELSEWHERE)).await;
}

#[tokio::test(start_paused = true)]
async fn a_new_port_drops_the_key() {
    dropped_by(at(HOME), |s| url(s, "https://litellm.example.test:8443")).await;
}

#[tokio::test(start_paused = true)]
async fn a_new_scheme_drops_the_key() {
    dropped_by(at("https://localhost:4000"), |s| url(s, "http://localhost:4000")).await;
}

#[tokio::test(start_paused = true)]
async fn a_new_path_prefix_drops_the_key() {
    let proxied = at("https://litellm.example.test/proxy");
    dropped_by(proxied, |s| url(s, "https://litellm.example.test/other")).await;
}

#[tokio::test(start_paused = true)]
async fn a_new_auth_header_drops_the_key() {
    dropped_by(at(HOME), |s| s.litellm.auth_header = "X-Api-Key".to_owned()).await;
}

#[tokio::test(start_paused = true)]
async fn trusting_another_certificate_authority_drops_the_key() {
    dropped_by(at(HOME), |s| {
        s.litellm.tls = TlsMode::CustomCa { pem_path: pem("corp-ca.pem") };
    })
    .await;
}

#[tokio::test(start_paused = true)]
async fn a_different_ca_file_drops_the_key() {
    let corp = rig().settings(|s| {
        url(s, HOME);
        s.litellm.tls = TlsMode::CustomCa { pem_path: pem("corp-ca.pem") };
    });
    dropped_by(corp, |s| {
        s.litellm.tls = TlsMode::CustomCa { pem_path: pem("other-ca.pem") };
    })
    .await;
}

#[tokio::test(start_paused = true)]
async fn skipping_the_certificate_check_drops_the_key() {
    dropped_by(at(HOME), |s| {
        s.litellm.tls = TlsMode::Insecure { host: "litellm.example.test".to_owned() };
    })
    .await;
}

#[tokio::test(start_paused = true)]
async fn leaving_the_server_and_coming_back_to_another_cannot_launder_the_key() {
    let rig = connected(at(HOME)).await;
    let before = rig.service.settings();

    rig.edit(|s| s.litellm.base_url = None);
    assert!(!key_is_stored(&rig), "leaving the server already drops the key");
    rig.edit(|s| url(s, ELSEWHERE));
    rig.after(seconds(900)).await;

    assert_dropped(&rig, &before);
}

#[tokio::test(start_paused = true)]
async fn a_key_stored_before_any_server_is_dropped_when_one_is_set() {
    let rig = rig().start().await;
    rig.at(seconds(1)).await;
    rig.service.set_secret(SecretKind::Litellm, KEY.to_owned()).expect("stored");
    let before = rig.service.settings();

    rig.edit(|s| url(s, HOME));
    rig.after(seconds(900)).await;

    assert_dropped(&rig, &before);
}

const RESPELLED: [(&str, &str); 6] = [
    (HOME, "https://LITELLM.Example.Test"),
    (HOME, "https://litellm.example.test:443"),
    (HOME, "https://litellm.example.test/"),
    (HOME, "HTTPS://LiteLLM.Example.Test:443/"),
    (HOME, "https://litellm.example.test."),
    ("https://litellm.example.test/proxy", "https://litellm.example.test/proxy/"),
];

#[tokio::test(start_paused = true)]
async fn respelling_the_same_server_keeps_the_key() {
    for (from, to) in RESPELLED {
        let rig = connected(at(from)).await;
        rig.edit(|s| url(s, to));
        rig.after(seconds(900)).await;
        assert_kept(&rig, to);
    }
}

const KEEPING: [(&str, Edit); 5] = [
    ("poll interval", |s| s.litellm.poll_interval_secs = 600),
    ("timeout", |s| s.litellm.timeout_secs = 20),
    ("scope and id", |s| {
        s.litellm.scope = LitellmScope::User;
        s.litellm.entity_id = Some("u-1".to_owned());
    }),
    ("switching the tab off", |s| s.litellm.enabled = false),
    ("another section", |s| s.codex.consent = true),
];

#[tokio::test(start_paused = true)]
async fn edits_that_do_not_move_the_key_keep_it() {
    for (what, edit) in KEEPING {
        let rig = connected(at(HOME)).await;
        rig.edit(edit);
        rig.after(seconds(900)).await;
        assert_kept(&rig, what);
    }
}

#[tokio::test(start_paused = true)]
async fn a_key_that_cannot_be_dropped_keeps_the_server_from_changing() {
    let store = Arc::new(StuckDelete::default());
    let rig = connected(at(HOME).secret_store(store.clone())).await;
    let (before, on_disk) = (rig.service.settings(), rig.saved_settings());
    let reads = rig.fake(LITELLM).count();

    let mut wanted = before.clone();
    url(&mut wanted, ELSEWHERE);
    let error = rig.service.save_settings(wanted).expect_err("the key cannot be dropped");
    settle().await;

    assert_eq!((error.code, error.retryable), (ErrorCode::Internal, false));
    assert_eq!(error.params.get("where").map(String::as_str), Some("secret_store"));
    assert!(!format!("{error:?}").contains(KEY));
    assert_eq!(store.attempts(), 1, "the drop was tried");
    assert_eq!(rig.service.settings(), before, "memory still holds the old connection");
    assert_eq!(rig.saved_settings(), on_disk, "and so does the file");
    assert!(rig.sink.settings().is_empty(), "nothing was announced");
    assert_eq!(rig.fake(LITELLM).count(), reads, "and nothing was read again");
    assert!(key_is_stored(&rig), "the key is untouched");
}

#[tokio::test(start_paused = true)]
async fn a_save_that_keeps_the_destination_never_touches_the_store() {
    let store = Arc::new(StuckDelete::default());
    let rig = connected(at(HOME).secret_store(store.clone())).await;

    rig.edit(|s| s.litellm.poll_interval_secs = 600);
    rig.edit(|s| s.general.autostart = true);

    assert_eq!(store.attempts(), 0, "a store that cannot delete does not block unrelated saves");
    assert!(key_is_stored(&rig));
}

#[tokio::test(start_paused = true)]
async fn a_rejected_save_leaves_the_key_alone() {
    let rig = connected(at(HOME)).await;
    let mut wanted = rig.service.settings();
    // Plain http to a host that is not loopback.
    url(&mut wanted, "http://collector.example.test");

    let error = rig.service.save_settings(wanted).expect_err("rejected");

    assert_eq!(error.code, ErrorCode::InvalidUrl);
    assert!(key_is_stored(&rig), "a value that changes nothing must not drop the key");
}

#[tokio::test(start_paused = true)]
async fn the_key_typed_after_the_change_belongs_to_the_new_server() {
    let rig = connected(at(HOME)).await;
    rig.edit(|s| url(s, ELSEWHERE));
    assert!(!key_is_stored(&rig));

    rig.service.set_secret(SecretKind::Litellm, KEY.to_owned()).expect("stored");
    settle().await;
    assert_eq!(last_key(&rig).as_deref(), Some(KEY), "the new key is read for the new server");

    rig.edit(|s| s.litellm.poll_interval_secs = 600);
    assert_kept(&rig, "an unrelated edit after re-entering the key");
}
