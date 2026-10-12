//! tauri-build copies each `externalBin` over `target/<profile>/<stem>.exe`, the path Cargo uplifts a
//! workspace bin of the same name to; a clash silently replaces the binary the e2e tests execute.

use std::fs;

#[test]
fn sidecar_stems_never_collide_with_workspace_bin_names() {
    let conf = fs::read_to_string("tauri.conf.json").expect("read tauri.conf.json");
    let json: serde_json::Value = serde_json::from_str(&conf).expect("parse tauri.conf.json");
    let stems: Vec<&str> = json["bundle"]["externalBin"]
        .as_array()
        .expect("bundle.externalBin")
        .iter()
        .map(|bin| bin.as_str().expect("string entry").rsplit('/').next().unwrap_or_default())
        .collect();
    assert!(!stems.is_empty());

    let mut bins = Vec::new();
    for entry in fs::read_dir("../crates").expect("read crates") {
        let manifest = entry.expect("dir entry").path().join("Cargo.toml");
        let Ok(text) = fs::read_to_string(&manifest) else { continue };
        let mut in_bin = false;
        for line in text.lines().map(str::trim) {
            if line.starts_with('[') {
                in_bin = line == "[[bin]]";
            } else if in_bin && line.starts_with("name") {
                bins.extend(line.split('"').nth(1).map(str::to_owned));
            }
        }
    }
    assert!(bins.iter().any(|b| b == "aism-statusline-shim"), "bin scan found nothing: {bins:?}");
    for stem in stems {
        assert!(!bins.iter().any(|b| b == stem), "externalBin {stem:?} clashes with a [[bin]]");
    }
}
