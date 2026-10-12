use super::*;

/// A fabricated stand-in with a PE-looking header, big enough to pass the size guard.
fn fake_executable(dir: &Path) -> PathBuf {
    let path = dir.join(SHIM_FILE);
    let mut bytes = b"MZ".to_vec();
    bytes.resize(8192, 0);
    std::fs::write(&path, bytes).expect("write");
    path
}

#[test]
fn finds_the_shim_next_to_the_app() {
    let dir = tempfile::tempdir().expect("tempdir");
    let expected = fake_executable(dir.path());
    assert_eq!(shim_exe_path(dir.path()), Ok(expected));
}

#[test]
fn a_missing_shim_is_reported_as_missing() {
    let dir = tempfile::tempdir().expect("tempdir");
    assert_eq!(shim_exe_path(dir.path()), Err(ShimPathError::Missing));
    assert_eq!(shim_exe_path(&dir.path().join("no-such-dir")), Err(ShimPathError::Missing));
}

#[test]
fn the_debug_placeholder_is_rejected() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join(SHIM_FILE), b"placeholder: run prepare-sidecar\n")
        .expect("write");
    assert_eq!(shim_exe_path(dir.path()), Err(ShimPathError::Invalid));
}

#[test]
fn a_truncated_copy_is_rejected() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join(SHIM_FILE), b"MZ").expect("write");
    assert_eq!(shim_exe_path(dir.path()), Err(ShimPathError::Invalid));
}

#[cfg(windows)]
#[test]
fn a_big_file_that_is_not_an_executable_is_rejected() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join(SHIM_FILE), vec![b'x'; 8192]).expect("write");
    assert_eq!(shim_exe_path(dir.path()), Err(ShimPathError::Invalid));
}

#[test]
fn a_directory_with_the_shim_name_is_rejected() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir(dir.path().join(SHIM_FILE)).expect("mkdir");
    assert_eq!(shim_exe_path(dir.path()), Err(ShimPathError::Invalid));
}

#[test]
fn both_failures_become_install_failed_with_a_reason() {
    let missing = HarnessError::from(ShimPathError::Missing);
    let invalid = HarnessError::from(ShimPathError::Invalid);
    assert_eq!(missing.code, ErrorCode::InstallFailed);
    assert_eq!(invalid.code, ErrorCode::InstallFailed);
    assert_eq!(missing.params.get("reason").map(String::as_str), Some("shim_missing"));
    assert_eq!(invalid.params.get("reason").map(String::as_str), Some("shim_invalid"));
    assert!(!missing.retryable && !invalid.retryable);
}
