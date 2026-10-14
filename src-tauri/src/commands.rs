//! The 16 IPC commands of docs/ipc.md. Each hands over to `AppService` or the flyout and holds no
//! rule of its own. They are `async` on purpose: a sync command runs on the main thread, which
//! must stay free for the window and the tray. Disk, registry and credential work goes through
//! `blocking`.

use std::sync::Arc;

use app_core::AppService;
use tauri::{AppHandle, State};
use usage_model::{
    ErrorCode, ExternalLink, HarnessError, HarnessHistory, HarnessId, SecretKind, SecretStatus,
    Settings, ShimStatus, Snapshot,
};

use crate::flyout::Flyout;
use crate::{links, shim_path};

#[cfg(test)]
mod tests;

type Service<'a> = State<'a, Arc<AppService>>;

/// Far above any API key or token, far below anything that would strain the IPC channel.
const MAX_SECRET_BYTES: usize = 16 * 1024;

fn internal(what: &'static str) -> HarnessError {
    HarnessError::new(ErrorCode::Internal, false).with_param("where", what)
}

fn invalid(field: &'static str) -> HarnessError {
    HarnessError::new(ErrorCode::InvalidInput, false).with_param("field", field)
}

/// Runs `work` on the blocking pool; a panic inside it becomes an opaque internal error.
async fn blocking<T, F>(work: F) -> Result<T, HarnessError>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, HarnessError> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(work).await.map_err(|_| internal("task"))?
}

/// The webview is ours, but the value still crosses a trust boundary.
fn check_secret(value: &str) -> Result<(), HarnessError> {
    if value.len() > MAX_SECRET_BYTES { Err(invalid("value")) } else { Ok(()) }
}

fn check_height(height: f64) -> Result<f64, HarnessError> {
    if height.is_finite() && height > 0.0 { Ok(height) } else { Err(invalid("height")) }
}

#[tauri::command]
pub async fn get_snapshot(service: Service<'_>) -> Result<Snapshot, HarnessError> {
    Ok(service.snapshot())
}

#[tauri::command]
pub async fn refresh(service: Service<'_>, harness: Option<HarnessId>) -> Result<(), HarnessError> {
    service.refresh(harness);
    Ok(())
}

#[tauri::command]
pub async fn get_settings(service: Service<'_>) -> Result<Settings, HarnessError> {
    Ok(service.settings())
}

#[tauri::command]
pub async fn save_settings(
    service: Service<'_>,
    settings: Settings,
) -> Result<Settings, HarnessError> {
    let service = Arc::clone(service.inner());
    blocking(move || service.save_settings(settings)).await
}

#[tauri::command]
pub async fn secret_status(service: Service<'_>) -> Result<SecretStatus, HarnessError> {
    let service = Arc::clone(service.inner());
    blocking(move || service.secret_status()).await
}

#[tauri::command]
pub async fn set_secret(
    service: Service<'_>,
    kind: SecretKind,
    value: String,
) -> Result<SecretStatus, HarnessError> {
    check_secret(&value)?;
    let service = Arc::clone(service.inner());
    blocking(move || service.set_secret(kind, value)).await
}

#[tauri::command]
pub async fn clear_secret(
    service: Service<'_>,
    kind: SecretKind,
) -> Result<SecretStatus, HarnessError> {
    let service = Arc::clone(service.inner());
    blocking(move || service.clear_secret(kind)).await
}

#[tauri::command]
pub async fn claude_shim_status(service: Service<'_>) -> Result<ShimStatus, HarnessError> {
    let service = Arc::clone(service.inner());
    blocking(move || Ok(service.claude_shim_status())).await
}

#[tauri::command]
pub async fn install_claude_shim(service: Service<'_>) -> Result<ShimStatus, HarnessError> {
    let service = Arc::clone(service.inner());
    blocking(move || {
        let exe = std::env::current_exe().map_err(|_| internal("current_exe"))?;
        let dir = exe.parent().ok_or_else(|| internal("current_exe"))?;
        let shim = shim_path::shim_exe_path(dir)?;
        service.install_claude_shim(&shim)
    })
    .await
}

#[tauri::command]
pub async fn uninstall_claude_shim(service: Service<'_>) -> Result<ShimStatus, HarnessError> {
    let service = Arc::clone(service.inner());
    blocking(move || service.uninstall_claude_shim()).await
}

#[tauri::command]
pub async fn get_history(
    service: Service<'_>,
    harness: HarnessId,
    limit: Option<u8>,
) -> Result<HarnessHistory, HarnessError> {
    service.history(harness, limit)
}

#[tauri::command]
pub async fn clear_history(service: Service<'_>) -> Result<(), HarnessError> {
    let service = Arc::clone(service.inner());
    blocking(move || service.clear_history()).await
}

#[tauri::command]
pub async fn open_link(link: ExternalLink) -> Result<(), HarnessError> {
    blocking(move || links::open(link, &links::launch_default_browser)).await
}

#[tauri::command]
pub async fn hide_flyout(flyout: State<'_, Arc<Flyout>>) -> Result<(), HarnessError> {
    flyout.hide();
    Ok(())
}

#[tauri::command]
pub async fn set_flyout_height(
    flyout: State<'_, Arc<Flyout>>,
    height: f64,
) -> Result<(), HarnessError> {
    flyout.set_height(check_height(height)?);
    Ok(())
}

#[tauri::command]
pub async fn quit_app(app: AppHandle) -> Result<(), HarnessError> {
    app.exit(0);
    Ok(())
}

#[tauri::command]
pub async fn get_update_status(
    updater: State<'_, crate::update::Updater>,
) -> Result<usage_model::UpdateStatus, HarnessError> {
    Ok(updater.manager.status())
}

#[tauri::command]
pub async fn check_update(
    updater: State<'_, crate::update::Updater>,
) -> Result<usage_model::UpdateStatus, HarnessError> {
    Ok(updater.manager.check().await)
}

#[tauri::command]
pub async fn download_update(
    updater: State<'_, crate::update::Updater>,
) -> Result<usage_model::UpdateStatus, HarnessError> {
    Ok(updater.manager.download().await)
}

#[tauri::command]
pub async fn install_update(
    app: AppHandle,
    service: Service<'_>,
    updater: State<'_, crate::update::Updater>,
) -> Result<usage_model::UpdateStatus, HarnessError> {
    crate::update::install(app, updater.inner(), Arc::clone(service.inner())).await
}
