//! The private CA bundle named in the settings: read on every poll, size-capped, validated, and
//! never described by its path. Plain loopback `http` is used, so only the file handling is under
//! test here; the handshake itself is in `tls_handshake`.

use std::fs;
use std::path::Path;

use tempfile::TempDir;
use usage_model::ErrorCode;
use usage_model::settings::{LitellmScope, LitellmSettings, TlsMode};
use usage_support::Secret;

use crate::common::tls::issued_by_new_ca;
use crate::common::{Fake, NOW, Reply, assert_ready, error_of, param, run, run_with, settings};

const ONE_MIB: usize = 1 << 20;

async fn proxy() -> Fake {
    Fake::routes([("/key/info", Reply::json(fixture!("key_with_budget.json")))]).await
}

fn with_bundle(base: &str, path: &Path) -> LitellmSettings {
    let pem_path = path.to_string_lossy().into_owned();
    LitellmSettings { tls: TlsMode::CustomCa { pem_path }, ..settings(base) }
}

#[tokio::test]
async fn a_valid_bundle_is_accepted_alone_or_among_comments_and_other_roots() {
    let (first, second) = (issued_by_new_ca(&["127.0.0.1"]), issued_by_new_ca(&["127.0.0.1"]));
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("roots.pem");
    let bundles = [
        first.ca_pem.clone(),
        format!("# corporate roots\n{}\n{}\n", first.ca_pem, second.ca_pem),
        format!("{}\r\n", first.ca_pem.replace('\n', "\r\n")),
    ];
    let fake = proxy().await;
    for bundle in bundles {
        fs::write(&path, bundle).unwrap();
        assert_ready(&run(&with_bundle(&fake.base(), &path)).await);
    }
}

#[tokio::test]
async fn the_bundle_is_read_again_on_every_poll() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("ca.pem");
    let fake = proxy().await;
    let settings = with_bundle(&fake.base(), &path);

    fs::write(&path, "not a certificate").unwrap();
    assert_eq!(param(error_of(&run(&settings).await), "reason"), Some("ca_invalid"));
    assert!(fake.requests().is_empty(), "a broken bundle stops the poll before the network");

    fs::write(&path, issued_by_new_ca(&["127.0.0.1"]).ca_pem).unwrap();
    assert_ready(&run(&settings).await);
    assert_eq!(fake.requests().len(), 1, "a renewed CA is picked up without a restart");

    fs::remove_file(&path).unwrap();
    assert_eq!(param(error_of(&run(&settings).await), "reason"), Some("ca_missing"));
}

#[tokio::test]
async fn nothing_is_read_from_disk_until_every_setting_has_been_validated() {
    let dir = TempDir::new().unwrap();
    let fake = proxy().await;
    let absent = dir.path().join("absent.pem");
    let base = || with_bundle(&fake.base(), &absent);

    // Control: on its own, the absent bundle is what fails.
    assert_eq!(param(error_of(&run(&base()).await), "reason"), Some("ca_missing"));

    let header = LitellmSettings { auth_header: "X Key".to_owned(), ..base() };
    let entity = LitellmSettings { scope: LitellmScope::User, entity_id: None, ..base() };
    for (settings, field) in [(header, "authHeader"), (entity, "entityId")] {
        let view = run(&settings).await;
        let error = error_of(&view);
        assert_eq!(error.code, ErrorCode::InvalidInput, "{field}");
        assert_eq!(param(error, "field"), Some(field));
    }
    let view = run_with(&base(), Some(&Secret::new("sk-é")), NOW).await;
    let error = error_of(&view);
    assert_eq!((error.code, param(error, "field")), (ErrorCode::InvalidInput, Some("value")));
    assert!(fake.requests().is_empty());
}

#[tokio::test]
async fn unusable_bundles_are_a_tls_error_with_only_a_reason() {
    let dir = TempDir::new().unwrap();
    let fake = proxy().await;
    let write = |name: &str, bytes: Vec<u8>| {
        let path = dir.path().join(name);
        fs::write(&path, bytes).unwrap();
        path
    };
    let cases = [
        ("missing.pem", dir.path().join("missing.pem"), "ca_missing"),
        ("a directory", dir.path().to_path_buf(), "ca_unreadable"),
        ("empty.pem", write("empty.pem", Vec::new()), "ca_invalid"),
        ("text.pem", write("text.pem", b"hello".to_vec()), "ca_invalid"),
        ("binary.pem", write("binary.pem", vec![0xff, 0xfe, 0x00, 0x01]), "ca_invalid"),
        (
            "truncated.pem",
            write(
                "truncated.pem",
                b"-----BEGIN CERTIFICATE-----\nMIIB\n-----END CERTIFICATE-----\n".to_vec(),
            ),
            "ca_invalid",
        ),
        (
            "just-under-the-cap.pem",
            write("just-under-the-cap.pem", vec![b'A'; ONE_MIB]),
            "ca_invalid",
        ),
        ("over-the-cap.pem", write("over-the-cap.pem", vec![b'A'; ONE_MIB + 1]), "ca_too_large"),
    ];
    for (name, path, reason) in cases {
        let view = run(&with_bundle(&fake.base(), &path)).await;
        let error = error_of(&view);
        assert_eq!((error.code, error.retryable), (ErrorCode::TlsError, false), "{name}");
        assert_eq!(param(error, "reason"), Some(reason), "{name}");
        assert_eq!(error.params.len(), 1, "{name}: {error:?}");
        let shown = serde_json::to_string(&view).unwrap();
        assert!(
            !shown.contains(name) && !shown.contains(&dir.path().to_string_lossy().into_owned())
        );
    }
    assert!(fake.requests().is_empty(), "no bad bundle ever led to a request");
}

#[tokio::test]
async fn a_bundle_path_must_be_absolute_and_is_trimmed() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("ca.pem");
    fs::write(&path, issued_by_new_ca(&["127.0.0.1"]).ca_pem).unwrap();
    let fake = proxy().await;
    let padded = format!("  {}\t", path.display());
    let settings =
        LitellmSettings { tls: TlsMode::CustomCa { pem_path: padded }, ..settings(&fake.base()) };
    assert_ready(&run(&settings).await);

    for relative in ["ca.pem", "./ca.pem", "..\\ca.pem", "certs/ca.pem", ""] {
        let settings = LitellmSettings {
            tls: TlsMode::CustomCa { pem_path: relative.to_owned() },
            ..settings.clone()
        };
        let view = run(&settings).await;
        let error = error_of(&view);
        assert_eq!(error.code, ErrorCode::InvalidInput, "{relative:?}");
        assert_eq!(param(error, "field"), Some("pemPath"), "{relative:?}");
    }
    assert_eq!(fake.requests().len(), 1, "only the valid bundle led to a request");
}
