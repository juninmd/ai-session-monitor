//! Fabricated responses under `tests/fixtures`, shaped after what community tools report (see the
//! crate docs); no value in them comes from a real account.

use serde_json::Value;

/// 2026-10-10T12:00:00Z, the clock every fixture is written against.
pub(crate) const NOW: i64 = 1_791_633_600;

pub(crate) fn text(name: &str) -> &'static str {
    match name {
        "cli_usage" => include_str!("../tests/fixtures/cli_usage.json"),
        "summary_full" => include_str!("../tests/fixtures/summary_full.json"),
        "summary_hub" => include_str!("../tests/fixtures/summary_hub.json"),
        "summary_free" => include_str!("../tests/fixtures/summary_free.json"),
        "summary_prose" => include_str!("../tests/fixtures/summary_prose.json"),
        "status_ide" => include_str!("../tests/fixtures/status_ide.json"),
        "status_hub" => include_str!("../tests/fixtures/status_hub.json"),
        "status_free" => include_str!("../tests/fixtures/status_free.json"),
        other => panic!("no fixture named {other}"),
    }
}

pub(crate) fn json(name: &str) -> Value {
    serde_json::from_str(text(name)).expect("fixtures are valid JSON")
}
