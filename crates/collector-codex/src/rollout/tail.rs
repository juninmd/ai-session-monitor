//! Reads the end of one rollout file and pulls out its latest usable `token_count` event.
//!
//! A line is `{"timestamp":..,"type":"event_msg","payload":{"type":"token_count",
//! "rate_limits":{..}}}`. Lines are tested cheaply as bytes first, then parsed; only the
//! timestamp and the `rate_limits` object are read from the one that wins.

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

use serde_json::Value;

use crate::limits::{self, Limits};

/// The main quota bucket. Events for a model-specific bucket carry another id and other windows.
const MAIN_BUCKET: &str = "codex";

pub(super) struct Hit {
    pub limits: Limits,
    /// Epoch seconds from the event's `timestamp`.
    pub timestamp: Option<i64>,
}

pub(super) fn latest_event(path: &Path, cap: u64) -> Option<Hit> {
    let (bytes, starts_mid_file) = read_tail(path, cap).ok()?;
    scan(&bytes, starts_mid_file)
}

/// At most `cap` bytes from the end, and whether the file continues before them.
fn read_tail(path: &Path, cap: u64) -> io::Result<(Vec<u8>, bool)> {
    let mut file = File::open(path)?;
    let start = file.metadata()?.len().saturating_sub(cap);
    file.seek(SeekFrom::Start(start))?;
    let mut bytes = Vec::new();
    // `take` keeps a file that is still growing from outrunning the cap.
    file.take(cap).read_to_end(&mut bytes)?;
    Ok((bytes, start > 0))
}

/// The newest event in `bytes`. A window that starts mid-file begins with the tail of a line
/// that was cut, which is dropped; a last line the writer has not finished simply fails to
/// parse and is skipped.
fn scan(bytes: &[u8], starts_mid_file: bool) -> Option<Hit> {
    let body = if starts_mid_file {
        let first_break = bytes.iter().position(|&byte| byte == b'\n')?;
        &bytes[first_break + 1..]
    } else {
        bytes
    };
    body.rsplit(|&byte| byte == b'\n').find_map(|line| {
        let line = line.strip_suffix(b"\r").unwrap_or(line);
        if contains(line, b"token_count") && contains(line, b"rate_limits") {
            parse(line)
        } else {
            None
        }
    })
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|window| window == needle)
}

fn parse(line: &[u8]) -> Option<Hit> {
    let event: Value = serde_json::from_slice(line).ok()?;
    if event.get("type")?.as_str()? != "event_msg" {
        return None;
    }
    let payload = event.get("payload")?;
    if payload.get("type")?.as_str()? != "token_count" {
        return None;
    }
    let rate_limits = payload.get("rate_limits")?;
    let bucket = rate_limits.get("limit_id").and_then(Value::as_str);
    if bucket.is_some_and(|id| id != MAIN_BUCKET) {
        return None;
    }
    let limits = limits::parse_snapshot(rate_limits).filter(|limits| !limits.windows.is_empty())?;
    let timestamp =
        event.get("timestamp").and_then(Value::as_str).and_then(usage_support::parse_rfc3339_secs);
    Some(Hit { limits, timestamp })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(stamp: &str, used: f64, extra: &str) -> String {
        format!(
            r#"{{"timestamp":"{stamp}","type":"event_msg","payload":{{"type":"token_count","info":null,"rate_limits":{{{extra}"primary":{{"used_percent":{used},"window_minutes":300,"resets_at":1791670500}}}}}}}}"#
        )
    }

    fn lines(parts: &[&str]) -> Vec<u8> {
        parts.join("\n").into_bytes()
    }

    fn used(hit: Option<Hit>) -> Option<f64> {
        hit.and_then(|hit| hit.limits.windows[0].used_pct)
    }

    #[test]
    fn the_last_event_wins_and_is_dated_by_its_timestamp() {
        let older = event("2026-10-10T19:00:00.000Z", 10.0, "");
        let newer = event("2026-10-10T20:00:05.250Z", 20.0, "");
        let hit =
            scan(&lines(&[&older, "{\"type\":\"other\"}", &newer, "{\"type\":\"other\"}"]), false);
        let hit = hit.expect("an event");
        assert_eq!(hit.limits.windows[0].used_pct, Some(20.0));
        assert_eq!(hit.timestamp, Some(1_791_662_405), "fractions are dropped, not rounded");
    }

    #[test]
    fn an_unfinished_last_line_is_skipped_for_the_previous_event() {
        let good = event("2026-10-10T19:00:00.000Z", 10.0, "");
        let cut = &event("2026-10-10T20:00:00.000Z", 99.0, "")[..60];
        assert_eq!(used(scan(&lines(&[&good, cut]), false)), Some(10.0));
        // Cut right after the opening brace of rate_limits is still not valid JSON.
        let just_cut = event("2026-10-10T20:00:00.000Z", 99.0, "").replace("}}}", "");
        assert_eq!(used(scan(&lines(&[&good, &just_cut]), false)), Some(10.0));
    }

    #[test]
    fn crlf_line_ends_are_accepted() {
        let good = event("2026-10-10T19:00:00.000Z", 10.0, "");
        let bytes = format!("{good}\r\n{{\"type\":\"x\"}}\r\n").into_bytes();
        assert_eq!(used(scan(&bytes, false)), Some(10.0));
    }

    #[test]
    fn events_without_rate_limits_or_windows_do_not_count() {
        let null_limits = r#"{"timestamp":"2026-10-10T20:00:00Z","type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"total_tokens":5}},"rate_limits":null}}"#;
        let empty = r#"{"timestamp":"2026-10-10T20:00:00Z","type":"event_msg","payload":{"type":"token_count","rate_limits":{"primary":null,"secondary":null}}}"#;
        let missing = r#"{"timestamp":"2026-10-10T20:00:00Z","type":"event_msg","payload":{"type":"token_count","info":null}}"#;
        assert!(scan(&lines(&[null_limits, empty, missing]), false).is_none());
        let good = event("2026-10-10T19:00:00.000Z", 10.0, "");
        assert_eq!(used(scan(&lines(&[&good, null_limits, empty, missing]), false)), Some(10.0));
    }

    #[test]
    fn only_event_msg_token_count_lines_count_whatever_the_text_says() {
        // A message that merely talks about the fields must not be mistaken for an event.
        let chatter = r#"{"timestamp":"2026-10-10T20:00:00Z","type":"response_item","payload":{"type":"message","content":[{"type":"output_text","text":"look at token_count and rate_limits: {\"primary\":{\"used_percent\":99}}"}]}}"#;
        let wrong_type = r#"{"timestamp":"2026-10-10T20:00:00Z","type":"response_item","payload":{"type":"token_count","rate_limits":{"primary":{"used_percent":99}}}}"#;
        let wrong_payload = r#"{"timestamp":"2026-10-10T20:00:00Z","type":"event_msg","payload":{"type":"agent_message","rate_limits":{"primary":{"used_percent":99}}}}"#;
        assert!(scan(&lines(&[chatter, wrong_type, wrong_payload]), false).is_none());
    }

    #[test]
    fn events_for_another_quota_bucket_are_passed_over() {
        let main = event("2026-10-10T19:00:00.000Z", 10.0, r#""limit_id":"codex","#);
        let other = event("2026-10-10T20:00:00.000Z", 77.0, r#""limit_id":"codex_other","#);
        let unnamed = event("2026-10-10T18:00:00.000Z", 5.0, "");
        assert_eq!(used(scan(&lines(&[&main, &other]), false)), Some(10.0));
        assert_eq!(used(scan(&lines(&[&unnamed]), false)), Some(5.0), "old files name no bucket");
    }

    #[test]
    fn a_window_starting_mid_file_drops_the_cut_first_line_even_inside_a_character() {
        let good = event("2026-10-10T19:00:00.000Z", 10.0, "");
        // The window begins inside the 4-byte emoji of a cut line.
        let mut bytes = "🔑 chave cortada\n".as_bytes()[2..].to_vec();
        bytes.extend_from_slice(good.as_bytes());
        assert_eq!(used(scan(&bytes, true)), Some(10.0));
        // Had the cut line been a lone fragment without a newline, there is nothing to read.
        assert!(scan("ção".as_bytes(), true).is_none());
        // A complete event at the very start of a mid-file window is the cut line: not trusted.
        assert!(scan(good.as_bytes(), true).is_none());
    }

    #[test]
    fn unicode_and_invalid_utf8_lines_are_survivable() {
        let good = event("2026-10-10T19:00:00.000Z", 10.0, "");
        let mut bytes =
            "{\"type\":\"response_item\",\"text\":\"ação 🔑 日本語\"}\n".as_bytes().to_vec();
        bytes.extend_from_slice(good.as_bytes());
        bytes.extend_from_slice(b"\n\xFF\xFE not utf8 token_count rate_limits\n");
        assert_eq!(used(scan(&bytes, false)), Some(10.0));
    }

    #[test]
    fn empty_and_newline_only_input_has_no_event() {
        assert!(scan(b"", false).is_none());
        assert!(scan(b"\n\n\n", false).is_none());
        assert!(scan(b"", true).is_none());
    }

    #[test]
    fn deeply_nested_garbage_is_rejected_not_a_stack_overflow() {
        let nest = format!("{}token_count rate_limits{}", "[".repeat(5000), "]".repeat(5000));
        assert!(scan(nest.as_bytes(), false).is_none());
    }

    #[test]
    fn only_the_tail_of_a_big_file_is_read() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("rollout-x.jsonl");
        let early = event("2026-10-10T19:00:00.000Z", 10.0, "");
        let filler = format!("{{\"type\":\"response_item\",\"pad\":\"{}\"}}\n", "z".repeat(900));
        let late = event("2026-10-10T20:00:00.000Z", 20.0, "");
        let mut body = format!("{early}\n");
        body.push_str(&filler.repeat(600));
        body.push_str(&late);
        std::fs::write(&path, &body).unwrap();
        assert!(body.len() > 500_000);
        assert_eq!(used(latest_event(&path, 64 * 1024)), Some(20.0));

        // With only the early event inside the file's first bytes, a small tail cannot see it.
        let only_early = format!("{early}\n{}", filler.repeat(600));
        std::fs::write(&path, only_early).unwrap();
        assert!(latest_event(&path, 64 * 1024).is_none());
        assert_eq!(
            used(latest_event(&path, 10 << 20)),
            Some(10.0),
            "a cap above the size reads it all"
        );
    }

    #[test]
    fn a_missing_or_unreadable_file_is_nothing() {
        let dir = tempfile::tempdir().unwrap();
        assert!(latest_event(&dir.path().join("absent.jsonl"), 1024).is_none());
        assert!(latest_event(dir.path(), 1024).is_none(), "a directory is not a rollout");
    }
}
