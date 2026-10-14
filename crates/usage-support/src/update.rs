//! Parse public release metadata and verify signed installers. No shell or network access.

use std::collections::BTreeMap;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use minisign_verify::{PublicKey, Signature};
use semver::Version;
use serde::Deserialize;
use usage_model::{ErrorCode, HarnessError};

use crate::http::release_url;

#[derive(Deserialize)]
struct Manifest {
    version: String,
    #[serde(default)]
    notes: Option<String>,
    platforms: BTreeMap<String, Platform>,
}

#[derive(Deserialize)]
struct Platform {
    url: String,
    signature: String,
}

/// Retained only on the Rust side. The webview never gets an installer URL or a signature.
#[derive(Clone)]
pub struct UpdateCandidate {
    pub version: String,
    pub notes: Option<String>,
    pub url: String,
    pub signature: String,
}

fn invalid() -> HarnessError {
    HarnessError::new(ErrorCode::BadResponse, false)
}
fn signature_error() -> HarnessError {
    HarnessError::new(ErrorCode::BadResponse, false).with_param("reason", "signature")
}

pub fn parse_manifest(
    bytes: &[u8],
    current: &str,
) -> Result<Option<UpdateCandidate>, HarnessError> {
    if bytes.len() > 1 << 20 {
        return Err(invalid());
    }
    let manifest: Manifest = serde_json::from_slice(bytes).map_err(|_| invalid())?;
    let current = Version::parse(current).map_err(|_| invalid())?;
    let version = Version::parse(&manifest.version).map_err(|_| invalid())?;
    if !version.pre.is_empty() || !version.build.is_empty() {
        return Err(invalid());
    }
    // Old, equal and rollback releases never become executable candidates.
    if version <= current {
        return Ok(None);
    }
    let platform = manifest
        .platforms
        .get("windows-x86_64-nsis")
        .or_else(|| manifest.platforms.get("windows-x86_64"))
        .ok_or_else(invalid)?;
    let url = release_url(&platform.url)?;
    if url.host_str() == Some("github.com") {
        let prefix = format!("/juninmd/ai-session-monitor/releases/download/v{version}/");
        if !url.path().starts_with(&prefix) || !url.path().ends_with("_x64-setup.exe") {
            return Err(invalid());
        }
    }
    if platform.signature.len() > 8192 || manifest.notes.as_ref().is_some_and(|s| s.len() > 65536) {
        return Err(invalid());
    }
    // A syntactically invalid envelope must fail before downloading a large file.
    let text = STANDARD.decode(&platform.signature).map_err(|_| signature_error())?;
    let text = std::str::from_utf8(&text).map_err(|_| signature_error())?;
    Signature::decode(text).map_err(|_| signature_error())?;
    Ok(Some(UpdateCandidate {
        version: version.to_string(),
        notes: manifest.notes,
        url: platform.url.clone(),
        signature: platform.signature.clone(),
    }))
}

/// Both the bytes and the version in the authenticated comment must match. Legacy signatures
/// are refused, and a network attacker cannot attach a new version to an old signed installer.
pub fn verify_installer(
    bytes: &[u8],
    candidate: &UpdateCandidate,
    encoded_key: &str,
) -> Result<(), HarnessError> {
    let decode = |value: &str| -> Result<String, HarnessError> {
        let bytes = STANDARD.decode(value).map_err(|_| signature_error())?;
        String::from_utf8(bytes).map_err(|_| signature_error())
    };
    let key = PublicKey::decode(&decode(encoded_key)?).map_err(|_| signature_error())?;
    let signature =
        Signature::decode(&decode(&candidate.signature)?).map_err(|_| signature_error())?;
    key.verify(bytes, &signature, false).map_err(|_| signature_error())?;
    if !signature
        .trusted_comment()
        .split('\t')
        .any(|part| part == format!("version:{}", candidate.version))
    {
        return Err(signature_error());
    }
    // This channel carries the NSIS .exe, never ZIP archives or other installer formats.
    if bytes.len() < 64 || !bytes.starts_with(b"MZ") {
        return Err(invalid());
    }
    Ok(())
}

#[cfg(test)]
mod tests;
