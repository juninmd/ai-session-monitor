//! Where the database is looked for: the default location and the validation of `dataDir`.

use std::path::{Path, PathBuf};

use super::{ctx, settings};
use crate::local::{Reason, path};

fn resolve(data_dir: Option<&str>, home: &Path) -> Result<PathBuf, Reason> {
    let mut configured = settings(true, None);
    configured.data_dir = data_dir.map(str::to_owned);
    path::resolve(&configured, &ctx(home))
}

#[test]
fn the_default_is_opencodes_own_location_under_the_home_directory() {
    let home = tempfile::tempdir().unwrap();
    let want = home.path().join(".local").join("share").join("opencode").join("opencode.db");
    assert_eq!(resolve(None, home.path()), Ok(want.clone()));
    for blank in ["", "   ", "\t\n"] {
        assert_eq!(resolve(Some(blank), home.path()), Ok(want.clone()), "{blank:?}");
    }
}

#[test]
fn a_configured_directory_wins_and_is_trimmed() {
    let (home, other) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let dir = other.path().to_str().unwrap();
    let want = other.path().join("opencode.db");
    assert_eq!(resolve(Some(dir), home.path()), Ok(want.clone()));
    assert_eq!(resolve(Some(&format!("  {dir}\t")), home.path()), Ok(want));
}

#[test]
fn relative_and_nul_paths_are_refused() {
    let home = tempfile::tempdir().unwrap();
    for bad in ["relative/dir", "opencode", ".", "..", "~/opencode", r"..\up", "dir\0evil", "\0"] {
        assert_eq!(resolve(Some(bad), home.path()), Err(Reason::InvalidPath), "{bad:?}");
    }
}

#[test]
fn resolving_never_touches_the_disk() {
    let home = tempfile::tempdir().unwrap();
    let missing = home.path().join("not").join("created");
    let got = resolve(missing.to_str(), home.path());
    assert_eq!(got, Ok(missing.join("opencode.db")));
    assert!(!home.path().join("not").exists());
}

#[cfg(windows)]
#[test]
fn network_device_and_drive_relative_paths_are_refused() {
    let home = tempfile::tempdir().unwrap();
    for bad in [
        r"\\server\share\dir",
        "//server/share/dir",
        r"\\?\UNC\server\share\dir",
        r"\\.\pipe\x",
        r"\\.\C:\dir",
        "C:dir",
        r"\dir",
        "/dir",
    ] {
        assert_eq!(resolve(Some(bad), home.path()), Err(Reason::InvalidPath), "{bad}");
    }
    for good in [r"C:\Data\opencode", "C:/Data/opencode", r"\\?\C:\Data\opencode"] {
        assert!(resolve(Some(good), home.path()).is_ok(), "{good}");
    }
}

#[cfg(unix)]
#[test]
fn absolute_paths_are_accepted() {
    let home = tempfile::tempdir().unwrap();
    assert_eq!(
        resolve(Some("/srv/opencode"), home.path()),
        Ok(PathBuf::from("/srv/opencode/opencode.db"))
    );
}
