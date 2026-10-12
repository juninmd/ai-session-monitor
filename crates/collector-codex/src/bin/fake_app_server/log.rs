//! The side files the tests read back.

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::Path;

pub struct Log {
    file: Option<File>,
}

impl Log {
    pub fn open(dir: &Path) -> Self {
        let file =
            OpenOptions::new().create(true).append(true).open(dir.join("fake-received.log")).ok();
        Self { file }
    }

    /// Every line the client sent, in order; a missing log only costs the test its evidence.
    pub fn record(&mut self, line: &str) {
        if let Some(file) = self.file.as_mut() {
            let _ = writeln!(file, "{line}");
        }
    }
}

/// A side file the tests read back; failing to write it only costs them their evidence.
pub fn write_file(dir: &Path, name: &str, text: &str) {
    let _ = fs::write(dir.join(name), text);
}
