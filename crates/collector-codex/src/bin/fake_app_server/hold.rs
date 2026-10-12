//! A process that only waits: the grandchild whose death proves that the tree was killed.

use std::thread::sleep;
use std::time::Duration;

/// Nothing the fake starts may outlive a failed test by long.
pub const LIFETIME: Duration = Duration::from_secs(120);

pub fn run() {
    sleep(LIFETIME);
}
