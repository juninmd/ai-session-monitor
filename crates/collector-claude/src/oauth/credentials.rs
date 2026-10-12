//! Claude Code's stored login, read only to borrow its access token for one request.
//!
//! Layout (community sources; Anthropic does not document the file): `<config dir>/.credentials.json`
//! holds `{"claudeAiOauth": {"accessToken", "refreshToken", "expiresAt" (epoch milliseconds),
//! "scopes", "subscriptionType", "rateLimitTier"}}`. Only the access token, its expiry and the
//! plan fields are read. The refresh token is never looked at, and nothing is ever written back:
//! when the token has expired the user opens Claude Code, which renews it.

use std::io::ErrorKind;
use std::path::Path;

use serde_json::Value;
use usage_model::{ErrorCode, HarnessError};
use usage_support::Secret;

const MAX_FILE_BYTES: usize = 64 * 1024;
const MAX_TOKEN_CHARS: usize = 4096;
const MIN_TOKEN_CHARS: usize = 8;
/// An `expiresAt` below this (about the year 1973 in milliseconds) can only be in seconds.
const SECONDS_BELOW: i64 = 100_000_000_000;
const UTF8_BOM: &[u8] = b"\xEF\xBB\xBF";

pub(crate) struct Credentials {
    pub token: Secret,
    pub expires_at_ms: Option<i64>,
    pub plan: Option<String>,
}

impl Credentials {
    /// An unknown expiry is not treated as expired: the server has the last word.
    pub(crate) fn is_expired(&self, now_secs: i64) -> bool {
        self.expires_at_ms.is_some_and(|expires| expires <= now_secs.saturating_mul(1000))
    }
}

pub(crate) fn load(path: &Path) -> Result<Credentials, HarnessError> {
    let bytes = match usage_support::read_capped(path, MAX_FILE_BYTES) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == ErrorKind::NotFound => return Err(not_signed_in()),
        Err(e) if e.kind() == ErrorKind::FileTooLarge => return Err(changed()),
        Err(_) => return Err(HarnessError::new(ErrorCode::Internal, true)),
    };
    parse(&bytes)
}

fn parse(bytes: &[u8]) -> Result<Credentials, HarnessError> {
    let bytes = bytes.strip_prefix(UTF8_BOM).unwrap_or(bytes);
    let doc: Value = serde_json::from_slice(bytes).map_err(|_| changed())?;
    let login = doc.get("claudeAiOauth").and_then(Value::as_object).ok_or_else(not_signed_in)?;
    let token = login
        .get("accessToken")
        .and_then(Value::as_str)
        .filter(|token| usable_token(token))
        .ok_or_else(not_signed_in)?;
    let text = |key: &str| login.get(key).and_then(Value::as_str);
    Ok(Credentials {
        token: Secret::new(token),
        expires_at_ms: login.get("expiresAt").and_then(expiry_ms),
        plan: plan_label(text("subscriptionType"), text("rateLimitTier")),
    })
}

/// Printable ASCII only: it becomes a header value, and a stray control character must never
/// reach the request builder.
fn usable_token(token: &str) -> bool {
    (MIN_TOKEN_CHARS..=MAX_TOKEN_CHARS).contains(&token.len())
        && token.bytes().all(|b| b.is_ascii_graphic())
}

fn expiry_ms(value: &Value) -> Option<i64> {
    let raw = value.as_i64().or_else(|| {
        let n = value.as_f64()?;
        (n.is_finite() && n.abs() < 1e17).then_some(n as i64)
    })?;
    Some(if raw < SECONDS_BELOW { raw.saturating_mul(1000) } else { raw })
}

/// Display name of the plan; unknown values are not shown rather than guessed.
fn plan_label(subscription: Option<&str>, tier: Option<&str>) -> Option<String> {
    let tier = tier.unwrap_or_default().to_ascii_lowercase();
    let label = match subscription?.trim().to_ascii_lowercase().as_str() {
        "pro" => "Pro",
        "team" => "Team",
        "enterprise" => "Enterprise",
        "free" => "Free",
        "max" if tier.contains("20x") => "Max 20x",
        "max" if tier.contains("5x") => "Max 5x",
        "max" => "Max",
        _ => return None,
    };
    Some(label.to_owned())
}

fn not_signed_in() -> HarnessError {
    HarnessError::new(ErrorCode::NotSignedIn, false)
}

fn changed() -> HarnessError {
    HarnessError::new(ErrorCode::EndpointChanged, false)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn creds(login: &Value) -> Vec<u8> {
        json!({"claudeAiOauth": login}).to_string().into_bytes()
    }

    #[test]
    fn reads_the_token_expiry_and_plan() {
        let c = parse(&creds(&json!({
            "accessToken": "fake-access-token-0001", "refreshToken": "fake-refresh-0001",
            "expiresAt": 1_800_000_900_000_i64, "scopes": ["user:profile"],
            "subscriptionType": "max", "rateLimitTier": "default_claude_max_5x"
        })))
        .unwrap();
        assert_eq!(c.token.expose(), "fake-access-token-0001");
        assert_eq!(c.expires_at_ms, Some(1_800_000_900_000));
        assert_eq!(c.plan.as_deref(), Some("Max 5x"));
    }

    #[test]
    fn expiry_is_milliseconds_and_expired_means_at_or_before_now() {
        let at = |ms: i64| {
            parse(&creds(&json!({"accessToken": "fake-access-token-0001", "expiresAt": ms})))
                .unwrap()
        };
        let now = 1_800_000_000;
        assert!(at(now * 1000 - 1).is_expired(now));
        assert!(at(now * 1000).is_expired(now), "the expiry instant itself is expired");
        assert!(!at(now * 1000 + 1).is_expired(now));
        assert!(at(0).is_expired(now), "an expiry of 0 (a known Claude Code bug) is expired");
    }

    #[test]
    fn an_expiry_written_in_seconds_is_still_understood() {
        let c = parse(&creds(
            &json!({"accessToken": "fake-access-token-0001", "expiresAt": 1_800_000_900}),
        ))
        .unwrap();
        assert_eq!(c.expires_at_ms, Some(1_800_000_900_000));
    }

    #[test]
    fn a_missing_or_malformed_expiry_is_unknown_not_expired() {
        for expiry in [json!(null), json!("soon"), json!([1]), json!({})] {
            let c = parse(&creds(
                &json!({"accessToken": "fake-access-token-0001", "expiresAt": expiry}),
            ))
            .unwrap();
            assert_eq!(c.expires_at_ms, None);
            assert!(!c.is_expired(1_800_000_000));
        }
        let none = parse(&creds(&json!({"accessToken": "fake-access-token-0001"}))).unwrap();
        assert!(!none.is_expired(i64::MAX / 2000));
    }

    #[test]
    fn no_login_means_not_signed_in() {
        for doc in [
            br#"{}"#.to_vec(),
            br#"{"claudeAiOauth":null}"#.to_vec(),
            br#"{"claudeAiOauth":{}}"#.to_vec(),
            creds(&json!({"accessToken": ""})),
            creds(&json!({"accessToken": "short"})),
            creds(&json!({"accessToken": 12345678})),
            creds(&json!({"accessToken": "has space inside it"})),
            creds(&json!({"accessToken": "tab\there-and-more"})),
            creds(&json!({"accessToken": "acentuação-no-token"})),
            creds(&json!({"accessToken": "x".repeat(MAX_TOKEN_CHARS + 1)})),
        ] {
            let error = parse(&doc).err().unwrap_or_else(|| {
                panic!("accepted {}", String::from_utf8_lossy(&doc[..doc.len().min(60)]))
            });
            assert_eq!((error.code, error.retryable), (ErrorCode::NotSignedIn, false));
        }
    }

    #[test]
    fn a_file_that_is_not_json_is_a_changed_format() {
        for doc in [&b"not json"[..], b"", b"[1]x"] {
            assert_eq!(parse(doc).err().map(|e| e.code), Some(ErrorCode::EndpointChanged));
        }
    }

    #[test]
    fn load_maps_io_outcomes() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            load(&dir.path().join("absent")).err().map(|e| e.code),
            Some(ErrorCode::NotSignedIn)
        );
        let big = dir.path().join("big");
        std::fs::write(&big, vec![b' '; MAX_FILE_BYTES + 1]).unwrap();
        assert_eq!(load(&big).err().map(|e| e.code), Some(ErrorCode::EndpointChanged));
        let error = load(dir.path()).err().unwrap();
        assert_eq!((error.code, error.retryable), (ErrorCode::Internal, true));
        let ok = dir.path().join(".credentials.json");
        std::fs::write(&ok, creds(&json!({"accessToken": "fake-access-token-0001"}))).unwrap();
        assert_eq!(load(&ok).unwrap().token.expose(), "fake-access-token-0001");
    }

    #[test]
    fn plans_are_a_whitelist() {
        assert_eq!(plan_label(Some("pro"), None).as_deref(), Some("Pro"));
        assert_eq!(
            plan_label(Some(" MAX "), Some("default_claude_max_20x")).as_deref(),
            Some("Max 20x")
        );
        assert_eq!(plan_label(Some("max"), Some("whatever")).as_deref(), Some("Max"));
        assert_eq!(plan_label(Some("team"), None).as_deref(), Some("Team"));
        assert_eq!(plan_label(Some("<script>"), None), None);
        assert_eq!(plan_label(None, Some("default_claude_max_5x")), None);
    }

    #[test]
    fn the_token_never_shows_in_debug_output() {
        let c = parse(&creds(&json!({"accessToken": "fake-access-token-0001"}))).unwrap();
        assert!(!format!("{:?}", c.token).contains("fake-access"));
    }
}
