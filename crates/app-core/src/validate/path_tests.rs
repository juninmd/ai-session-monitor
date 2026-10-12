//! Custom paths name files the app opens, or runs, on every poll. Only a plain local path may: a
//! share would open a network connection (and send credentials) to whatever host the path names,
//! a device path or a reserved name is not a file at all.

use std::path::Path;

use usage_model::settings::TlsMode;

use super::tests::{abs, reject};
use super::*;

type Set = fn(&mut Settings, Option<String>);

/// Every field that holds a custom path, with how to set it.
const FIELDS: [(&str, Set); 4] = [
    ("configDir", |s, v| s.claude.config_dir = v),
    ("binaryPath", |s, v| s.codex.binary_path = v),
    ("dataDir", |s, v| s.opencode.data_dir = v),
    ("pemPath", |s, v| {
        s.litellm.tls = v.map_or(TlsMode::System, |pem_path| TlsMode::CustomCa { pem_path });
    }),
];

fn expect_ok(field: &str, set: Set, path: &str) {
    let mut settings = Settings::default();
    set(&mut settings, Some(path.to_owned()));
    assert!(validate(&settings).is_ok(), "{field} = {path:?} should be accepted");
}

fn expect_rejected(field: &str, set: Set, path: &str) {
    let mut settings = Settings::default();
    set(&mut settings, Some(path.to_owned()));
    reject(&settings, ErrorCode::InvalidInput, field);
}

fn under(base: &str, parts: &[&str]) -> String {
    let joined = parts.iter().fold(Path::new(base).to_path_buf(), |path, part| path.join(part));
    joined.to_string_lossy().into_owned()
}

#[test]
fn a_plain_absolute_path_is_accepted_and_unset_is_valid() {
    for (field, set) in FIELDS {
        for good in [abs("a b"), abs("x"), under(&abs("x"), &["nested", "ca.pem"])] {
            expect_ok(field, set, &good);
        }
        let mut settings = Settings::default();
        set(&mut settings, None);
        assert!(validate(&settings).is_ok(), "{field} unset");
    }
}

#[test]
fn relative_nul_empty_and_blank_paths_are_rejected() {
    let nul = format!("{}\0x", abs("x"));
    for (field, set) in FIELDS {
        for bad in ["codex", "..", "./ca.pem", r"certs\ca.pem", r"\rooted-without-drive", "", " "] {
            expect_rejected(field, set, bad);
        }
        expect_rejected(field, set, &nul);
    }
}

#[test]
fn the_length_limit_is_1024_characters() {
    let base = abs("x");
    let room = 1024 - base.chars().count();
    let at_limit = format!("{base}{}", "a".repeat(room));
    let over = format!("{at_limit}a");
    assert_eq!(at_limit.chars().count(), 1024);
    for (field, set) in FIELDS {
        expect_ok(field, set, &at_limit);
        expect_rejected(field, set, &over);
    }
}

#[test]
fn shares_and_device_paths_are_rejected() {
    for bad in [
        r"\\10.0.0.9\s\ca.pem",
        r"\\server\share",
        "//server/share/x",
        r"\\.\pipe\x",
        r"\\.\COM1",
        r"\\?\UNC\host\share\ca.pem",
        r"\\?\GLOBALROOT\Device\x",
    ] {
        for (field, set) in FIELDS {
            expect_rejected(field, set, bad);
        }
    }
}

#[test]
fn an_extended_length_local_path_is_still_a_local_path() {
    // `\\?\C:\...` opens a file on the drive like any other; only its spelling is unusual.
    if cfg!(windows) {
        for (field, set) in FIELDS {
            expect_ok(field, set, r"\\?\C:\certs\ca.pem");
        }
    }
}

#[test]
fn reserved_device_names_are_rejected_in_any_component() {
    let base = abs("x");
    for name in ["NUL", "nul", "CON.txt", "aux", "Prn.pem", "COM1", "com9.pem", "LPT3", "NUL "] {
        for (field, set) in FIELDS {
            expect_rejected(field, set, &under(&base, &[name]));
            expect_rejected(field, set, &under(&base, &[name, "ca.pem"]));
        }
    }
}

#[test]
fn names_that_only_look_like_a_device_are_fine() {
    let base = abs("x");
    for name in ["console.pem", "nullable", "COM10", "COM0", "AUXILIARY.pem", "ca.nul", "lpt"] {
        for (field, set) in FIELDS {
            expect_ok(field, set, &under(&base, &[name]));
        }
    }
}
