//! What the installer remembers between runs: the `statusLine` the user had (so an uninstall can
//! restore it exactly) and the command the shim chains to.

use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const MAX_STATE_BYTES: usize = 64 * 1024;
/// Bytes, like the shim's own 32 KiB limit on the command.
const MAX_CHAIN_COMMAND_BYTES: usize = 30_000;
/// The prefix of `chain.json` the shim reads; a longer file is cut off and no longer parses.
const MAX_CHAIN_FILE_BYTES: usize = 64 * 1024;

/// The `statusLine` that was in settings.json before the install.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Saved {
    v: u32,
    had_status_line: bool,
    /// Verbatim, whatever its type. `null` when there was none.
    #[serde(default)]
    status_line: Value,
}

impl Saved {
    pub(crate) fn of(status_line: Option<&Value>) -> Self {
        Self {
            v: 1,
            had_status_line: status_line.is_some(),
            status_line: status_line.cloned().unwrap_or(Value::Null),
        }
    }

    /// The value to put back, `None` when there was no `statusLine` key to begin with.
    pub(crate) fn previous(&self) -> Option<&Value> {
        self.had_status_line.then_some(&self.status_line)
    }
}

/// `None` when the file is absent, oversized, unparseable or from another version.
pub(crate) fn load(path: &Path) -> Option<Saved> {
    let saved: Saved = usage_support::read_json_capped(path, MAX_STATE_BYTES).ok()?;
    (saved.v == 1).then_some(saved)
}

pub(crate) fn save(path: &Path, saved: &Saved) -> io::Result<()> {
    let bytes = serde_json::to_vec_pretty(saved).map_err(io::Error::other)?;
    usage_support::atomic_write(path, &bytes)
}

/// The record from the state file, or from the copy beside settings.json when that file is gone or
/// unreadable (a wiped app data dir). `save_both` writes the state file first, so it is never the
/// staler of the two.
pub(crate) fn load_either(primary: &Path, copy: &Path) -> Option<Saved> {
    load(primary).or_else(|| load(copy))
}

/// Puts the record in both places, state file first. A place that already holds it is left alone.
pub(crate) fn save_both(primary: &Path, copy: &Path, saved: &Saved) -> io::Result<()> {
    for path in [primary, copy] {
        if load(path).as_ref() != Some(saved) {
            save(path, saved)?;
        }
    }
    Ok(())
}

/// The command a shim should chain to: only a `command`-type `statusLine` has one.
pub(crate) fn chain_command(status_line: &Value) -> Option<String> {
    let object = status_line.as_object()?;
    if object.get("type").is_some_and(|kind| kind != "command") {
        return None;
    }
    let command = object.get("command")?.as_str()?.trim();
    (!command.is_empty()).then(|| command.to_owned())
}

/// True when the shim can read `command` back. Checked on the escaped file, because a control
/// character takes six bytes there.
pub(crate) fn fits_the_shim(command: &str) -> bool {
    command.len() <= MAX_CHAIN_COMMAND_BYTES
        && chain_bytes(command).is_ok_and(|bytes| bytes.len() <= MAX_CHAIN_FILE_BYTES)
}

fn chain_bytes(command: &str) -> io::Result<Vec<u8>> {
    serde_json::to_vec(&json!({ "command": command })).map_err(io::Error::other)
}

pub(crate) fn save_chain(path: &Path, command: &str) -> io::Result<()> {
    usage_support::atomic_write(path, &chain_bytes(command)?)
}

/// Removes a file we own; a file that is already gone is the desired outcome.
pub(crate) fn remove(path: &Path) -> io::Result<()> {
    match std::fs::remove_file(path) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_distinguishes_no_key_from_a_null_value() {
        assert_eq!(Saved::of(None).previous(), None);
        assert_eq!(Saved::of(Some(&Value::Null)).previous(), Some(&Value::Null));
        let value = json!({"type": "command", "command": "x", "padding": 2});
        assert_eq!(Saved::of(Some(&value)).previous(), Some(&value));
    }

    #[test]
    fn state_round_trips_through_disk_and_rejects_foreign_content() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("claude").join("shim-state.json");
        let value = json!({"type": "command", "command": "~/ç ✓.sh", "padding": 1});
        save(&path, &Saved::of(Some(&value))).unwrap();
        assert_eq!(load(&path).unwrap().previous(), Some(&value));

        assert!(load(&dir.path().join("absent.json")).is_none());
        for (name, body) in [
            ("v2", r#"{"v":2,"hadStatusLine":true,"statusLine":1}"#),
            ("garbage", "not json"),
            ("array", "[1]"),
        ] {
            let file = dir.path().join(name);
            std::fs::write(&file, body).unwrap();
            assert!(load(&file).is_none(), "{name}");
        }
    }

    #[test]
    fn either_place_serves_and_the_state_file_wins() {
        let dir = tempfile::tempdir().unwrap();
        let (primary, copy) = (dir.path().join("claude").join("state.json"), dir.path().join("c"));
        let (old, new) = (Saved::of(Some(&json!("old"))), Saved::of(Some(&json!("new"))));
        assert_eq!(load_either(&primary, &copy), None);

        save(&copy, &old).unwrap();
        assert_eq!(load_either(&primary, &copy), Some(old.clone()), "the data dir was wiped");
        std::fs::create_dir_all(primary.parent().unwrap()).unwrap();
        std::fs::write(&primary, "not json").unwrap();
        assert_eq!(load_either(&primary, &copy), Some(old), "an unreadable state file");

        save(&primary, &new).unwrap();
        assert_eq!(load_either(&primary, &copy), Some(new), "the state file is never the staler");
    }

    #[test]
    fn save_both_fills_in_what_is_missing_or_different_and_leaves_the_rest_alone() {
        let dir = tempfile::tempdir().unwrap();
        let (primary, copy) = (dir.path().join("claude").join("state.json"), dir.path().join("c"));
        let saved = Saved::of(Some(&json!({"type": "command", "command": "~/ç ✓.sh"})));

        save_both(&primary, &copy, &saved).unwrap();
        assert_eq!((load(&primary), load(&copy)), (Some(saved.clone()), Some(saved.clone())));

        let untouched = std::fs::metadata(&copy).unwrap().modified().unwrap();
        std::fs::write(&primary, "stale").unwrap();
        save_both(&primary, &copy, &saved).unwrap();
        assert_eq!(load(&primary), Some(saved.clone()), "a damaged side is rewritten");
        assert_eq!(
            std::fs::metadata(&copy).unwrap().modified().unwrap(),
            untouched,
            "a side that already holds it is not written again"
        );

        std::fs::remove_file(&copy).unwrap();
        save_both(&primary, &copy, &saved).unwrap();
        assert_eq!(load(&copy), Some(saved));
    }

    #[test]
    fn only_command_status_lines_can_be_chained() {
        assert_eq!(
            chain_command(&json!({"type": "command", "command": "  ~/a.sh  "})).as_deref(),
            Some("~/a.sh")
        );
        assert_eq!(chain_command(&json!({"command": "jq ."})).as_deref(), Some("jq ."));
        for other in [
            json!({"type": "static", "command": "x"}),
            json!({"type": "command"}),
            json!({"type": "command", "command": "   "}),
            json!({"type": "command", "command": 5}),
            json!("a string"),
            json!(null),
            json!([1]),
        ] {
            assert_eq!(chain_command(&other), None, "{other}");
        }
    }

    #[test]
    fn the_chain_file_is_the_shape_the_shim_reads() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("chain.json");
        save_chain(&path, "powershell -File \"C:/a b/s.ps1\"").unwrap();
        let doc: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(doc, json!({"command": "powershell -File \"C:/a b/s.ps1\""}));
    }

    #[test]
    fn the_escaped_chain_file_may_fill_what_the_shim_reads_and_not_a_byte_more() {
        // The JSON frame takes 14 bytes and each control character six, so this is exactly 64 KiB.
        let exact = format!("{}{}", "\u{1}".repeat(10_000), "x".repeat(5_522));
        assert_eq!(chain_bytes(&exact).unwrap().len(), MAX_CHAIN_FILE_BYTES);
        assert!(fits_the_shim(&exact));
        assert!(!fits_the_shim(&format!("{exact}x")));
    }

    #[test]
    fn removing_a_missing_file_is_fine() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("x");
        std::fs::write(&file, "1").unwrap();
        remove(&file).unwrap();
        remove(&file).unwrap();
        assert!(!file.exists());
    }
}
