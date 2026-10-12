//! Nothing started for a read may outlive it, however it ends: success, failure, timeout, or the
//! scheduler dropping the future at its own hard deadline.

use std::time::Duration;

use usage_model::ViewState;

use crate::common::process::{wait_for_pid, wait_until_gone};
use crate::common::{NOW, PATIENT, Rig};

async fn assert_app_server_gone(rig: &Rig, what: &str) {
    let pid = rig.pid().unwrap_or_else(|| panic!("{what}: the fake never started"));
    assert!(wait_until_gone(pid).await, "{what}: the app-server (pid {pid}) is still running");
}

#[tokio::test]
async fn a_successful_read_leaves_nothing_running() {
    let rig = Rig::new("ok");
    assert_eq!(rig.collect().await.state, ViewState::Ready);
    assert_app_server_gone(&rig, "success").await;
}

#[tokio::test]
async fn a_failed_read_leaves_nothing_running() {
    let modes = [
        "bad_handshake",
        "garbage_line",
        "huge_line",
        "bad_shape",
        "backend_500",
        "not_signed_in",
        "api_key",
    ];
    for mode in modes {
        let rig = Rig::new(mode);
        rig.collect().await;
        assert_app_server_gone(&rig, mode).await;
    }
}

#[tokio::test]
async fn a_timed_out_read_leaves_nothing_running() {
    let rig = Rig::new("hang");
    let view = rig.collect_within(Duration::from_secs(1)).await;
    assert_eq!(view.state, ViewState::Error);
    assert_app_server_gone(&rig, "timeout").await;
}

#[tokio::test]
async fn dropping_the_read_ends_the_process_too() {
    // The scheduler gives each poll a hard deadline and drops the future when it passes.
    let rig = Rig::new("hang");
    let (settings, ctx) = (rig.settings(), rig.ctx_at(NOW));
    let read = tokio::spawn(async move {
        collector_codex::collect_with_budget(&settings, &ctx, PATIENT).await
    });
    let pid = wait_for_pid(&rig.bin.path().join("fake-pid")).await.expect("the fake started");
    read.abort();
    assert!(read.await.unwrap_err().is_cancelled());
    assert!(wait_until_gone(pid).await, "the app-server outlived the dropped read");
}

#[tokio::test]
async fn reads_running_at_the_same_time_do_not_interfere() {
    let (both, five_only, week_only) =
        (Rig::new("ok"), Rig::new("plus_only"), Rig::new("pro_no_primary"));
    let (a, b, c) = tokio::join!(both.collect(), five_only.collect(), week_only.collect());
    let ids = |view: &usage_model::HarnessView| {
        view.windows.iter().map(|w| w.id.clone()).collect::<Vec<_>>()
    };
    assert_eq!(ids(&a), ["five_hour", "weekly"]);
    assert_eq!(ids(&b), ["five_hour"]);
    assert_eq!(ids(&c), ["weekly"]);
    let pids = [both.pid(), five_only.pid(), week_only.pid()];
    assert!(pids.iter().all(Option::is_some) && pids[0] != pids[1] && pids[1] != pids[2]);
}

/// The fake starts a grandchild that shares its pipes, like node behind an npm shim: ending only
/// the direct child would leave it running.
#[cfg(windows)]
#[tokio::test]
async fn the_whole_process_tree_is_ended() {
    let rig = Rig::new("grandchild");
    assert_eq!(rig.collect().await.state, ViewState::Ready);
    assert_app_server_gone(&rig, "tree").await;
    let child = rig.child_pid().expect("the fake started a grandchild");
    assert!(wait_until_gone(child).await, "the grandchild (pid {child}) is still running");
}

/// npm installs `codex.cmd`, a batch file; it must be started without any shell-string building,
/// even from a directory whose name has spaces and non-ASCII characters.
#[cfg(windows)]
#[tokio::test]
async fn an_npm_style_batch_shim_works_and_its_tree_is_ended() {
    let rig = Rig::named("grandchild", "codex shim ção ");
    let shim = rig.bin.path().join("codex.cmd");
    // `%~dp0` is the shim's own directory, as in the files npm writes.
    std::fs::write(&shim, "@echo off\r\n\"%~dp0fake_app_server.exe\" %*\r\n").unwrap();
    let settings = usage_model::settings::CodexSettings {
        binary_path: Some(shim.to_string_lossy().into_owned()),
        ..rig.settings()
    };
    let view = rig.collect_with(&settings).await;
    assert_eq!(view.state, ViewState::Ready, "{view:?}");
    assert_eq!(view.windows.len(), 2);
    assert_app_server_gone(&rig, "shim").await;
    let child = rig.child_pid().expect("the fake started a grandchild");
    assert!(wait_until_gone(child).await, "the grandchild (pid {child}) is still running");
}

#[tokio::test]
async fn the_app_server_starts_in_the_programs_own_directory() {
    // Not in the directory this app happens to run from: a shim looks there for commands first.
    let rig = Rig::new("ok");
    rig.collect().await;
    let started_in = std::fs::read_to_string(rig.bin.path().join("fake-cwd")).unwrap();
    let canonical = |path: &std::path::Path| std::fs::canonicalize(path).unwrap();
    assert_eq!(canonical(started_in.trim().as_ref()), canonical(rig.bin.path()));
    assert_ne!(canonical(rig.bin.path()), canonical(&std::env::current_dir().unwrap()));
}
