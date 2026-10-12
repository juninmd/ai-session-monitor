//! `resolve_binary`: PATH/PATHEXT lookup against a temp dir of fake programs. The environment is
//! passed in, so nothing here depends on the machine's real PATH.

use std::env::join_paths;
use std::path::{Path, PathBuf};

use usage_support::{resolve_binary, resolve_binary_from_env, run_capped};

use crate::common::peer::peer;

const PATHEXT: &str = ".COM;.EXE;.BAT;.CMD";

struct Bin {
    dir: tempfile::TempDir,
}

impl Bin {
    fn new(files: &[&str]) -> Self {
        let dir = tempfile::tempdir().unwrap();
        for file in files {
            std::fs::write(dir.path().join(file), "fake").unwrap();
        }
        Self { dir }
    }

    fn path(&self, file: &str) -> PathBuf {
        self.dir.path().join(file)
    }

    fn path_env(&self) -> String {
        join_paths([self.dir.path()]).unwrap().to_string_lossy().into_owned()
    }

    fn resolve(&self, name: &str) -> Option<PathBuf> {
        resolve_binary(name, &self.path_env(), PATHEXT, &[])
    }
}

#[cfg(windows)]
#[test]
fn bare_names_find_cmd_exe_and_bat_shims() {
    let bin = Bin::new(&["codex.cmd", "agy.exe", "legacy.bat", "old.com"]);
    assert_eq!(bin.resolve("codex"), Some(bin.path("codex.cmd")));
    assert_eq!(bin.resolve("agy"), Some(bin.path("agy.exe")));
    assert_eq!(bin.resolve("legacy"), Some(bin.path("legacy.bat")));
    assert_eq!(bin.resolve("old"), Some(bin.path("old.com")));
}

#[cfg(windows)]
#[test]
fn a_name_with_an_extension_matches_that_file_only() {
    let bin = Bin::new(&["codex.cmd"]);
    assert_eq!(bin.resolve("codex.cmd"), Some(bin.path("codex.cmd")));
    assert_eq!(bin.resolve("CODEX.CMD"), Some(bin.path("CODEX.CMD")));
    assert_eq!(bin.resolve("codex.exe"), None);
}

#[cfg(windows)]
#[test]
fn pathext_order_decides_between_extensions_in_one_directory() {
    let bin = Bin::new(&["tool.cmd", "tool.exe"]);
    assert_eq!(bin.resolve("tool"), Some(bin.path("tool.exe")));
    let cmd_first = resolve_binary("tool", &bin.path_env(), ".CMD;.EXE", &[]);
    assert_eq!(cmd_first, Some(bin.path("tool.cmd")));
    let lower = resolve_binary("tool", &bin.path_env(), ".cmd;.exe", &[]);
    assert_eq!(lower, Some(bin.path("tool.cmd")), "PATHEXT is case-insensitive");
}

#[cfg(windows)]
#[test]
fn path_order_beats_pathext_order() {
    let (first, second) = (Bin::new(&["tool.cmd"]), Bin::new(&["tool.exe"]));
    let path = join_paths([first.dir.path(), second.dir.path()]).unwrap();
    let found = resolve_binary("tool", &path.to_string_lossy(), PATHEXT, &[]);
    assert_eq!(found, Some(first.path("tool.cmd")));
}

#[cfg(windows)]
#[test]
fn only_runnable_extensions_count_whatever_pathext_says() {
    let bin = Bin::new(&["script.ps1", "macro.vbs", "readme.txt"]);
    for name in ["script", "macro", "readme", "script.ps1", "macro.vbs", "readme.txt"] {
        let found = resolve_binary(name, &bin.path_env(), ".PS1;.VBS;.TXT;.EXE", &[]);
        assert_eq!(found, None, "{name}");
    }
}

#[cfg(windows)]
#[test]
fn an_empty_or_useless_pathext_falls_back_to_the_defaults() {
    let bin = Bin::new(&["codex.cmd"]);
    for pathext in ["", ";;", ".VBS;.JS"] {
        let found = resolve_binary("codex", &bin.path_env(), pathext, &[]);
        assert_eq!(found, Some(bin.path("codex.cmd")), "{pathext:?}");
    }
}

#[cfg(windows)]
#[test]
fn an_alternate_data_stream_is_never_a_program_even_by_absolute_path() {
    let bin = Bin::new(&["host.txt", "host.txt:payload.exe"]);
    let stream = bin.path("host.txt:payload.exe");
    assert!(stream.is_file(), "the control: the stream exists and a naive check would accept it");
    assert_eq!(resolve_binary(&stream.to_string_lossy(), "", PATHEXT, &[]), None);
    let bare = bin.path("host.txt:payload");
    assert_eq!(resolve_binary(&bare.to_string_lossy(), "", PATHEXT, &[]), None);
}

#[cfg(windows)]
#[test]
fn an_explicit_absolute_path_is_honoured_with_or_without_the_extension() {
    let bin = Bin::new(&["codex.cmd", "notes.txt"]);
    let elsewhere = "";
    let given = bin.path("codex.cmd");
    assert_eq!(resolve_binary(&given.to_string_lossy(), elsewhere, PATHEXT, &[]), Some(given));
    let no_ext = bin.path("codex");
    let found = resolve_binary(&no_ext.to_string_lossy(), elsewhere, PATHEXT, &[]);
    assert_eq!(found, Some(bin.path("codex.cmd")));

    let missing = bin.path("absent.exe");
    assert_eq!(resolve_binary(&missing.to_string_lossy(), elsewhere, PATHEXT, &[]), None);
    let text = bin.path("notes.txt");
    assert_eq!(resolve_binary(&text.to_string_lossy(), elsewhere, PATHEXT, &[]), None);
    let dir = bin.dir.path().to_string_lossy().into_owned();
    assert_eq!(resolve_binary(&dir, elsewhere, PATHEXT, &[]), None, "a directory is not a program");
}

#[test]
fn relative_paths_and_drive_relative_names_are_refused_even_when_present() {
    let bin = Bin::new(&["codex.cmd", "codex.exe"]);
    for name in [r".\codex.cmd", "./codex.cmd", r"..\codex.cmd", r"sub\codex.cmd", "C:codex.cmd"] {
        assert_eq!(bin.resolve(name), None, "{name}");
    }
}

#[test]
fn hostile_names_are_refused() {
    let bin = Bin::new(&["codex.cmd"]);
    let long = "a".repeat(5000);
    for name in [
        "",
        " ",
        "codex ",
        " codex",
        "codex.",
        "co\0dex",
        "codex\n",
        "co*",
        "co?",
        "co<x",
        "co|x",
        "co\"x",
        "codex.cmd:stream",
        long.as_str(),
    ] {
        assert_eq!(bin.resolve(name), None, "{name:?}");
    }
}

/// Runs `resolve_binary` inside a peer whose working directory is `cwd`.
async fn resolve_from(cwd: &Path, name: &str, path_env: &str) -> String {
    let mut spec = peer(&["resolve", name, path_env]);
    spec.cwd = Some(cwd.to_path_buf());
    let out = run_capped(spec).await.unwrap();
    String::from_utf8_lossy(&out.stdout).trim().to_owned()
}

#[cfg(windows)]
#[tokio::test]
async fn the_current_directory_is_never_searched() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("codex.cmd"), "planted").unwrap();
    std::fs::write(dir.path().join("codex.exe"), "planted").unwrap();
    // Empty, dot and relative PATH entries all mean "here" to a careless resolver.
    for path_env in ["", ";", ".", ".;", r".\", "sub;.;..", ";.;;"] {
        assert_eq!(resolve_from(dir.path(), "codex", path_env).await, "none", "{path_env:?}");
    }
    for name in [r".\codex.exe", "./codex.cmd", r"..\codex.exe", "codex.exe\\"] {
        let path_env = dir.path().to_string_lossy().into_owned();
        assert_eq!(resolve_from(dir.path(), name, &path_env).await, "none", "{name}");
    }
    // Control: the same directory named absolutely is found, so the file really is resolvable.
    let absolute = dir.path().to_string_lossy().into_owned();
    let found = resolve_from(dir.path(), "codex", &absolute).await;
    assert!(found.ends_with("codex.exe"), "{found}");
}

#[cfg(windows)]
#[test]
fn extra_dirs_are_searched_after_path_and_must_be_absolute() {
    let (on_path, extra) = (Bin::new(&["tool.cmd"]), Bin::new(&["tool.exe", "only-extra.exe"]));
    let dirs = [extra.dir.path().to_path_buf()];
    let found = resolve_binary("tool", &on_path.path_env(), PATHEXT, &dirs);
    assert_eq!(found, Some(on_path.path("tool.cmd")), "PATH wins over extra directories");
    let found = resolve_binary("only-extra", &on_path.path_env(), PATHEXT, &dirs);
    assert_eq!(found, Some(extra.path("only-extra.exe")));
    assert_eq!(resolve_binary("only-extra", &on_path.path_env(), PATHEXT, &[]), None);
}

#[cfg(windows)]
#[test]
fn unicode_directories_and_names_resolve() {
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().join("ferramentas ção");
    std::fs::create_dir(&home).unwrap();
    std::fs::write(home.join("ação.cmd"), "fake").unwrap();
    let path = join_paths([&home]).unwrap();
    let found = resolve_binary("ação", &path.to_string_lossy(), PATHEXT, &[]);
    assert_eq!(found, Some(home.join("ação.cmd")));
}

#[cfg(windows)]
#[test]
fn a_quoted_path_entry_with_a_semicolon_is_one_directory() {
    let dir = tempfile::tempdir().unwrap();
    let odd = dir.path().join("a;b");
    std::fs::create_dir(&odd).unwrap();
    std::fs::write(odd.join("tool.exe"), "fake").unwrap();
    let path_env = format!("\"{}\"", odd.display());
    assert_eq!(resolve_binary("tool", &path_env, PATHEXT, &[]), Some(odd.join("tool.exe")));
}

#[cfg(windows)]
#[test]
fn the_process_environment_wrapper_finds_system_tools_and_refuses_nonsense() {
    // cmd.exe lives in System32, which is on every Windows PATH; nothing here reads user data.
    let found = resolve_binary_from_env("cmd", &[]).unwrap();
    assert!(found.is_absolute() && found.is_file());
    assert!(found.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("exe")));
    assert_eq!(resolve_binary_from_env("ai-sm-no-such-tool-0b1c2d3e", &[]), None);
    assert_eq!(resolve_binary_from_env("", &[]), None);
}
