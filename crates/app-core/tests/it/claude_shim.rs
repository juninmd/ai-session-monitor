//! The Claude Code statusLine shim: consent gate, plumbing, and the read that follows a change.

use std::path::{Path, PathBuf};

use usage_model::{ErrorCode, HarnessError, HarnessId};

use crate::fake_shim::{FakeShim, installed};
use crate::rig::{rig, seconds, secs, settle};
use crate::views::BASE;

const CLAUDE: HarnessId = HarnessId::Claude;
const SHIM_EXE: &str = "fabricated/statusline-shim.exe";

#[tokio::test(start_paused = true)]
async fn installing_needs_the_users_consent_and_touches_nothing_without_it() {
    let rig = rig().start().await;
    let error = rig.service.install_claude_shim(Path::new(SHIM_EXE)).expect_err("no consent");
    settle().await;

    assert_eq!((error.code, error.retryable), (ErrorCode::InvalidInput, false));
    assert_eq!(error.params.get("field").map(String::as_str), Some("shimConsent"));
    assert_eq!(error.params.len(), 1);
    assert!(rig.shim.calls().is_empty(), "the shim was not asked");
    assert_eq!(rig.fake(CLAUDE).count(), 1, "and Claude was not read again");
}

#[tokio::test(start_paused = true)]
async fn with_consent_the_shim_gets_paths_settings_and_clock_and_claude_is_read_again() {
    let rig = rig().settings(|s| s.claude.shim_consent = true).start().await;
    rig.at(seconds(1)).await;

    let status = rig.service.install_claude_shim(Path::new(SHIM_EXE)).expect("installed");
    settle().await;

    assert_eq!(status, installed());
    let calls = rig.shim.calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].op, "install");
    assert_eq!(calls[0].exe, Some(PathBuf::from(SHIM_EXE)));
    assert!(calls[0].claude.shim_consent);
    assert_eq!((&calls[0].data_dir, &calls[0].home_dir), (&rig.data_dir, &rig.home_dir));
    assert_eq!(calls[0].now_secs, BASE + 1);
    assert_eq!(rig.fake(CLAUDE).offsets_ms(rig.start), secs(&[0, 1]), "inside the 10 s gap");
}

#[tokio::test(start_paused = true)]
async fn uninstalling_needs_no_consent_and_reads_claude_again() {
    let rig = rig().start().await;
    rig.at(seconds(1)).await;

    let status = rig.service.uninstall_claude_shim().expect("uninstalled");
    settle().await;

    assert_eq!(status, installed());
    assert_eq!(rig.shim.calls().iter().map(|c| c.op).collect::<Vec<_>>(), ["uninstall"]);
    assert_eq!(rig.fake(CLAUDE).count(), 2);
}

#[tokio::test(start_paused = true)]
async fn a_shim_failure_is_passed_on_unchanged_and_triggers_no_read() {
    let failure = HarnessError::new(ErrorCode::InstallFailed, false).with_param("where", "shim");
    let shim = FakeShim::answering(Err(failure.clone()));
    let rig = rig().settings(|s| s.claude.shim_consent = true).shim(shim).start().await;
    rig.at(seconds(1)).await;

    let install = rig.service.install_claude_shim(Path::new(SHIM_EXE)).expect_err("fails");
    let uninstall = rig.service.uninstall_claude_shim().expect_err("fails");
    settle().await;

    assert_eq!((install, uninstall), (failure.clone(), failure));
    assert_eq!(rig.fake(CLAUDE).count(), 1);
}

#[tokio::test(start_paused = true)]
async fn refreshing_the_copy_gives_the_shim_the_bundled_build_and_reads_claude_no_more() {
    let rig = rig().settings(|s| s.claude.shim_consent = true).start().await;
    rig.at(seconds(1)).await;
    assert!(rig.shim.calls().is_empty(), "polling never touches the copy by itself");

    rig.service.refresh_claude_shim_copy(Path::new(SHIM_EXE)).expect("refreshed");
    settle().await;

    let calls = rig.shim.calls();
    assert_eq!(calls.len(), 1, "one refresh, one call");
    assert_eq!(calls[0].op, "refresh_copy");
    assert_eq!(calls[0].exe, Some(PathBuf::from(SHIM_EXE)));
    assert!(calls[0].claude.shim_consent);
    assert_eq!((&calls[0].data_dir, &calls[0].home_dir), (&rig.data_dir, &rig.home_dir));
    assert_eq!(calls[0].now_secs, BASE + 1);
    assert_eq!(rig.fake(CLAUDE).count(), 1, "a new copy changes nothing the tab shows");
}

#[tokio::test(start_paused = true)]
async fn refreshing_needs_the_users_consent_and_does_not_even_ask_the_shim_without_it() {
    let rig = rig().start().await;
    let refreshed = rig.service.refresh_claude_shim_copy(Path::new(SHIM_EXE));
    settle().await;

    assert_eq!(refreshed, Ok(()), "a startup step is quiet when there is nothing to do");
    assert!(rig.shim.calls().is_empty(), "the shim was not asked");
}

#[tokio::test(start_paused = true)]
async fn a_refresh_failure_is_passed_on_unchanged_and_triggers_no_read() {
    let failure = HarnessError::new(ErrorCode::InstallFailed, true).with_param("reason", "busy");
    let shim = FakeShim::answering(Err(failure.clone()));
    let rig = rig().settings(|s| s.claude.shim_consent = true).shim(shim).start().await;
    rig.at(seconds(1)).await;

    let refreshed = rig.service.refresh_claude_shim_copy(Path::new(SHIM_EXE));
    settle().await;

    assert_eq!(refreshed, Err(failure));
    assert_eq!(rig.fake(CLAUDE).count(), 1);
}

#[tokio::test(start_paused = true)]
async fn status_is_a_pass_through_that_reads_nothing() {
    let rig = rig().start().await;
    rig.at(seconds(1)).await;

    assert_eq!(rig.service.claude_shim_status(), installed());
    settle().await;
    assert_eq!(rig.shim.calls().iter().map(|c| c.op).collect::<Vec<_>>(), ["status"]);
    assert_eq!(rig.fake(CLAUDE).count(), 1);
}
