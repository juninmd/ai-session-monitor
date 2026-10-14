use std::sync::atomic::{AtomicBool, Ordering};

use serde_json::{Value, json};

use super::*;

fn fixture() -> Value {
    serde_json::from_str(include_str!("../../../usage-support/src/update/fixture.json")).unwrap()
}

fn manifest() -> Vec<u8> {
    serde_json::to_vec(&json!({"version":"1.2.3", "notes":"Release notes", "platforms":{
        "windows-x86_64-nsis": {"url":"https://github.com/juninmd/ai-session-monitor/releases/download/v1.2.3/app_1.2.3_x64-setup.exe",
            "signature":fixture()["signature"]}
    }})).unwrap()
}

fn bytes() -> Vec<u8> {
    let mut bytes = vec![b'A'; 128];
    bytes[..2].copy_from_slice(b"MZ");
    bytes
}

fn manager(installable: bool) -> Arc<UpdateManager> {
    UpdateManager::new(
        "0.1.0",
        fixture()["key"].as_str().unwrap().into(),
        installable,
        true,
        Arc::new(|_| {}),
    )
}

#[tokio::test]
async fn downloaded_bytes_must_verify_before_the_install_adapter_can_run() {
    let manager = manager(true);
    assert_eq!(manager.check_with(async { Ok(manifest()) }).await.phase, UpdatePhase::Available);
    let status = manager
        .download_with(|_, progress| async move {
            progress(0, None);
            progress(128, None);
            Ok(bytes())
        })
        .await;
    assert_eq!(status.phase, UpdatePhase::Ready);
    assert_eq!(status.downloaded_bytes, 128);
    assert_eq!(status.total_bytes, None);
    // A deferred verified installer is not discarded by a background or manual check.
    assert_eq!(
        manager.check_with(async { panic!("must not check") }).await.phase,
        UpdatePhase::Ready
    );
    let called = AtomicBool::new(false);
    manager
        .install(|received| {
            let called = &called;
            async move {
                assert_eq!(received, bytes());
                called.store(true, Ordering::SeqCst);
                Ok(())
            }
        })
        .await
        .unwrap();
    assert!(called.load(Ordering::SeqCst));
}

#[tokio::test]
async fn corrupt_installer_is_discarded_and_never_reaches_the_adapter() {
    let manager = manager(true);
    manager.check_with(async { Ok(manifest()) }).await;
    let status = manager.download_with(|_, _| async { Ok(vec![1; 128]) }).await;
    assert_eq!(status.failure, Some(UpdateFailure::Signature));
    assert!(manager.install(|_| async { panic!("must not install") }).await.is_err());
    assert!(lock(&manager.state).bytes.is_none());
}

#[tokio::test]
async fn operation_lock_drops_concurrent_checks_and_downloads() {
    let manager = manager(true);
    let (started, ready) = tokio::sync::oneshot::channel();
    let (finish, pending) = tokio::sync::oneshot::channel();
    let worker = Arc::clone(&manager);
    let task = tokio::spawn(async move {
        worker
            .check_with(async {
                started.send(()).unwrap();
                pending.await.unwrap();
                Ok(manifest())
            })
            .await
    });
    ready.await.unwrap();
    let status = manager.check_with(async { panic!("second check must not run") }).await;
    assert_eq!(status.phase, UpdatePhase::Checking);
    assert_eq!(
        manager.download_with(|_, _| async { panic!("download must not run") }).await.phase,
        UpdatePhase::Checking
    );
    finish.send(()).unwrap();
    assert_eq!(task.await.unwrap().phase, UpdatePhase::Available);
}

#[tokio::test]
async fn failure_keeps_raw_errors_out_of_status_and_can_be_retried() {
    let manager = manager(true);
    manager.check_with(async { Ok(manifest()) }).await;
    let error = HarnessError::new(ErrorCode::Network, true).with_param("secret", "token-sentinel");
    let status = manager.check_with(async { Err(error) }).await;
    assert_eq!(status.failure, Some(UpdateFailure::Check));
    assert!(status.available_version.is_none());
    assert!(!serde_json::to_string(&status).unwrap().contains("token-sentinel"));
    assert_eq!(manager.check_with(async { Ok(manifest()) }).await.phase, UpdatePhase::Available);
    let status = manager
        .download_with(|_, _| async { Err(HarnessError::new(ErrorCode::Timeout, true)) })
        .await;
    assert_eq!(status.failure, Some(UpdateFailure::Download));
    assert_eq!(manager.download_with(|_, _| async { Ok(bytes()) }).await.phase, UpdatePhase::Ready);
    let status = manager
        .install(|_| async { Err(HarnessError::new(ErrorCode::Internal, false)) })
        .await
        .unwrap();
    assert_eq!(status.failure, Some(UpdateFailure::Install));
}

#[tokio::test]
async fn debug_and_isolated_runs_can_never_install() {
    let manager = manager(false);
    manager.check_with(async { Ok(manifest()) }).await;
    manager.download_with(|_, _| async { Ok(bytes()) }).await;
    assert!(manager.install(|_| async { panic!("debug install must not run") }).await.is_err());
    let manager = UpdateManager::new("0.1.0", String::new(), false, false, Arc::new(|_| {}));
    assert_eq!(
        manager.check_with(async { panic!("isolated run must not fetch") }).await.phase,
        UpdatePhase::Unavailable
    );
}
