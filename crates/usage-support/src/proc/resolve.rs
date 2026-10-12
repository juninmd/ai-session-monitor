//! Finds an executable the way the Windows shell does (PATH, then PATHEXT) but never looks in the
//! current directory, so a planted `codex.exe` next to the app cannot be picked up.

use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// What `CreateProcess` can start directly; script hosts (`.vbs`, `.ps1`) are deliberately absent.
const RUNNABLE: [&str; 4] = ["com", "exe", "bat", "cmd"];
const DEFAULT_PATHEXT: &str = ".COM;.EXE;.BAT;.CMD";
const MAX_NAME_BYTES: usize = 4096;

/// Resolves `name_or_path` to an existing executable file.
///
/// - An absolute path is honoured as given (an extension is tried from `pathext_env` when it has
///   none); a relative path such as `.\codex.exe` is refused, since it would mean the current
///   directory.
/// - A bare name is searched in the absolute directories of `path_env`, then in `extra_dirs`.
///   Empty and relative entries are skipped. Within a directory the name is tried as given when it
///   already ends in a runnable extension, then with each PATHEXT extension in order.
/// - Names with control characters (NUL included), wildcards, a trailing dot or space, or an
///   alternate-data-stream colon are refused.
///
/// The environment is passed in so callers and tests do not depend on the real PATH.
pub fn resolve_binary(
    name_or_path: &str,
    path_env: &str,
    pathext_env: &str,
    extra_dirs: &[PathBuf],
) -> Option<PathBuf> {
    if !is_acceptable(name_or_path) {
        return None;
    }
    let extensions = extensions(pathext_env);
    let given = Path::new(name_or_path);
    if given.is_absolute() {
        let file = given.file_name()?;
        // The drive colon sits in the prefix, so one in the file name is an alternate data stream.
        if file.to_string_lossy().contains(':') {
            return None;
        }
        return find_in_dir(given.parent()?, file, &extensions);
    }
    if name_or_path.contains(['/', '\\', ':']) {
        return None;
    }
    search_dirs(path_env, extra_dirs)
        .into_iter()
        .find_map(|dir| find_in_dir(&dir, OsStr::new(name_or_path), &extensions))
}

/// [`resolve_binary`] with the PATH and PATHEXT of this process.
pub fn resolve_binary_from_env(name_or_path: &str, extra_dirs: &[PathBuf]) -> Option<PathBuf> {
    let path = env::var_os("PATH").unwrap_or_default();
    let pathext = env::var_os("PATHEXT").unwrap_or_default();
    resolve_binary(name_or_path, &path.to_string_lossy(), &pathext.to_string_lossy(), extra_dirs)
}

fn is_acceptable(name: &str) -> bool {
    let forbidden = |c: char| c.is_control() || matches!(c, '<' | '>' | '"' | '|' | '?' | '*');
    !name.is_empty()
        && name.len() <= MAX_NAME_BYTES
        && !name.contains(forbidden)
        && name.trim() == name
        && !name.ends_with(['.', '/', '\\'])
}

/// Only absolute directories: an empty or relative entry means "the current directory".
fn search_dirs(path_env: &str, extra_dirs: &[PathBuf]) -> Vec<PathBuf> {
    let from_path = env::split_paths(path_env);
    from_path.chain(extra_dirs.iter().cloned()).filter(|dir| dir.is_absolute()).collect()
}

/// Runnable extensions from PATHEXT in order, lower-case and without the dot; the default list when
/// PATHEXT is empty or names nothing runnable.
fn extensions(pathext_env: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for entry in pathext_env.split(';') {
        let ext = entry.trim().trim_start_matches('.').to_ascii_lowercase();
        if RUNNABLE.contains(&ext.as_str()) && !found.contains(&ext) {
            found.push(ext);
        }
    }
    if found.is_empty() && pathext_env != DEFAULT_PATHEXT {
        return extensions(DEFAULT_PATHEXT);
    }
    found
}

fn find_in_dir(dir: &Path, file: &OsStr, extensions: &[String]) -> Option<PathBuf> {
    let as_given = has_runnable_extension(file, extensions).then(|| file.to_owned());
    let with_extension = extensions.iter().map(|ext| {
        let mut name = file.to_owned();
        name.push(".");
        name.push(ext);
        name
    });
    as_given.into_iter().chain(with_extension).map(|name| dir.join(name)).find(|p| is_runnable(p))
}

fn has_runnable_extension(file: &OsStr, extensions: &[String]) -> bool {
    let ext = Path::new(file).extension().map(|e| e.to_string_lossy().to_ascii_lowercase());
    ext.is_some_and(|ext| extensions.contains(&ext))
}

#[cfg(windows)]
fn is_runnable(path: &Path) -> bool {
    path.is_file()
}

#[cfg(not(windows))]
fn is_runnable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata().is_ok_and(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_absolute_directories_are_searched() {
        let abs = if cfg!(windows) { r"C:\tools" } else { "/usr/local/bin" };
        let extra = [PathBuf::from("relative"), PathBuf::from(abs)];
        let joined = env::join_paths([abs, "relative", ".", ""]).unwrap();
        let dirs = search_dirs(&joined.to_string_lossy(), &extra);
        assert_eq!(dirs, [PathBuf::from(abs), PathBuf::from(abs)]);
        assert!(search_dirs("", &[]).is_empty());
    }

    #[test]
    fn pathext_keeps_runnable_entries_in_order_and_falls_back_to_the_default() {
        assert_eq!(extensions(".CMD;.exe;.Cmd;;  ;.VBS;com;.PS1"), ["cmd", "exe", "com"]);
        for useless in ["", ";;", ".VBS;.JS", "garbage"] {
            assert_eq!(extensions(useless), ["com", "exe", "bat", "cmd"], "{useless:?}");
        }
    }

    #[test]
    fn unacceptable_names_are_refused_before_touching_the_disk() {
        let long = "a".repeat(5000);
        for name in [
            "",
            " ",
            " codex",
            "codex ",
            "codex.",
            "co\0dex",
            "co\ndex",
            "a*b",
            "a?b",
            "a<b",
            "a|b",
            "a\"b",
            "\u{85}x",
            r"C:\tools\",
            "C:/tools/",
            long.as_str(),
        ] {
            assert!(!is_acceptable(name), "{name:?}");
        }
        for name in ["codex", "codex.cmd", "ferramentas-ção", "my tool", r"C:\a\b.exe"] {
            assert!(is_acceptable(name), "{name:?}");
        }
    }
}
