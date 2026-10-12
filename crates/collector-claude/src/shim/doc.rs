//! Claude Code's settings.json as an order-preserving JSON object, plus the formatting needed to
//! write it back the way it was found (BOM, indentation, line endings, final newline).
//!
//! A file that is not a JSON object is never loaded, so it can never be rewritten.

use std::io::ErrorKind;
use std::path::Path;

use serde::Serialize;
use serde_json::ser::{PrettyFormatter, Serializer};
use serde_json::{Map, Value};

pub(crate) const MAX_SETTINGS_BYTES: usize = 2 * 1024 * 1024;
const UTF8_BOM: &[u8] = b"\xEF\xBB\xBF";
const STATUS_LINE: &str = "statusLine";

/// The file exists but is not something we may edit.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Unreadable;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Style {
    bom: bool,
    indent: String,
    crlf: bool,
    final_newline: bool,
}

impl Default for Style {
    fn default() -> Self {
        Self { bom: false, indent: "  ".to_owned(), crlf: false, final_newline: true }
    }
}

pub(crate) struct SettingsDoc {
    root: Map<String, Value>,
    style: Style,
    /// Exactly the bytes read; `None` when the file did not exist.
    original: Option<Vec<u8>>,
}

impl SettingsDoc {
    pub(crate) fn load(path: &Path) -> Result<Self, Unreadable> {
        match usage_support::read_capped(path, MAX_SETTINGS_BYTES) {
            Ok(bytes) => Self::parse(bytes),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(Self::missing()),
            Err(_) => Err(Unreadable),
        }
    }

    fn missing() -> Self {
        Self { root: Map::new(), style: Style::default(), original: None }
    }

    fn parse(bytes: Vec<u8>) -> Result<Self, Unreadable> {
        let bom = bytes.starts_with(UTF8_BOM);
        let body = bytes.strip_prefix(UTF8_BOM).unwrap_or(&bytes);
        let text = std::str::from_utf8(body).map_err(|_| Unreadable)?;
        let Value::Object(root) = serde_json::from_str(text).map_err(|_| Unreadable)? else {
            return Err(Unreadable);
        };
        let style = detect_style(text, bom);
        Ok(Self { root, style, original: Some(bytes) })
    }

    /// The bytes as read, for the "did anything change since?" check.
    pub(crate) fn original(&self) -> Option<&[u8]> {
        self.original.as_deref()
    }

    pub(crate) fn status_line(&self) -> Option<&Value> {
        self.root.get(STATUS_LINE)
    }

    /// Replaces the value in place, or appends the key when there is none.
    pub(crate) fn set_status_line(&mut self, value: Value) {
        self.root.insert(STATUS_LINE.to_owned(), value);
    }

    /// Removing must keep the order of the other keys, hence `shift_remove` and not `remove`.
    pub(crate) fn remove_status_line(&mut self) {
        self.root.shift_remove(STATUS_LINE);
    }

    pub(crate) fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        let mut json = Vec::new();
        let formatter = PrettyFormatter::with_indent(self.style.indent.as_bytes());
        self.root.serialize(&mut Serializer::with_formatter(&mut json, formatter))?;
        if self.style.final_newline {
            json.push(b'\n');
        }
        // Raw newlines only occur between tokens: inside strings they are escaped.
        let body = if self.style.crlf { crlf(&json) } else { json };
        let mut out = Vec::with_capacity(body.len() + UTF8_BOM.len());
        if self.style.bom {
            out.extend_from_slice(UTF8_BOM);
        }
        out.extend_from_slice(&body);
        Ok(out)
    }
}

fn crlf(lf: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(lf.len() + lf.len() / 16);
    for &byte in lf {
        if byte == b'\n' {
            out.push(b'\r');
        }
        out.push(byte);
    }
    out
}

fn detect_style(text: &str, bom: bool) -> Style {
    // The first indented line that starts a key shows the indent unit of the top-level object.
    let indent = text.lines().skip(1).find_map(|line| {
        let trimmed = line.trim_start_matches([' ', '\t']);
        let lead = &line[..line.len() - trimmed.len()];
        let uniform = lead.bytes().all(|b| b == b' ') || lead.bytes().all(|b| b == b'\t');
        (!lead.is_empty() && uniform && trimmed.starts_with('"')).then(|| lead.to_owned())
    });
    Style {
        bom,
        indent: indent.unwrap_or_else(|| Style::default().indent),
        crlf: text.contains("\r\n"),
        final_newline: text.ends_with('\n'),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn doc(text: &str) -> SettingsDoc {
        SettingsDoc::parse(text.as_bytes().to_vec()).unwrap()
    }

    fn keys(doc: &SettingsDoc) -> Vec<&str> {
        doc.root.keys().map(String::as_str).collect()
    }

    #[test]
    fn unchanged_documents_serialize_to_the_same_bytes() {
        for text in [
            "{\n  \"a\": 1,\n  \"statusLine\": {\n    \"type\": \"command\"\n  },\n  \"z\": [\n    1,\n    2\n  ]\n}\n",
            "{\n    \"model\": \"opus\",\n    \"ção\": \"ação ✓\"\n}\n",
            "{\n\t\"a\": {},\n\t\"b\": []\n}",
            "{\r\n  \"a\": 1\r\n}\r\n",
            "\u{feff}{\n  \"a\": 1\n}\n",
            "{}\n",
        ] {
            assert_eq!(doc(text).to_bytes().unwrap(), text.as_bytes(), "{text:?}");
        }
    }

    #[test]
    fn key_order_survives_set_and_remove() {
        let mut d = doc("{\"z\":1,\"statusLine\":{\"type\":\"command\"},\"a\":2,\"m\":3}");
        d.set_status_line(json!({"type": "command", "command": "x"}));
        assert_eq!(keys(&d), ["z", "statusLine", "a", "m"], "replaced in place");
        d.remove_status_line();
        assert_eq!(keys(&d), ["z", "a", "m"], "the others keep their order");

        let mut fresh = doc("{\"z\":1,\"a\":2}");
        fresh.set_status_line(json!("v"));
        assert_eq!(keys(&fresh), ["z", "a", "statusLine"], "new keys go last");
    }

    #[test]
    fn style_is_detected_from_the_file() {
        let tabs = detect_style("{\n\t\"a\": 1\n}\n", false);
        assert_eq!(tabs.indent, "\t");
        assert_eq!(detect_style("{\n    \"a\": 1\n}", false).indent, "    ");
        assert_eq!(
            detect_style("{\"a\":1}", false),
            Style { final_newline: false, ..Style::default() }
        );
        assert!(detect_style("{\r\n  \"a\": 1\r\n}\r\n", false).crlf);
        assert!(detect_style("{}", true).bom);
        // Mixed indentation is not trusted.
        assert_eq!(detect_style("{\n \t\"a\": 1\n}", false).indent, "  ");
    }

    #[test]
    fn a_missing_file_is_an_empty_document_and_a_bad_one_is_unreadable() {
        let dir = tempfile::tempdir().unwrap();
        let missing = SettingsDoc::load(&dir.path().join("settings.json")).unwrap();
        assert!(missing.original().is_none() && missing.status_line().is_none());
        assert_eq!(missing.to_bytes().unwrap(), b"{}\n");

        for (name, bytes) in [
            ("trailing_comma", &b"{\"a\":1,}"[..]),
            ("comment", b"{/* c */\"a\":1}"),
            ("array", b"[1]"),
            ("scalar", b"\"x\""),
            ("empty", b""),
            ("not_utf8", b"{\"a\":\"\xff\"}"),
            ("truncated", b"{\"a\":"),
        ] {
            let path = dir.path().join(format!("{name}.json"));
            std::fs::write(&path, bytes).unwrap();
            assert!(SettingsDoc::load(&path).is_err(), "{name}");
        }
        assert!(SettingsDoc::load(dir.path()).is_err(), "a directory is not a settings file");

        let big = dir.path().join("big.json");
        std::fs::write(&big, vec![b' '; MAX_SETTINGS_BYTES + 1]).unwrap();
        assert!(SettingsDoc::load(&big).is_err(), "oversized");
    }

    #[test]
    fn unknown_values_round_trip_including_unicode_and_nesting() {
        let text = "{\n  \"env\": {\n    \"PROJETO\": \"ação ✓ 日本\"\n  },\n  \"hooks\": {\n    \"Stop\": [\n      {\n        \"matcher\": \"\",\n        \"n\": 1.5\n      }\n    ]\n  }\n}\n";
        let d = doc(text);
        assert_eq!(d.to_bytes().unwrap(), text.as_bytes());
        assert_eq!(d.original(), Some(text.as_bytes()));
    }
}
