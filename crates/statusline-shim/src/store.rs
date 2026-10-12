//! The recorded file: `{"v":1,"capturedAt":<epoch s>,"rateLimits":<object or null>}`, replaced
//! atomically so the tray app (a reader in another process) never sees a half-written file.
//!
//! Claude Code runs the command often (every assistant message, window reset, optional timer) and
//! from several sessions at once, so what is written is decided here, not blindly overwritten.

use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::Value;

use crate::atomic::write_atomic;
use crate::capture::Extract;

/// Same bound the app applies when it reads the file.
const MAX_FILE_BYTES: u64 = 64 * 1024;
/// Unchanged content is not rewritten more often than this.
pub const REWRITE_AFTER: Duration = Duration::from_secs(5);
/// Real limits are kept through payloads that carry none (the start of every session) for as
/// long as the longest window lasts; after that "no limits" is the honest answer again.
const KEEP_REAL_FOR_SECS: i64 = 8 * 24 * 3600;

pub struct Existing {
    pub captured_at: i64,
    pub rate_limits: Option<Value>,
    pub age: Duration,
}

#[derive(Debug, PartialEq)]
pub enum Action {
    Skip,
    Write(Option<Value>),
}

pub fn now_secs() -> i64 {
    let elapsed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    i64::try_from(elapsed.as_secs()).unwrap_or(i64::MAX)
}

pub fn render(captured_at: i64, rate_limits: Option<&Value>) -> Vec<u8> {
    let limits = rate_limits.map_or_else(|| "null".to_owned(), Value::to_string);
    format!("{{\"v\":1,\"capturedAt\":{captured_at},\"rateLimits\":{limits}}}\n").into_bytes()
}

/// `None` when the file is missing, oversized or not ours: then it is simply replaced.
pub fn read_existing(path: &Path) -> Option<Existing> {
    let file = File::open(path).ok()?;
    let meta = file.metadata().ok()?;
    if meta.len() > MAX_FILE_BYTES {
        return None;
    }
    let mut bytes = Vec::new();
    file.take(MAX_FILE_BYTES).read_to_end(&mut bytes).ok()?;
    let Value::Object(mut doc) = serde_json::from_slice(&bytes).ok()? else {
        return None;
    };
    if doc.get("v") != Some(&Value::from(1)) {
        return None;
    }
    let captured_at = doc.get("capturedAt")?.as_i64()?;
    // A modification time in the future (clock change) counts as old so the file gets rewritten.
    let age = meta.modified().ok().and_then(|at| SystemTime::now().duration_since(at).ok());
    let rate_limits = doc.remove("rateLimits").filter(Value::is_object);
    Some(Existing { captured_at, rate_limits, age: age.unwrap_or(Duration::MAX) })
}

pub fn decide(extract: &Extract, existing: Option<&Existing>, now: i64) -> Action {
    let fresh = |known: &Existing| known.age < REWRITE_AFTER;
    match extract {
        Extract::Invalid => Action::Skip,
        Extract::Present(limits) => match existing {
            Some(known) if known.rate_limits.as_ref() == Some(limits) && fresh(known) => {
                Action::Skip
            }
            _ => Action::Write(Some(limits.clone())),
        },
        Extract::Absent => match existing {
            Some(known) if known.rate_limits.is_some() => {
                if now.saturating_sub(known.captured_at) <= KEEP_REAL_FOR_SECS {
                    Action::Skip
                } else {
                    Action::Write(None)
                }
            }
            Some(known) if fresh(known) => Action::Skip,
            _ => Action::Write(None),
        },
    }
}

/// Reads the previous file, applies the policy and writes when needed. Never panics, never
/// reports: a failed write only means the app keeps showing the previous data.
pub fn record(path: &Path, extract: &Extract) {
    let existing = read_existing(path);
    let now = now_secs();
    if let Action::Write(limits) = decide(extract, existing.as_ref(), now) {
        let _ = write_atomic(path, &render(now, limits.as_ref()));
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use serde_json::json;

    use super::*;

    const NOW: i64 = 1_800_000_000;

    fn known(limits: Option<Value>, captured_at: i64, age_secs: u64) -> Existing {
        Existing { captured_at, rate_limits: limits, age: Duration::from_secs(age_secs) }
    }

    fn present(value: Value) -> Extract {
        Extract::Present(value)
    }

    #[test]
    fn render_is_one_line_with_the_documented_keys() {
        let text = String::from_utf8(render(NOW, Some(&json!({"five_hour": {"u": 1}})))).unwrap();
        assert_eq!(
            text,
            "{\"v\":1,\"capturedAt\":1800000000,\"rateLimits\":{\"five_hour\":{\"u\":1}}}\n"
        );
        let null = String::from_utf8(render(7, None)).unwrap();
        assert_eq!(null, "{\"v\":1,\"capturedAt\":7,\"rateLimits\":null}\n");
    }

    #[test]
    fn first_write_happens_without_a_previous_file() {
        let limits = json!({"seven_day": {"used_percentage": 4}});
        assert_eq!(decide(&present(limits.clone()), None, NOW), Action::Write(Some(limits)));
        assert_eq!(decide(&Extract::Absent, None, NOW), Action::Write(None));
    }

    #[test]
    fn invalid_payloads_never_write() {
        let old = known(Some(json!({"a": 1})), NOW - 100, 100);
        assert_eq!(decide(&Extract::Invalid, Some(&old), NOW), Action::Skip);
        assert_eq!(decide(&Extract::Invalid, None, NOW), Action::Skip);
    }

    #[test]
    fn unchanged_content_is_skipped_only_while_the_file_is_young() {
        let limits = json!({"five_hour": {"used_percentage": 10}});
        let young = known(Some(limits.clone()), NOW - 1, 1);
        assert_eq!(decide(&present(limits.clone()), Some(&young), NOW), Action::Skip);
        let old = known(Some(limits.clone()), NOW - 6, 6);
        assert_eq!(decide(&present(limits.clone()), Some(&old), NOW), Action::Write(Some(limits)));
    }

    #[test]
    fn changed_content_is_written_even_when_the_file_is_young() {
        let young = known(Some(json!({"five_hour": {"used_percentage": 10}})), NOW - 1, 1);
        let next = json!({"five_hour": {"used_percentage": 11}});
        assert_eq!(decide(&present(next.clone()), Some(&young), NOW), Action::Write(Some(next)));
    }

    #[test]
    fn real_limits_survive_a_payload_without_limits() {
        let real = known(Some(json!({"five_hour": {"used_percentage": 10}})), NOW - 3600, 3600);
        assert_eq!(decide(&Extract::Absent, Some(&real), NOW), Action::Skip);
    }

    #[test]
    fn real_limits_expire_after_the_longest_window() {
        let ancient = known(Some(json!({"a": 1})), NOW - KEEP_REAL_FOR_SECS - 1, 99_999);
        assert_eq!(decide(&Extract::Absent, Some(&ancient), NOW), Action::Write(None));
        let edge = known(Some(json!({"a": 1})), NOW - KEEP_REAL_FOR_SECS, 99_999);
        assert_eq!(decide(&Extract::Absent, Some(&edge), NOW), Action::Skip);
    }

    #[test]
    fn data_captured_in_the_future_is_still_kept() {
        let future = known(Some(json!({"a": 1})), NOW + 7200, 1);
        assert_eq!(decide(&Extract::Absent, Some(&future), NOW), Action::Skip);
    }

    #[test]
    fn a_recorded_null_is_refreshed_but_not_churned() {
        assert_eq!(decide(&Extract::Absent, Some(&known(None, NOW - 1, 1)), NOW), Action::Skip);
        assert_eq!(
            decide(&Extract::Absent, Some(&known(None, NOW - 9, 9)), NOW),
            Action::Write(None)
        );
    }

    #[test]
    fn real_limits_replace_a_recorded_null() {
        let limits = json!({"five_hour": {"used_percentage": 1}});
        let null = known(None, NOW - 1, 1);
        assert_eq!(decide(&present(limits.clone()), Some(&null), NOW), Action::Write(Some(limits)));
    }

    #[test]
    fn read_existing_accepts_only_our_v1_documents() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("rl.json");
        let cases: [(&str, bool); 7] = [
            (r#"{"v":1,"capturedAt":5,"rateLimits":null}"#, true),
            (r#"{"v":1,"capturedAt":5,"rateLimits":{"a":1}}"#, true),
            (r#"{"v":2,"capturedAt":5,"rateLimits":null}"#, false),
            (r#"{"capturedAt":5,"rateLimits":null}"#, false),
            (r#"{"v":1,"rateLimits":null}"#, false),
            ("not json", false),
            ("[]", false),
        ];
        for (text, ok) in cases {
            fs::write(&path, text).unwrap();
            assert_eq!(read_existing(&path).is_some(), ok, "{text}");
        }
        assert!(read_existing(&dir.path().join("missing.json")).is_none());
    }

    #[test]
    fn read_existing_accepts_the_size_bound_itself_and_ignores_anything_larger() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("rl.json");
        let body = r#"{"v":1,"capturedAt":5,"rateLimits":null}"#;
        let room = usize::try_from(MAX_FILE_BYTES).unwrap() - body.len();
        fs::write(&path, format!("{body}{}", " ".repeat(room))).unwrap();
        assert!(read_existing(&path).is_some(), "exactly the bound is fine");
        fs::write(&path, format!("{body}{}", " ".repeat(room + 1))).unwrap();
        assert!(read_existing(&path).is_none(), "one byte more is not");
    }

    #[test]
    fn a_non_object_rate_limits_field_reads_as_none() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("rl.json");
        fs::write(&path, r#"{"v":1,"capturedAt":5,"rateLimits":"oops"}"#).unwrap();
        let existing = read_existing(&path).unwrap();
        assert!(existing.rate_limits.is_none());
    }
}
