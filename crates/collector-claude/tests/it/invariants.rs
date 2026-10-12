//! Rules about the crate itself that are cheap to check now and expensive to discover later.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use collector_claude::{NOTICE_CODES, collect};
use usage_model::settings::ClaudeSettings;

use crate::common::{NOW, null_limits};
use crate::setup::{installed, recording, view_of};

#[tokio::test]
async fn every_declared_notice_code_is_emitted_and_every_emitted_code_is_declared() {
    let mut seen = BTreeSet::new();
    let mut note = |view: usage_model::HarnessView| {
        seen.extend(view.notices.into_iter().map(|notice| notice.code));
    };

    note(view_of(&installed()).await);

    let no_limits = installed();
    no_limits.write_rate_limits(&null_limits());
    note(view_of(&no_limits).await);

    let outdated = recording();
    std::fs::remove_file(outdated.shim_copy()).unwrap();
    note(view_of(&outdated).await);

    // No login file: the option fails before any request.
    let no_login = recording();
    let settings = ClaudeSettings { oauth_usage: true, ..no_login.settings() };
    note(collect(&settings, &no_login.ctx_at(NOW)).await);

    let declared: BTreeSet<String> = NOTICE_CODES.iter().map(|code| (*code).to_owned()).collect();
    assert_eq!(declared.len(), NOTICE_CODES.len(), "no duplicates");
    assert_eq!(seen, declared);
}

#[test]
fn the_declared_codes_are_in_the_ipc_catalog() {
    let doc = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/ipc.md"))
        .unwrap();
    for code in NOTICE_CODES {
        assert!(doc.contains(&format!("`{code}`")), "{code} is missing from docs/ipc.md");
    }
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap().map(Result::unwrap) {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

/// Where the config dir comes from is the caller's business (`ClaudeSettings::config_dir`); the
/// crate must not quietly start depending on, or leaking, the process environment.
#[test]
fn the_crate_never_reads_the_process_environment() {
    let mut files = Vec::new();
    rust_files(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src"), &mut files);
    assert!(files.len() > 20, "the scan found the sources");
    let forbidden = [
        "env::var",
        "env::home_dir",
        "env::current_dir",
        "env::args",
        "env::set_var",
        "option_env!",
    ];
    for file in files {
        let text = fs::read_to_string(&file).unwrap();
        for needle in forbidden {
            assert!(!text.contains(needle), "{} uses {needle}", file.display());
        }
    }
}
