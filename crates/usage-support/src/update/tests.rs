use serde_json::{Value, json};

use super::*;

fn fixture() -> Value {
    serde_json::from_str(include_str!("fixture.json")).unwrap()
}

fn manifest() -> Value {
    json!({"version":"1.2.3", "notes":"Changes", "platforms":{
        "windows-x86_64-nsis": {
            "url":"https://github.com/juninmd/ai-session-monitor/releases/download/v1.2.3/app_1.2.3_x64-setup.exe",
            "signature":fixture()["signature"],
        }
    }})
}

fn candidate(value: &Value) -> Result<Option<UpdateCandidate>, HarnessError> {
    parse_manifest(&serde_json::to_vec(value).unwrap(), "0.1.0")
}

#[test]
fn signed_fixture_is_valid_but_modified_bytes_keys_comments_and_versions_are_rejected() {
    let fixture = fixture();
    let bytes = STANDARD.decode(fixture["bytes"].as_str().unwrap()).unwrap();
    let key = fixture["key"].as_str().unwrap();
    let candidate = candidate(&manifest()).unwrap().unwrap();
    verify_installer(&bytes, &candidate, key).unwrap();
    let mut bad_bytes = bytes.clone();
    bad_bytes[30] ^= 1;
    assert!(verify_installer(&bad_bytes, &candidate, key).is_err());
    assert!(verify_installer(&bytes, &candidate, "invalid secret key").is_err());
    let mut wrong_version = candidate.clone();
    wrong_version.version = "1.2.4".into();
    assert!(verify_installer(&bytes, &wrong_version, key).is_err());
    let text = String::from_utf8(STANDARD.decode(&candidate.signature).unwrap()).unwrap();
    let mut wrong_comment = candidate.clone();
    wrong_comment.signature = STANDARD.encode(text.replace("timestamp:123", "timestamp:456"));
    assert!(verify_installer(&bytes, &wrong_comment, key).is_err());
    let error = verify_installer(&bytes, &candidate, "invalid secret key").unwrap_err();
    assert!(!serde_json::to_string(&error).unwrap().contains("secret"));
}

#[test]
fn equal_and_older_releases_never_become_candidates() {
    let value = serde_json::to_vec(&manifest()).unwrap();
    assert!(parse_manifest(&value, "1.2.3").unwrap().is_none());
    assert!(parse_manifest(&value, "2.0.0").unwrap().is_none());
    assert!(parse_manifest(&value, "0.1.0").unwrap().is_some());
}

#[test]
fn unstable_versions_invalid_metadata_and_foreign_urls_are_rejected() {
    for version in ["1.2.3-beta", "1.2.3+build", "garbage"] {
        let mut value = manifest();
        value["version"] = version.into();
        assert!(candidate(&value).is_err());
    }
    for url in [
        "https://evil.invalid/app.exe?secret=token",
        "https://github.com/juninmd/ai-session-monitor/releases/download/v1.2.4/app_x64-setup.exe",
        "https://github.com/juninmd/ai-session-monitor/releases/download/v1.2.3/app_x64-setup.nsis.zip",
    ] {
        let mut value = manifest();
        value["platforms"]["windows-x86_64-nsis"]["url"] = url.into();
        let error = candidate(&value).err().unwrap();
        assert!(!serde_json::to_string(&error).unwrap().contains("secret"));
    }
    let mut value = manifest();
    value["notes"] = "a".repeat(65537).into();
    assert!(candidate(&value).is_err());
    assert!(parse_manifest(&vec![b' '; (1 << 20) + 1], "0.1.0").is_err());
}
