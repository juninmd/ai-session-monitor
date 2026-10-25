//! `fsutil`: atomic replacement under concurrent readers, and reads that are bounded.

use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use serde::Deserialize;
use usage_support::{atomic_write, neutral_dir, read_capped, read_json_capped};

#[test]
fn the_cli_cwd_lives_under_the_app_data_dir_and_is_reused() {
    // A shared temp dir may hold a `.claude` folder from another tool; the app's own dir never does.
    let data = tempfile::tempdir().unwrap();
    let dir = neutral_dir(data.path()).unwrap();
    assert_eq!(dir, data.path().join("cli-cwd"));
    assert!(dir.is_dir());
    assert_eq!(neutral_dir(data.path()).unwrap(), dir);
    assert_eq!(fs::read_dir(&dir).unwrap().count(), 0);
}

fn entries(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[test]
fn replaces_existing_content_and_leaves_no_temp_files() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    atomic_write(&path, b"first version, longer than the second").unwrap();
    atomic_write(&path, b"second").unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"second");
    assert_eq!(entries(dir.path()), ["settings.json"]);
}

#[test]
fn creates_missing_parent_directories() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("a").join("b").join("c").join("state.json");
    atomic_write(&path, b"{}").unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"{}");
    assert_eq!(entries(path.parent().unwrap()), ["state.json"]);
}

#[test]
fn writes_empty_and_unicode_content_to_a_unicode_path() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("configuração-é-密码.json");
    atomic_write(&path, "{\"nome\":\"João 🔑\"}".as_bytes()).unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), "{\"nome\":\"João 🔑\"}");
    atomic_write(&path, b"").unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"");
    assert_eq!(entries(dir.path()), ["configuração-é-密码.json"]);
}

#[test]
fn a_failed_replace_removes_the_temp_file_and_touches_nothing_else() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("target");
    fs::create_dir(&target).unwrap();
    fs::write(target.join("inside.txt"), b"keep").unwrap();

    let err = atomic_write(&target, b"data").unwrap_err();
    assert_ne!(err.kind(), ErrorKind::NotFound, "{err}");
    assert_eq!(entries(dir.path()), ["target"], "no temp file may stay behind");
    assert_eq!(fs::read(target.join("inside.txt")).unwrap(), b"keep");
}

#[test]
fn a_parent_that_is_a_file_fails_without_side_effects() {
    let dir = tempfile::tempdir().unwrap();
    let blocker = dir.path().join("blocker");
    fs::write(&blocker, b"i am a file").unwrap();
    assert!(atomic_write(&blocker.join("child.json"), b"{}").is_err());
    assert_eq!(entries(dir.path()), ["blocker"]);
    assert_eq!(fs::read(&blocker).unwrap(), b"i am a file");
}

#[test]
fn a_path_without_a_file_name_is_rejected() {
    for path in [Path::new(""), Path::new(".."), Path::new("/")] {
        let err = atomic_write(path, b"x").unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidInput, "{path:?}");
    }
}

fn payload(version: u32) -> Vec<u8> {
    let pad = match version % 4 {
        0 => 0,
        1 => 100,
        2 => 8 * 1024,
        _ => 100 * 1024,
    };
    format!("{{\"version\":{version},\"pad\":\"{}\"}}", "x".repeat(pad)).into_bytes()
}

fn is_a_complete_version(bytes: &[u8]) -> bool {
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(bytes) else { return false };
    let Some(version) = value["version"].as_u64().and_then(|v| u32::try_from(v).ok()) else {
        return false;
    };
    payload(version) == bytes
}

#[test]
fn concurrent_readers_never_observe_a_partial_or_empty_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = Arc::new(dir.path().join("live.json"));
    atomic_write(&path, &payload(0)).unwrap();
    let done = Arc::new(AtomicBool::new(false));

    let readers: Vec<_> = (0..3)
        .map(|_| {
            let (path, done) = (Arc::clone(&path), Arc::clone(&done));
            thread::spawn(move || {
                let (mut complete, mut busy) = (0u32, 0u32);
                while !done.load(Ordering::Acquire) {
                    match fs::read(path.as_path()) {
                        Ok(bytes) => {
                            assert!(
                                is_a_complete_version(&bytes),
                                "torn read of {} bytes",
                                bytes.len()
                            );
                            complete += 1;
                        }
                        // Windows may refuse an open for an instant while the name is swapped;
                        // the file disappearing would break atomicity and is not tolerated.
                        Err(e) if e.kind() == ErrorKind::PermissionDenied => busy += 1,
                        Err(e) => panic!("reader failed: {e:?}"),
                    }
                }
                (complete, busy)
            })
        })
        .collect();

    let deadline = Instant::now() + Duration::from_secs(60);
    for version in 1..=120 {
        atomic_write(&path, &payload(version)).unwrap();
        assert!(Instant::now() < deadline, "writer too slow");
    }
    done.store(true, Ordering::Release);

    let mut complete = 0;
    for reader in readers {
        complete += reader.join().unwrap().0;
    }
    assert!(complete >= 30, "readers only completed {complete} reads");
    assert!(is_a_complete_version(&fs::read(path.as_path()).unwrap()));
    assert_eq!(entries(dir.path()), ["live.json"]);
}

#[cfg(windows)]
mod windows_sharing {
    use std::os::windows::fs::OpenOptionsExt;

    use super::*;

    // FILE_SHARE_READ only: no FILE_SHARE_DELETE, so the file cannot be replaced while open.
    const FILE_SHARE_READ: u32 = 1;

    #[test]
    fn rename_waits_for_a_reader_that_blocks_replacement() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.json");
        atomic_write(&path, b"old").unwrap();
        let held =
            fs::OpenOptions::new().read(true).share_mode(FILE_SHARE_READ).open(&path).unwrap();
        let releaser = thread::spawn(move || {
            thread::sleep(Duration::from_millis(200));
            drop(held);
        });

        let started = Instant::now();
        atomic_write(&path, b"new").unwrap();
        releaser.join().unwrap();

        assert!(started.elapsed() >= Duration::from_millis(100), "the retry path did not run");
        assert_eq!(fs::read(&path).unwrap(), b"new");
        assert_eq!(entries(dir.path()), ["state.json"]);
    }

    #[test]
    fn rename_gives_up_after_bounded_retries_and_keeps_the_old_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.json");
        atomic_write(&path, b"old").unwrap();
        let held =
            fs::OpenOptions::new().read(true).share_mode(FILE_SHARE_READ).open(&path).unwrap();

        let started = Instant::now();
        let err = atomic_write(&path, b"new").unwrap_err();
        assert!(started.elapsed() < Duration::from_secs(10), "retries must be bounded: {err}");
        assert!(
            err.kind() == ErrorKind::PermissionDenied || err.raw_os_error() == Some(32),
            "{err:?}"
        );
        drop(held);

        assert_eq!(fs::read(&path).unwrap(), b"old");
        assert_eq!(entries(dir.path()), ["state.json"], "temp file must be removed");
        atomic_write(&path, b"new").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"new");
    }
}

#[test]
fn read_capped_allows_exactly_the_limit_and_rejects_one_byte_more() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.bin");
    fs::write(&path, vec![7u8; 1000]).unwrap();
    assert_eq!(read_capped(&path, 1000).unwrap().len(), 1000);
    let err = read_capped(&path, 999).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::FileTooLarge);
    assert!(!err.to_string().contains("data.bin"), "paths stay out of messages: {err}");
}

#[test]
fn read_capped_handles_empty_files_zero_caps_and_a_huge_cap() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("empty");
    fs::write(&path, b"").unwrap();
    assert_eq!(read_capped(&path, 0).unwrap(), b"");
    assert_eq!(read_capped(&path, usize::MAX).unwrap(), b"");
    fs::write(&path, b"x").unwrap();
    assert_eq!(read_capped(&path, 0).unwrap_err().kind(), ErrorKind::FileTooLarge);
    assert_eq!(read_capped(&path, usize::MAX).unwrap(), b"x");
}

#[test]
fn read_capped_refuses_a_huge_file_without_reading_it() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("huge.bin");
    fs::File::create(&path).unwrap().set_len(512 * 1024 * 1024).unwrap();
    let started = Instant::now();
    assert_eq!(read_capped(&path, 4096).unwrap_err().kind(), ErrorKind::FileTooLarge);
    assert!(started.elapsed() < Duration::from_secs(2), "must fail on the size, not by reading");
}

#[test]
fn read_capped_reports_missing_files_and_directories() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(read_capped(&dir.path().join("nope"), 10).unwrap_err().kind(), ErrorKind::NotFound);
    assert!(read_capped(dir.path(), 10).is_err());
}

#[derive(Debug, Deserialize, PartialEq)]
struct Config {
    name: String,
    limit: u32,
}

#[test]
fn read_json_capped_decodes_unicode_with_or_without_a_byte_order_mark() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    let want = Config { name: "João 🔑".into(), limit: 7 };
    fs::write(&path, "{\"name\":\"João 🔑\",\"limit\":7}").unwrap();
    assert_eq!(read_json_capped::<Config>(&path, 1024).unwrap(), want);
    fs::write(&path, "\u{feff}{\"name\":\"João 🔑\",\"limit\":7}").unwrap();
    assert_eq!(read_json_capped::<Config>(&path, 1024).unwrap(), want);
}

#[test]
fn read_json_capped_errors_describe_the_position_but_never_echo_the_content() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    for (content, kind) in [
        ("{\"name\":\"x\",\"limit\":\"sk-live-wrong-type-secret\"}", ErrorKind::InvalidData),
        ("{\"name\":\"sk-live-truncated-secret", ErrorKind::InvalidData),
        ("{\"name\":\"x\",\"limit\":1} trailing sk-live-garbage", ErrorKind::InvalidData),
        ("", ErrorKind::InvalidData),
        ("[]", ErrorKind::InvalidData),
        ("null", ErrorKind::InvalidData),
    ] {
        fs::write(&path, content).unwrap();
        let err = read_json_capped::<Config>(&path, 1024).unwrap_err();
        assert_eq!(err.kind(), kind, "{content:?}");
        let text = format!("{err} {err:?}");
        assert!(!text.contains("sk-live"), "content leaked into the error: {text}");
        assert!(text.contains("line"), "{text}");
    }
}

#[test]
fn read_json_capped_checks_the_size_before_parsing() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("big.json");
    fs::write(&path, format!("{{\"name\":\"{}\",\"limit\":1}}", "a".repeat(2000))).unwrap();
    assert_eq!(read_json_capped::<Config>(&path, 100).unwrap_err().kind(), ErrorKind::FileTooLarge);
    assert!(read_json_capped::<Config>(&path, 4096).is_ok());
}
