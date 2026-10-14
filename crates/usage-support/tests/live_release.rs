//! Explicit, read-only acceptance against the public GitHub channel. No profile or installer I/O.

use std::time::Duration;

use usage_support::http::MAX_RELEASE_BYTES;
use usage_support::update::{parse_manifest, verify_installer};
use usage_support::{ClientOptions, SafeClient};

#[tokio::test]
#[ignore = "requires public GitHub networking; never installs or reads a user profile"]
async fn public_channel_downloads_a_valid_signed_installer_through_safe_client() {
    let client = SafeClient::new(ClientOptions {
        timeout: Duration::from_secs(180),
        ..ClientOptions::default()
    })
    .expect("safe client");
    let manifest = client
        .get_release_bytes(
            "https://github.com/juninmd/ai-session-monitor/releases/latest/download/latest.json",
            1 << 20,
            |_, _| {},
        )
        .await
        .expect("public channel manifest");
    let candidate = parse_manifest(&manifest, "0.0.0").expect("valid manifest").expect("release");
    let config: serde_json::Value =
        serde_json::from_str(include_str!("../../../src-tauri/tauri.conf.json")).expect("config");
    let key = config["plugins"]["updater"]["pubkey"].as_str().expect("public key");
    let bytes = client
        .get_release_bytes(&candidate.url, MAX_RELEASE_BYTES, |_, _| {})
        .await
        .expect("bounded installer download");
    verify_installer(&bytes, &candidate, key).expect("both Minisign signatures and signed version");
    let mut corrupt = bytes;
    corrupt[64] ^= 1;
    assert!(verify_installer(&corrupt, &candidate, key).is_err());
}
