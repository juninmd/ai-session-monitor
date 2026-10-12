//! Reads Claude Code's statusLine JSON from stdin and keeps ONLY its `rate_limits` object. Every
//! other field (cwd, transcript path, model, cost, session id, ...) is dropped on the floor here,
//! before anything touches the disk.

use std::io::Read;

use serde_json::Value;

/// A statusLine payload is a few KB; more than this is not one.
pub const MAX_STDIN_BYTES: usize = 1 << 20;
/// Keeps the recorded file far below the 64 KiB the app accepts when reading it.
const MAX_RATE_LIMITS_BYTES: usize = 32 * 1024;
const UTF8_BOM: &[u8] = b"\xEF\xBB\xBF";

/// What was read from stdin: at most `MAX_STDIN_BYTES + 1` bytes. The extra byte is the overflow
/// marker; the unread remainder of stdin is still forwarded to the chained command untouched.
pub struct Payload {
    pub head: Vec<u8>,
    pub overflow: bool,
}

pub fn read_payload(input: &mut impl Read) -> Payload {
    let mut head = Vec::new();
    let limit = u64::try_from(MAX_STDIN_BYTES).unwrap_or(u64::MAX).saturating_add(1);
    // A read error keeps whatever arrived before it: the chained command still gets that much.
    let _ = input.take(limit).read_to_end(&mut head);
    let overflow = head.len() > MAX_STDIN_BYTES;
    Payload { head, overflow }
}

#[derive(Debug, PartialEq)]
pub enum Extract {
    /// Not a statusLine payload (bad JSON, not an object, oversized): record nothing at all.
    Invalid,
    /// A valid payload without usable limits, which is normal before the first API response.
    Absent,
    Present(Value),
}

pub fn extract_rate_limits(payload: &Payload) -> Extract {
    if payload.overflow {
        return Extract::Invalid;
    }
    let bytes = payload.head.strip_prefix(UTF8_BOM).unwrap_or(&payload.head);
    let Ok(Value::Object(mut root)) = serde_json::from_slice::<Value>(bytes) else {
        return Extract::Invalid;
    };
    match root.remove("rate_limits") {
        Some(limits @ Value::Object(_)) if !is_empty_object(&limits) => {
            let size = serde_json::to_vec(&limits).map_or(usize::MAX, |bytes| bytes.len());
            if size > MAX_RATE_LIMITS_BYTES { Extract::Invalid } else { Extract::Present(limits) }
        }
        _ => Extract::Absent,
    }
}

fn is_empty_object(value: &Value) -> bool {
    value.as_object().is_some_and(serde_json::Map::is_empty)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payload(text: &str) -> Payload {
        read_payload(&mut text.as_bytes())
    }

    #[test]
    fn extracts_only_the_rate_limits_object() {
        let json = r#"{"cwd":"/secret/dir","model":{"id":"m"},"session_id":"s-1",
            "rate_limits":{"five_hour":{"used_percentage":12.5,"resets_at":1800000000}}}"#;
        let Extract::Present(limits) = extract_rate_limits(&payload(json)) else {
            panic!("expected rate limits");
        };
        assert_eq!(limits["five_hour"]["used_percentage"], 12.5);
        assert!(!limits.to_string().contains("secret"));
    }

    #[test]
    fn missing_null_empty_and_non_object_limits_are_absent() {
        for json in [
            r#"{"cwd":"x"}"#,
            r#"{"rate_limits":null}"#,
            r#"{"rate_limits":{}}"#,
            r#"{"rate_limits":[1,2]}"#,
            r#"{"rate_limits":"none"}"#,
            r#"{"rate_limits":7}"#,
        ] {
            assert_eq!(extract_rate_limits(&payload(json)), Extract::Absent, "{json}");
        }
    }

    #[test]
    fn garbage_and_non_object_documents_are_invalid() {
        for text in ["", "   ", "not json", "{\"rate_limits\":", "[]", "null", "42", "\"x\""] {
            assert_eq!(extract_rate_limits(&payload(text)), Extract::Invalid, "{text:?}");
        }
    }

    #[test]
    fn a_utf8_bom_is_tolerated() {
        let mut bytes = UTF8_BOM.to_vec();
        bytes.extend_from_slice(br#"{"rate_limits":{"seven_day":{"used_percentage":3}}}"#);
        let payload = read_payload(&mut bytes.as_slice());
        assert!(matches!(extract_rate_limits(&payload), Extract::Present(_)));
    }

    #[test]
    fn oversized_input_is_flagged_and_invalid() {
        let big = vec![b' '; MAX_STDIN_BYTES + 10];
        let payload = read_payload(&mut big.as_slice());
        assert!(payload.overflow);
        assert_eq!(payload.head.len(), MAX_STDIN_BYTES + 1, "only one byte past the cap is read");
        assert_eq!(extract_rate_limits(&payload), Extract::Invalid);
    }

    #[test]
    fn exactly_the_cap_is_not_an_overflow() {
        let mut body = br#"{"rate_limits":{"a":{"used_percentage":1}}}"#.to_vec();
        body.resize(MAX_STDIN_BYTES, b' ');
        let payload = read_payload(&mut body.as_slice());
        assert!(!payload.overflow);
        assert!(matches!(extract_rate_limits(&payload), Extract::Present(_)));
    }

    #[test]
    fn an_implausibly_large_rate_limits_object_is_refused() {
        let filler = "x".repeat(MAX_RATE_LIMITS_BYTES);
        let json = format!(r#"{{"rate_limits":{{"note":"{filler}"}}}}"#);
        assert_eq!(extract_rate_limits(&payload(&json)), Extract::Invalid);
    }

    #[test]
    fn the_rate_limits_size_bound_is_inclusive() {
        // `{"n":""}` is 8 bytes once serialized, so the filler sets the size exactly.
        let sized = |bytes: usize| {
            payload(&format!(r#"{{"rate_limits":{{"n":"{}"}}}}"#, "x".repeat(bytes - 8)))
        };
        assert!(matches!(extract_rate_limits(&sized(MAX_RATE_LIMITS_BYTES)), Extract::Present(_)));
        assert_eq!(extract_rate_limits(&sized(MAX_RATE_LIMITS_BYTES + 1)), Extract::Invalid);
    }

    #[test]
    fn unicode_values_survive() {
        let json = r#"{"rate_limits":{"spend_limit":{"period":"mensal ção 🗓️"}}}"#;
        let Extract::Present(limits) = extract_rate_limits(&payload(json)) else {
            panic!("expected rate limits");
        };
        assert_eq!(limits["spend_limit"]["period"], "mensal ção 🗓️");
    }
}
