//! What must never happen to the user's settings.json: being rewritten when it cannot be parsed,
//! losing its keys, order or formatting, or being half written.

use usage_model::{ErrorCode, ShimState};

use super::*;
use crate::testenv::{settings_foreign, settings_plain};

const NOT_AN_OBJECT: [(&str, &[u8]); 9] = [
    ("trailing comma", b"{\"a\":1,}"),
    ("comment", b"{/* c */\"a\":1}"),
    ("array root", b"[1]"),
    ("scalar root", b"\"x\""),
    ("null root", b"null"),
    ("empty file", b""),
    ("whitespace only", b"  \n"),
    ("not utf-8", b"{\"a\":\"\xff\"}"),
    ("truncated", b"{\"a\":"),
];

fn assert_untouched_and_unreadable(env: &Env, name: &str) {
    let before = snapshot(env);
    let error = install(env).unwrap_err();
    assert_eq!((error.code, error.retryable), (ErrorCode::SettingsUnreadable, false), "{name}");
    let error = uninstall(env).unwrap_err();
    assert_eq!(error.code, ErrorCode::SettingsUnreadable, "{name}");
    assert_unchanged(env, &before, &format!("{name}: no backup, no copy of the shim, nothing"));
    assert_eq!(status(env).state, ShimState::Unreadable, "{name}");
}

#[test]
fn a_file_that_is_not_a_json_object_is_never_modified() {
    for (name, bytes) in NOT_AN_OBJECT {
        let env = Env::new();
        env.write_settings(bytes);
        assert_untouched_and_unreadable(&env, name);
    }
}

#[test]
fn an_oversized_file_or_a_directory_in_its_place_is_left_alone() {
    let env = Env::new();
    env.write_settings(vec![b' '; 2 * 1024 * 1024 + 1]);
    assert_untouched_and_unreadable(&env, "over 2 MiB");

    let dir = Env::new();
    std::fs::create_dir_all(dir.settings_json()).unwrap();
    assert_untouched_and_unreadable(&dir, "directory");
}

#[test]
fn keys_values_and_order_survive_a_round_trip_through_both_operations() {
    let env = Env::new();
    env.write_settings(settings_plain());
    let before = json(&env);
    install(&env).unwrap();
    let during = json(&env);
    assert_eq!(&keys(&during)[..keys(&before).len()], keys(&before).as_slice());
    uninstall(&env).unwrap();
    let after = json(&env);
    assert_eq!(keys(&after), keys(&before));
    assert_eq!(after, before);
    assert_eq!(after["zeta"], 1.5);
    assert!(after.get("alfa").is_some_and(Value::is_null), "null values are kept, not dropped");
}

#[test]
fn duplicate_keys_collapse_like_claude_code_reads_them_and_the_backup_keeps_the_original() {
    let original =
        "{\n  \"model\": \"opus\",\n  \"env\": {\"A\": \"1\"},\n  \"model\": \"sonnet\"\n}\n";
    let env = Env::new();
    env.write_settings(original);

    install(&env).unwrap();

    let installed = json(&env);
    assert_eq!(installed["model"], "sonnet", "JSON.parse keeps the last duplicate");
    assert_eq!(keys(&installed), ["model", "env", "statusLine"], "at the first one's position");
    assert_eq!(std::fs::read(env.backup()).unwrap(), original.as_bytes());
}

#[test]
fn bom_line_endings_indentation_and_the_final_newline_are_kept() {
    let object = |nl: &str, indent: &str, bom: &str, end: &str| {
        format!(
            "{bom}{{{nl}{indent}\"model\": \"opus\",{nl}{indent}\"statusLine\": {{{nl}{indent}{indent}\"type\": \"command\",{nl}{indent}{indent}\"command\": \"~/s.sh\"{nl}{indent}}}{nl}}}{end}"
        )
    };
    let variants = [
        ("bom + crlf + 4 spaces", object("\r\n", "    ", "\u{feff}", "\r\n")),
        ("tabs", object("\n", "\t", "", "\n")),
        ("no final newline", object("\n", "  ", "", "")),
    ];
    for (name, original) in variants {
        let env = Env::new();
        env.write_settings(&original);
        install(&env).unwrap();
        let installed = env.settings_text();
        assert_eq!(
            installed.starts_with('\u{feff}'),
            original.starts_with('\u{feff}'),
            "{name}: bom"
        );
        assert_eq!(installed.contains("\r\n"), original.contains("\r\n"), "{name}: crlf");
        assert_eq!(
            installed.replace("\r\n", "").contains('\n'),
            original.replace("\r\n", "").contains('\n'),
            "{name}: no mixed endings"
        );
        assert_eq!(installed.ends_with('\n'), original.ends_with('\n'), "{name}: final newline");
        assert_eq!(
            installed.lines().nth(1).map(|l| l.len() - l.trim_start().len()),
            original.lines().nth(1).map(|l| l.len() - l.trim_start().len()),
            "{name}: indent"
        );

        uninstall(&env).unwrap();
        assert_eq!(env.settings_bytes(), original.as_bytes(), "{name}: restored byte for byte");
    }
}

// Elsewhere a read-only file can still be replaced through its directory, so this is Windows only.
#[cfg(windows)]
#[test]
fn a_read_only_settings_file_is_reported_not_clobbered() {
    let env = Env::new();
    env.write_settings(settings_foreign());
    let writable = std::fs::metadata(env.settings_json()).unwrap().permissions();
    let mut read_only = writable.clone();
    read_only.set_readonly(true);
    std::fs::set_permissions(env.settings_json(), read_only).unwrap();

    let error = install(&env).unwrap_err();
    assert_eq!(
        (error.code, error.retryable, reason(&error)),
        (ErrorCode::InstallFailed, true, "write_failed")
    );
    assert_eq!(env.settings_text(), settings_foreign());
    let leftovers: Vec<_> =
        files_under(&env.config_dir()).into_iter().filter(|name| name.ends_with(".tmp")).collect();
    assert!(leftovers.is_empty(), "no temp file stays behind: {leftovers:?}");

    std::fs::set_permissions(env.settings_json(), writable).unwrap();
    assert_eq!(
        install(&env).unwrap().state,
        ShimState::Installed,
        "the retry works once it is writable"
    );
}

#[cfg(windows)]
#[test]
fn a_settings_file_held_open_by_another_program_is_reported_not_clobbered() {
    use std::os::windows::fs::OpenOptionsExt;

    let env = Env::new();
    env.write_settings(settings_foreign());
    // Readers allowed, but nobody may write, rename or delete while it is open.
    let held =
        std::fs::OpenOptions::new().read(true).share_mode(1).open(env.settings_json()).unwrap();

    let error = install(&env).unwrap_err();
    assert_eq!(
        (error.code, error.retryable, reason(&error)),
        (ErrorCode::InstallFailed, true, "write_failed")
    );
    assert_eq!(env.settings_text(), settings_foreign());
    drop(held);
    assert_eq!(install(&env).unwrap().state, ShimState::Installed);
}
