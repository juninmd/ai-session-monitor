//! A release build has no console, so a child that is not started with `CREATE_NO_WINDOW` flashes
//! a console window and takes focus from whatever the user is typing in. Every production file that
//! starts a process must hide the window, or be listed here with the reason it cannot open one.

use std::fs;
use std::path::{Path, PathBuf};

/// GUI-subsystem targets only: they never allocate a console, so there is nothing to hide.
const GUI_ONLY: [&str; 1] = ["src/links.rs"];
const HIDING: [&str; 3] = ["CREATE_NO_WINDOW", "0x0800_0000", "hide_window("];

fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    // An unreadable directory shrinks the scan; the spawner count in the test catches that.
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten().map(|e| e.path()) {
        let name = entry.file_name().and_then(|n| n.to_str()).unwrap_or_default().to_owned();
        if entry.is_dir() {
            // Test code and helper binaries are never shipped to the user's desktop.
            if !matches!(name.as_str(), "tests" | "bin" | "target") {
                sources(&entry, out);
            }
        } else if name.ends_with(".rs") && name != "tests.rs" {
            out.push(entry);
        }
    }
}

fn spawns_a_process(text: &str) -> bool {
    text.lines().map(str::trim).any(|line| {
        !line.starts_with("//")
            && (line.contains("Command::new(") || line.contains("Command::from("))
    })
}

#[test]
fn every_production_spawn_hides_its_console_window() {
    let mut roots = vec![PathBuf::from("src")];
    for entry in fs::read_dir("../crates").expect("read crates") {
        roots.push(entry.expect("dir entry").path().join("src"));
    }
    let mut files = Vec::new();
    for root in roots.iter().filter(|root| root.is_dir()) {
        sources(root, &mut files);
    }
    let spawners: Vec<_> = files
        .iter()
        .filter(|file| fs::read_to_string(file).is_ok_and(|text| spawns_a_process(&text)))
        .collect();
    assert!(spawners.len() >= 5, "spawn scan found too little: {spawners:?}");

    let mut visible = Vec::new();
    for file in spawners {
        let shown = file.to_string_lossy().replace('\\', "/");
        let allowed = GUI_ONLY.iter().any(|gui| shown == *gui);
        let text = fs::read_to_string(file).expect("read source");
        if !allowed && !HIDING.iter().any(|marker| text.contains(marker)) {
            visible.push(shown);
        }
    }
    assert!(
        visible.is_empty(),
        "these files start a process without hiding its window: {visible:?}"
    );
}
