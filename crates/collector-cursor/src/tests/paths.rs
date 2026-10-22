//! Where `state.vscdb` is looked for on each platform.

use std::path::{Path, PathBuf};

use crate::path::{Os, state_db};

#[test]
fn each_platform_has_its_own_default_under_the_home_directory() {
    let home = Path::new("/h/user");
    let tail = Path::new("Cursor").join("User").join("globalStorage").join("state.vscdb");
    assert_eq!(state_db(Os::Windows, home, None), home.join("AppData").join("Roaming").join(&tail));
    assert_eq!(
        state_db(Os::Mac, home, None),
        home.join("Library").join("Application Support").join(&tail)
    );
    assert_eq!(state_db(Os::Linux, home, None), home.join(".config").join(&tail));
}

#[test]
fn xdg_config_home_counts_on_linux_only_when_absolute() {
    let home = Path::new("/h/user");
    let tail = Path::new("Cursor").join("User").join("globalStorage").join("state.vscdb");
    let absolute = PathBuf::from(if cfg!(windows) { "C:\\xdg" } else { "/xdg" });
    assert_eq!(state_db(Os::Linux, home, Some(&absolute)), absolute.join(&tail));
    assert_eq!(
        state_db(Os::Linux, home, Some(Path::new("relative"))),
        home.join(".config").join(&tail)
    );
    assert_eq!(state_db(Os::Mac, home, Some(&absolute)), state_db(Os::Mac, home, None));
}
