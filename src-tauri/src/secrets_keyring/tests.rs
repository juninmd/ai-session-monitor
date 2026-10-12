use keyring_core::api::CredentialStoreApi;
use keyring_core::mock;

use super::*;

const TEST_SERVICE: &str = "ai-session-monitor-unit-test";
const LEAK: &str = "sk-live-fabricated-leak-canary";

/// A store over keyring-core's in-memory mock; the concrete mock stays reachable to inject errors.
fn mock_store() -> (Arc<mock::Store>, KeyringSecretStore) {
    let concrete = mock::Store::new().expect("mock store");
    let store = KeyringSecretStore::with_store(concrete.clone(), TEST_SERVICE, false);
    (concrete, store)
}

fn inject(mock: &mock::Store, kind: SecretKind, error: KeyringError) {
    let entry = mock.build(TEST_SERVICE, kind.as_str(), None).expect("entry");
    let cred: &mock::Cred = entry.as_any().downcast_ref().expect("mock credential");
    cred.set_error(error);
}

fn assert_opaque(err: &HarnessError) {
    assert_eq!(err, &store_error());
    assert!(!format!("{err:?}").contains(LEAK), "{err:?}");
}

#[test]
fn round_trips_a_secret_and_deletes_it_idempotently() {
    let (_, store) = mock_store();
    assert!(store.get(SecretKind::Litellm).expect("get").is_none());
    store.set(SecretKind::Litellm, "fabricated-value").expect("set");
    let got = store.get(SecretKind::Litellm).expect("get").expect("some");
    assert_eq!(got.expose(), "fabricated-value");
    store.delete(SecretKind::Litellm).expect("delete");
    store.delete(SecretKind::Litellm).expect("delete again");
    assert!(store.get(SecretKind::Litellm).expect("get").is_none());
}

#[test]
fn setting_again_replaces_the_value() {
    let (_, store) = mock_store();
    store.set(SecretKind::Opencode, "first").expect("set");
    store.set(SecretKind::Opencode, "second").expect("set");
    assert_eq!(store.get(SecretKind::Opencode).expect("get").expect("some").expose(), "second");
}

#[test]
fn kinds_do_not_share_a_slot() {
    let (_, store) = mock_store();
    store.set(SecretKind::Litellm, "for-litellm").expect("set");
    assert!(store.get(SecretKind::Opencode).expect("get").is_none());
    store.set(SecretKind::Opencode, "for-opencode").expect("set");
    store.delete(SecretKind::Litellm).expect("delete");
    assert_eq!(
        store.get(SecretKind::Opencode).expect("get").expect("some").expose(),
        "for-opencode"
    );
}

#[test]
fn credential_manager_targets_are_namespaced_per_kind() {
    assert_eq!(target_name(SERVICE, SecretKind::Litellm), "ai-session-monitor/litellm");
    assert_eq!(target_name(SERVICE, SecretKind::Opencode), "ai-session-monitor/opencode");
    assert_eq!(target_name(SERVICE, SecretKind::Kimi), "ai-session-monitor/kimi");
    assert_eq!(target_name(SERVICE, SecretKind::Zai), "ai-session-monitor/zai");
    assert_eq!(target_name(SERVICE, SecretKind::Openrouter), "ai-session-monitor/openrouter");
    assert_eq!(target_name(SERVICE, SecretKind::Deepseek), "ai-session-monitor/deepseek");
    assert_eq!(target_name(SERVICE, SecretKind::Deepinfra), "ai-session-monitor/deepinfra");
    assert_eq!(target_name(SERVICE, SecretKind::Moonshot), "ai-session-monitor/moonshot");
    assert_eq!(target_name(SERVICE, SecretKind::Novita), "ai-session-monitor/novita");
    assert_eq!(target_name(SERVICE, SecretKind::Ollama), "ai-session-monitor/ollama");
    assert_eq!(target_name(SERVICE, SecretKind::Minimax), "ai-session-monitor/minimax");
}

#[test]
fn every_kind_has_its_own_slot_in_the_store() {
    let (_, store) = mock_store();
    for kind in SecretKind::ALL {
        store.set(kind, &format!("value-of-{}", kind.as_str())).expect("set");
    }
    for kind in SecretKind::ALL {
        let got = store.get(kind).expect("get").expect("some");
        assert_eq!(got.expose(), format!("value-of-{}", kind.as_str()));
    }
    store.delete(SecretKind::Kimi).expect("delete");
    assert!(store.get(SecretKind::Kimi).expect("get").is_none());
    assert!(store.get(SecretKind::Zai).expect("get").is_some());
}

#[test]
fn a_platform_failure_never_leaks_its_text() {
    let (mock, store) = mock_store();
    let failure = || KeyringError::PlatformFailure(Box::new(std::io::Error::other(LEAK)));
    inject(&mock, SecretKind::Litellm, failure());
    assert_opaque(&store.get(SecretKind::Litellm).expect_err("get"));
    inject(&mock, SecretKind::Litellm, failure());
    assert_opaque(&store.set(SecretKind::Litellm, "x").expect_err("set"));
    inject(&mock, SecretKind::Litellm, failure());
    assert_opaque(&store.delete(SecretKind::Litellm).expect_err("delete"));
}

#[test]
fn undecodable_bytes_are_not_echoed() {
    let (mock, store) = mock_store();
    inject(&mock, SecretKind::Opencode, KeyringError::BadEncoding(LEAK.as_bytes().to_vec()));
    assert_opaque(&store.get(SecretKind::Opencode).expect_err("get"));
}

#[test]
fn a_locked_store_is_an_opaque_failure() {
    let (mock, store) = mock_store();
    let locked = KeyringError::NoStorageAccess(Box::new(std::io::Error::other(LEAK)));
    inject(&mock, SecretKind::Opencode, locked);
    assert_opaque(&store.delete(SecretKind::Opencode).expect_err("delete"));
}

#[test]
fn an_oversized_value_is_reported_as_invalid_input() {
    let (mock, store) = mock_store();
    inject(&mock, SecretKind::Litellm, KeyringError::TooLong("password".into(), 2560));
    let err = store.set(SecretKind::Litellm, "x").expect_err("set");
    assert_eq!(err.code, ErrorCode::InvalidInput);
    assert_eq!(err.params.get("field").map(String::as_str), Some("value"));
}

#[test]
fn a_failed_write_keeps_the_previous_value() {
    let (mock, store) = mock_store();
    store.set(SecretKind::Litellm, "kept").expect("set");
    inject(
        &mock,
        SecretKind::Litellm,
        KeyringError::PlatformFailure(Box::new(std::io::Error::other("x"))),
    );
    store.set(SecretKind::Litellm, "lost").expect_err("set");
    assert_eq!(store.get(SecretKind::Litellm).expect("get").expect("some").expose(), "kept");
}

#[test]
fn native_modifiers_are_sent_so_the_mock_refusing_them_fails_closed() {
    let concrete = mock::Store::new().expect("mock store");
    let store = KeyringSecretStore::with_store(concrete, TEST_SERVICE, true);
    assert_opaque(&store.get(SecretKind::Litellm).expect_err("modifiers refused"));
}

#[test]
fn the_unavailable_store_fails_every_call_without_keeping_anything() {
    let store = UnavailableSecretStore;
    assert_opaque(&store.set(SecretKind::Litellm, LEAK).expect_err("set"));
    assert_opaque(&store.get(SecretKind::Litellm).expect_err("get"));
    assert_opaque(&store.delete(SecretKind::Litellm).expect_err("delete"));
}

/// Touches the real Windows Credential Manager, under a namespace of its own so it can never
/// collide with the app's entries. Run on purpose: `cargo test -p ai-session-monitor -- --ignored`.
#[cfg(windows)]
#[test]
#[ignore = "manual: writes to the real Windows Credential Manager"]
fn manual_round_trip_against_the_credential_manager() {
    let native = windows_native_keyring_store::Store::new().expect("native store");
    let store = KeyringSecretStore::with_store(native, "ai-session-monitor-manual-test", true);
    store.set(SecretKind::Litellm, "fabricated-manual-value").expect("set");
    let got = store.get(SecretKind::Litellm).expect("get").expect("some");
    assert_eq!(got.expose(), "fabricated-manual-value");
    store.delete(SecretKind::Litellm).expect("delete");
    assert!(store.get(SecretKind::Litellm).expect("get").is_none());
}
