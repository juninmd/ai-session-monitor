//! The fallback source: the newest `token_count` event that carries `rate_limits` in Codex's
//! session files (`~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl`). Only that one object is
//! extracted; message content in the same files is never kept. Everything is bounded: how deep
//! and wide the tree walk goes, how long it may take and how much of a file is read.
//!
//! Confidence: low. Nothing documents this format. The layout (`event_msg` > `token_count` >
//! `rate_limits`, snake_case fields) was read from the openai/codex sources and never checked
//! against a real session file, so it can change silently; the view says "last seen" for that.
//!
//! Known limits: `CODEX_HOME` is not honoured (the context carries no environment), and a session
//! whose last event is a single line larger than the tail window (an embedded image, say) hides
//! the `token_count` events before it, so the next-newest file is tried instead.

mod tail;
mod walk;

use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::limits::Limits;

pub(crate) struct Found {
    pub limits: Limits,
    /// The event's own timestamp, else the file's modification time.
    pub read_at: Option<i64>,
}

pub(crate) struct Caps {
    /// Bytes read from the end of a file.
    pub tail_bytes: u64,
    pub max_depth: usize,
    /// Directory entries looked at in one walk.
    pub max_entries: usize,
    pub walk_budget: Duration,
    /// Files tried, newest first, until one holds a usable event.
    pub max_files: usize,
}

impl Caps {
    pub(crate) const DEFAULT: Self = Self {
        tail_bytes: 512 * 1024,
        max_depth: 6,
        max_entries: 20_000,
        walk_budget: Duration::from_millis(1_500),
        max_files: 8,
    };
}

/// Off the async runtime: this is plain blocking file work.
pub(crate) async fn latest(home: &Path) -> Option<Found> {
    let root = sessions_root(home)?;
    tokio::task::spawn_blocking(move || find(&root, &Caps::DEFAULT)).await.ok().flatten()
}

/// Only for an absolute home: an empty or relative one would turn this into a read of whatever
/// directory the app happens to run from.
fn sessions_root(home: &Path) -> Option<PathBuf> {
    home.is_absolute().then(|| home.join(".codex").join("sessions"))
}

fn find(root: &Path, caps: &Caps) -> Option<Found> {
    walk::newest(root, caps).into_iter().find_map(|file| {
        let hit = tail::latest_event(&file.path, caps.tail_bytes)?;
        Some(Found { limits: hit.limits, read_at: hit.timestamp.or(file.mtime) })
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    const EVENT: &str = r#"{"timestamp":"2026-10-10T20:00:05.000Z","type":"event_msg","payload":{"type":"token_count","info":null,"rate_limits":{"primary":{"used_percent":12.0,"window_minutes":300,"resets_at":1791670500}}}}"#;

    fn session(root: &Path, day: &str, name: &str, body: &str) -> std::path::PathBuf {
        let dir = root.join(day);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        fs::write(&path, body).unwrap();
        path
    }

    #[test]
    fn a_newer_file_without_events_does_not_hide_an_older_one_with_data() {
        let home = tempfile::tempdir().unwrap();
        let root = home.path();
        session(root, "2026/10/10", "rollout-2026-10-10T19-00-00-a.jsonl", &format!("{EVENT}\n"));
        session(
            root,
            "2026/10/11",
            "rollout-2026-10-11T08-00-00-b.jsonl",
            "{\"type\":\"session_meta\"}\n",
        );
        let found = find(root, &Caps::DEFAULT).expect("older file has data");
        assert_eq!(found.limits.windows[0].used_pct, Some(12.0));
        assert_eq!(found.read_at, Some(1_791_662_405));
    }

    #[test]
    fn the_file_modification_time_dates_an_event_without_a_timestamp() {
        let home = tempfile::tempdir().unwrap();
        let undated = EVENT.replace(r#""timestamp":"2026-10-10T20:00:05.000Z","#, "");
        let path =
            session(home.path(), "2026/10/10", "rollout-2026-10-10T19-00-00-a.jsonl", &undated);
        let modified = std::time::UNIX_EPOCH + Duration::from_secs(1_791_600_000);
        fs::File::options().write(true).open(&path).unwrap().set_modified(modified).unwrap();
        let found = find(home.path(), &Caps::DEFAULT).expect("event found");
        assert_eq!(found.read_at, Some(1_791_600_000));
    }

    #[test]
    fn only_an_absolute_home_has_a_sessions_directory() {
        let home = tempfile::tempdir().unwrap();
        let expected = home.path().join(".codex").join("sessions");
        assert_eq!(sessions_root(home.path()), Some(expected));
        for relative in ["", ".", "home", "..", "../x", r"\rooted"] {
            assert_eq!(sessions_root(Path::new(relative)), None, "{relative:?}");
        }
    }

    #[test]
    fn nothing_there_is_nothing_found() {
        let home = tempfile::tempdir().unwrap();
        assert!(find(home.path(), &Caps::DEFAULT).is_none(), "no sessions directory");
        session(home.path(), "2026/10/10", "notes.txt", EVENT);
        assert!(find(home.path(), &Caps::DEFAULT).is_none(), "only files that are not rollouts");
    }
}
