//! The part of `collect` that reads local files only (settings.json state and the shim's data
//! file). Blocking: callers run it on a blocking thread.

use std::time::Duration;

use usage_model::settings::ClaudeSettings;
use usage_model::{Ctx, ErrorCode, HarnessError, HarnessView, Notice, Severity, ShimState};

use crate::paths::Paths;
use crate::{datafile, notice, shim, view};

/// Claude Code, an editor or a sync tool rewrites settings.json while we look at it, and the shim
/// replaces its recording at every status line refresh. One read that lands in the middle (a file
/// that is gone for a moment, half written or locked) must not pass for "the command was removed":
/// anything short of a healthy read is looked at again before it is believed.
const SETTLE_READS: u32 = 3;
const SETTLE_PAUSE: Duration = Duration::from_millis(60);

pub(crate) fn view(settings: &ClaudeSettings, ctx: &Ctx) -> HarnessView {
    let Some(paths) = Paths::resolve(settings, ctx) else {
        let error =
            HarnessError::new(ErrorCode::InvalidInput, false).with_param("field", "configDir");
        return view::failed(error);
    };
    let state = settled(|| shim::status_at(&paths).state, |s| *s == ShimState::Installed);
    match state {
        // Retryable: the file belongs to other programs, and the next tick may find it whole.
        ShimState::Unreadable => {
            view::failed(HarnessError::new(ErrorCode::SettingsUnreadable, true))
        }
        ShimState::NotInstalled => view::needs_setup(lost(&paths, notice::STATUS_LINE_REMOVED)),
        ShimState::Foreign => view::needs_setup(lost(&paths, notice::STATUS_LINE_REPLACED)),
        ShimState::Outdated => {
            view::needs_setup(Some(notice::plain(notice::SHIM_OUTDATED, Severity::Warn)))
        }
        ShimState::Installed => {
            let read = settled(
                || datafile::read(&paths.rate_limits),
                |outcome| !matches!(outcome, Err(error) if error.retryable),
            );
            match read {
                Ok(None) => view::waiting_first_session(),
                Ok(Some(file)) => view::from_data(file, ctx.now_secs),
                Err(error) => view::failed(error),
            }
        }
    }
}

/// A recording on disk proves the command ran here before: it was installed and something took it
/// out of settings.json. Without one this is a first setup, which needs no warning.
fn lost(paths: &Paths, code: &str) -> Option<Notice> {
    paths.rate_limits.is_file().then(|| notice::plain(code, Severity::Warn))
}

fn settled<T>(mut read: impl FnMut() -> T, healthy: impl Fn(&T) -> bool) -> T {
    let mut last = read();
    for _ in 1..SETTLE_READS {
        if healthy(&last) {
            break;
        }
        std::thread::sleep(SETTLE_PAUSE);
        last = read();
    }
    last
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    #[test]
    fn a_read_that_heals_is_believed_and_a_healthy_one_is_not_repeated() {
        let calls = Cell::new(0);
        let read = || {
            calls.set(calls.get() + 1);
            calls.get()
        };
        assert_eq!(settled(read, |n| *n >= 2), 2, "the second look found it whole");
        calls.set(0);
        assert_eq!(settled(read, |_| true), 1);
        calls.set(0);
        assert_eq!(settled(read, |_| false), SETTLE_READS, "what stays broken is reported");
    }
}
