//! Windows NSIS installation adapter. Debug builds cannot launch an installer.

use std::path::PathBuf;
use std::sync::Arc;

use app_core::AppService;
use tauri::AppHandle;
use usage_model::{ErrorCode, HarnessError};

fn failed() -> HarnessError {
    HarnessError::new(ErrorCode::Internal, false)
}

#[cfg(not(all(windows, target_arch = "x86_64", not(debug_assertions))))]
pub async fn install(
    _app: AppHandle,
    _service: Arc<AppService>,
    _root: PathBuf,
    _bytes: Vec<u8>,
) -> Result<(), HarnessError> {
    Err(failed())
}

#[cfg(all(windows, target_arch = "x86_64", not(debug_assertions)))]
pub async fn install(
    app: AppHandle,
    service: Arc<AppService>,
    root: PathBuf,
    bytes: Vec<u8>,
) -> Result<(), HarnessError> {
    use std::os::windows::process::CommandExt;
    use std::process::{Command, Stdio};
    use std::time::Duration;

    // Stage before stopping anything: a full disk must not interrupt monitoring.
    let staged = tauri::async_runtime::spawn_blocking(move || {
        std::fs::create_dir_all(&root).map_err(|_| failed())?;
        let dir =
            tempfile::Builder::new().prefix("signed-").tempdir_in(root).map_err(|_| failed())?;
        let path = dir.path().join("update-setup.exe");
        usage_support::atomic_write(&path, &bytes).map_err(|_| failed())?;
        Ok::<_, HarnessError>((dir, path))
    })
    .await
    .map_err(|_| failed())??;
    let drained = service.shutdown_and_wait(Duration::from_secs(5)).await;
    if !drained {
        restore_polling(service);
        return Err(failed());
    }
    let started = tauri::async_runtime::spawn_blocking(move || {
        let (dir, path) = staged;
        let child = Command::new(path)
            .args(["/P", "/UPDATE", "/R"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(0x0800_0000)
            .spawn()
            .map_err(|_| failed())?;
        // NSIS must retain the installer while this process exits. Do not inherit CLI arguments.
        let _retained = dir.keep();
        drop(child);
        Ok::<_, HarnessError>(())
    })
    .await
    .map_err(|_| failed())
    .and_then(|result| result);
    if started.is_err() {
        restore_polling(service);
        return Err(failed());
    }
    app.exit(0);
    Ok(())
}

#[cfg(all(windows, target_arch = "x86_64", not(debug_assertions)))]
fn restore_polling(service: Arc<AppService>) {
    tauri::async_runtime::spawn(async move {
        // A timed-out shutdown may still be unwinding its children. Resume only after it ends.
        loop {
            if service.resume_after_failed_update() {
                crate::setup::supervise_polling(service);
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    });
}
