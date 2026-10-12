//! Finds rollout files under the sessions directory without ever walking an unbounded tree. The
//! walk visits the newest-named directories first, so when a cap cuts it short, what it saw is
//! what matters most. Symbolic links and junctions are not followed.

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use super::Caps;

pub(super) struct Candidate {
    pub path: PathBuf,
    /// Modification time, epoch seconds.
    pub mtime: Option<i64>,
}

/// Up to `caps.max_files` rollout files, most recently written first. A resumed session keeps its
/// old name but gets a new modification time, which is why the time decides, not the name.
pub(super) fn newest(root: &Path, caps: &Caps) -> Vec<Candidate> {
    let mut walk = Walk { caps, started: Instant::now(), entries: 0, found: Vec::new() };
    walk.directory(root, 0);
    let mut found = walk.found;
    found.sort_by(|a, b| b.mtime.cmp(&a.mtime).then_with(|| b.path.cmp(&a.path)));
    found.truncate(caps.max_files);
    found
}

struct Walk<'a> {
    caps: &'a Caps,
    started: Instant,
    entries: usize,
    found: Vec<Candidate>,
}

impl Walk<'_> {
    fn spent(&self) -> bool {
        self.entries >= self.caps.max_entries || self.started.elapsed() >= self.caps.walk_budget
    }

    fn directory(&mut self, dir: &Path, depth: usize) {
        if depth > self.caps.max_depth || self.spent() {
            return;
        }
        let Ok(entries) = fs::read_dir(dir) else { return };
        let mut subdirs: Vec<(OsString, PathBuf)> = Vec::new();
        for entry in entries.flatten() {
            if self.spent() {
                break;
            }
            self.entries += 1;
            let Ok(kind) = entry.file_type() else { continue };
            if kind.is_symlink() {
                continue;
            }
            if kind.is_dir() {
                subdirs.push((entry.file_name(), entry.path()));
            } else if kind.is_file() && is_rollout(&entry.file_name()) {
                let mtime = entry
                    .metadata()
                    .ok()
                    .and_then(|meta| meta.modified().ok())
                    .and_then(epoch_secs);
                self.found.push(Candidate { path: entry.path(), mtime });
            }
        }
        subdirs.sort_by(|a, b| b.0.cmp(&a.0));
        for (_, path) in subdirs {
            self.directory(&path, depth + 1);
        }
    }
}

fn is_rollout(name: &std::ffi::OsStr) -> bool {
    name.to_str().is_some_and(|name| name.starts_with("rollout-") && name.ends_with(".jsonl"))
}

fn epoch_secs(time: SystemTime) -> Option<i64> {
    let secs = time.duration_since(UNIX_EPOCH).ok()?.as_secs();
    i64::try_from(secs).ok()
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn touch(path: &Path, secs: u64) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, b"{}\n").unwrap();
        let time = UNIX_EPOCH + Duration::from_secs(secs);
        fs::File::options().write(true).open(path).unwrap().set_modified(time).unwrap();
    }

    fn caps(max_files: usize, max_entries: usize, max_depth: usize) -> Caps {
        Caps { max_files, max_entries, max_depth, ..Caps::DEFAULT }
    }

    fn names(found: &[Candidate]) -> Vec<String> {
        found.iter().map(|c| c.path.file_name().unwrap().to_string_lossy().into_owned()).collect()
    }

    #[test]
    fn only_rollout_jsonl_files_count_and_the_latest_write_comes_first() {
        let root = tempfile::tempdir().unwrap();
        touch(&root.path().join("2026/10/09/rollout-a.jsonl"), 100);
        touch(&root.path().join("2026/10/10/rollout-b.jsonl"), 300);
        touch(&root.path().join("2026/10/10/rollout-c.jsonl.zst"), 900);
        touch(&root.path().join("2026/10/10/other.jsonl"), 900);
        touch(&root.path().join("2026/10/10/rollout-d.json"), 900);
        // Resumed: an old name with a fresh write time beats a newer name.
        touch(&root.path().join("2026/09/01/rollout-resumed.jsonl"), 500);
        let found = newest(root.path(), &caps(8, 1000, 6));
        assert_eq!(names(&found), ["rollout-resumed.jsonl", "rollout-b.jsonl", "rollout-a.jsonl"]);
        assert_eq!(found[0].mtime, Some(500));
    }

    #[test]
    fn the_file_cap_keeps_the_newest_only() {
        let root = tempfile::tempdir().unwrap();
        for n in 0..10u64 {
            touch(&root.path().join(format!("d/rollout-{n}.jsonl")), 1000 + n);
        }
        let found = newest(root.path(), &caps(3, 1000, 6));
        assert_eq!(names(&found), ["rollout-9.jsonl", "rollout-8.jsonl", "rollout-7.jsonl"]);
    }

    #[test]
    fn the_entry_cap_stops_the_walk_and_the_newest_named_directories_go_first() {
        let root = tempfile::tempdir().unwrap();
        for day in 1..=9 {
            touch(&root.path().join(format!("2026/10/0{day}/rollout-{day}.jsonl")), 100 + day);
        }
        // Entries: 1 (2026) + 1 (10) + 9 (days) are directories; the cap bites well before the end.
        let found = newest(root.path(), &caps(8, 14, 6));
        assert!(!found.is_empty() && found.len() < 9, "{}", found.len());
        assert!(names(&found).contains(&"rollout-9.jsonl".to_owned()), "{:?}", names(&found));
    }

    #[test]
    fn depth_is_bounded() {
        let root = tempfile::tempdir().unwrap();
        touch(&root.path().join("a/b/c/rollout-deep.jsonl"), 1);
        touch(&root.path().join("a/rollout-shallow.jsonl"), 2);
        let found = newest(root.path(), &caps(8, 1000, 1));
        assert_eq!(names(&found), ["rollout-shallow.jsonl"]);
    }

    #[test]
    fn a_missing_or_unreadable_root_yields_nothing() {
        let root = tempfile::tempdir().unwrap();
        assert!(newest(&root.path().join("absent"), &caps(8, 1000, 6)).is_empty());
        let file = root.path().join("file");
        fs::write(&file, b"x").unwrap();
        assert!(newest(&file, &caps(8, 1000, 6)).is_empty(), "a file is not a directory");
    }

    #[test]
    fn a_zero_time_budget_ends_the_walk_at_once() {
        let root = tempfile::tempdir().unwrap();
        touch(&root.path().join("d/rollout-a.jsonl"), 1);
        let none = Caps { walk_budget: Duration::ZERO, ..Caps::DEFAULT };
        assert!(newest(root.path(), &none).is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn symbolic_links_are_not_followed() {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        touch(&outside.path().join("rollout-outside.jsonl"), 5);
        std::os::unix::fs::symlink(outside.path(), root.path().join("link")).unwrap();
        assert!(newest(root.path(), &caps(8, 1000, 6)).is_empty());
    }
}
