//! Finding the `codex` executable: the configured path, or `codex` on PATH. The npm shim
//! (`codex.cmd`) is returned as is; starting a batch file safely is the process runner's job.

use std::path::PathBuf;

use usage_model::settings::CodexSettings;
use usage_model::{ErrorCode, HarnessError};

const DEFAULT_NAME: &str = "codex";

fn wanted(settings: &CodexSettings) -> &str {
    // The settings form saves an emptied field as "": that means "search PATH", not "this path".
    settings
        .binary_path
        .as_deref()
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .unwrap_or(DEFAULT_NAME)
}

fn resolve_with(
    settings: &CodexSettings,
    lookup: impl FnOnce(&str) -> Option<PathBuf>,
) -> Result<PathBuf, HarnessError> {
    lookup(wanted(settings)).ok_or_else(|| HarnessError::new(ErrorCode::BinaryNotFound, false))
}

/// Walks PATH, which can be slow when it lists an unreachable network drive: keep it off the
/// async runtime's threads.
pub(crate) async fn resolve(settings: &CodexSettings) -> Result<PathBuf, HarnessError> {
    let settings = settings.clone();
    let search =
        move || resolve_with(&settings, |name| usage_support::resolve_binary_from_env(name, &[]));
    tokio::task::spawn_blocking(search)
        .await
        .unwrap_or_else(|_| Err(HarnessError::new(ErrorCode::Internal, false)))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use usage_support::resolve_binary;

    use super::*;

    /// A directory holding fabricated executables; `lookup` searches only it.
    struct Fixture {
        dir: tempfile::TempDir,
    }

    impl Fixture {
        fn new(files: &[&str]) -> Self {
            let dir = tempfile::tempdir().unwrap();
            for file in files {
                let path = dir.path().join(file);
                fs::write(&path, b"not a real program").unwrap();
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
                }
            }
            Self { dir }
        }

        fn resolve(&self, binary_path: Option<&str>) -> Result<PathBuf, HarnessError> {
            let settings = CodexSettings {
                binary_path: binary_path.map(str::to_owned),
                ..CodexSettings::default()
            };
            let path = self.dir.path().to_string_lossy().into_owned();
            resolve_with(&settings, |name| resolve_binary(name, &path, ".COM;.EXE;.BAT;.CMD", &[]))
        }
    }

    #[test]
    fn path_search_finds_the_npm_shim_before_the_extensionless_script() {
        let fixture = Fixture::new(&["codex", "codex.cmd", "codex.ps1"]);
        let found = fixture.resolve(None).unwrap();
        assert_eq!(found.file_name().and_then(|n| n.to_str()), Some("codex.cmd"));
    }

    #[test]
    fn a_native_exe_is_found_too() {
        let fixture = Fixture::new(&["codex.exe"]);
        assert!(fixture.resolve(None).unwrap().ends_with("codex.exe"));
    }

    #[test]
    fn an_explicit_absolute_path_wins_over_path() {
        let fixture = Fixture::new(&["codex.cmd", "custom-codex.exe"]);
        let explicit = fixture.dir.path().join("custom-codex.exe");
        let found = fixture.resolve(explicit.to_str()).unwrap();
        assert_eq!(found, explicit);
    }

    #[test]
    fn blank_settings_mean_search_path() {
        let fixture = Fixture::new(&["codex.cmd"]);
        for blank in [Some(""), Some("   "), Some("\t")] {
            assert!(fixture.resolve(blank).is_ok(), "{blank:?}");
        }
    }

    #[test]
    fn nothing_found_is_binary_not_found_and_not_retryable() {
        let fixture = Fixture::new(&["unrelated.exe"]);
        let missing = fixture.dir.path().join("nope.exe");
        for binary_path in [None, missing.to_str(), Some("relative\\codex.exe"), Some("co\0dex")] {
            let error = fixture.resolve(binary_path).unwrap_err();
            assert_eq!(
                (error.code, error.retryable),
                (ErrorCode::BinaryNotFound, false),
                "{binary_path:?}"
            );
            assert!(error.params.is_empty(), "no path may leak into the error");
        }
    }
}
