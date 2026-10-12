//! Unit tests of the entry point itself; the behaviour behind it is covered module by module and
//! in `tests/it`.

use std::collections::BTreeSet;

use super::*;

#[tokio::test]
async fn a_lost_blocking_task_is_a_retryable_internal_error() {
    let task = tokio::spawn(std::future::pending::<()>());
    task.abort();
    let view = task_lost(task.await.unwrap_err());
    assert_eq!(view.state, ViewState::Error);
    assert_eq!(view.error, Some(HarnessError::new(ErrorCode::Internal, true)));
    assert!(view.windows.is_empty() && view.notices.is_empty());
}

/// The scheduler spawns collectors on a multi-threaded runtime, which needs a `Send` future.
#[test]
fn the_collect_future_is_send() {
    fn is_send<T: Send>(_: &T) {}
    let settings = ClaudeSettings::default();
    let ctx = Ctx::new(0, std::path::PathBuf::new(), std::path::PathBuf::new());
    is_send(&collect(&settings, &ctx));
}

#[test]
fn notice_codes_are_unique_and_namespaced() {
    let unique: BTreeSet<_> = NOTICE_CODES.iter().collect();
    assert_eq!(unique.len(), NOTICE_CODES.len());
    assert!(NOTICE_CODES.iter().all(|code| code.starts_with("claude_")));
}
