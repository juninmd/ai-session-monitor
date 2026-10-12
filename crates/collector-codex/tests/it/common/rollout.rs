//! Fabricated rollout files in the layout Codex uses: `.codex/sessions/YYYY/MM/DD/rollout-*.jsonl`.
//! The shapes follow the openai/codex sources; every value is invented.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, UNIX_EPOCH};

use serde_json::{Value, json};

/// A `rate_limits` object (the rollout spelling is snake_case). Each window is `(used %, reset)`.
pub fn rate_limits(five_hour: Option<(f64, i64)>, weekly: Option<(f64, i64)>) -> Value {
    let window = |minutes: i64, (used, resets_at): (f64, i64)| json!({"used_percent": used, "window_minutes": minutes, "resets_at": resets_at});
    json!({
        "limit_id": "codex",
        "primary": five_hour.map(|w| window(300, w)),
        "secondary": weekly.map(|w| window(10_080, w)),
        "credits": {"has_credits": false, "unlimited": false, "balance": null},
        "plan_type": "plus"
    })
}

/// One `token_count` line around `rate_limits`.
pub fn event_with(stamp: &str, rate_limits: Value) -> String {
    json!({
        "timestamp": stamp,
        "type": "event_msg",
        "payload": {"type": "token_count", "info": null, "rate_limits": rate_limits}
    })
    .to_string()
}

pub fn event(stamp: &str, five_hour: Option<(f64, i64)>, weekly: Option<(f64, i64)>) -> String {
    event_with(stamp, rate_limits(five_hour, weekly))
}

/// A line of conversation that has nothing to do with limits.
pub fn chatter(text: &str) -> String {
    json!({
        "timestamp": "2026-10-10T19:00:00.000Z",
        "type": "response_item",
        "payload": {"type": "message", "role": "user", "content": [{"type": "input_text", "text": text}]}
    })
    .to_string()
}

pub fn session_path(home: &Path, day: &str, name: &str) -> PathBuf {
    home.join(".codex").join("sessions").join(day).join(name)
}

/// Writes `lines` as a rollout file under `day` (for example `2026/10/10`).
pub fn write_session(home: &Path, day: &str, name: &str, lines: &[String]) -> PathBuf {
    let path = session_path(home, day, name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, lines.join("\n") + "\n").unwrap();
    path
}

/// A resumed session keeps its old name but is written to again.
pub fn set_modified(path: &Path, epoch_secs: u64) {
    let file = fs::File::options().write(true).open(path).unwrap();
    file.set_modified(UNIX_EPOCH + Duration::from_secs(epoch_secs)).unwrap();
}
