//! The part of `collect` that reads local files only (settings.json state and the shim's data
//! file). Blocking: callers run it on a blocking thread.

use usage_model::settings::ClaudeSettings;
use usage_model::{Ctx, ErrorCode, HarnessError, HarnessView, ShimState};

use crate::paths::Paths;
use crate::{datafile, shim, view};

pub(crate) fn view(settings: &ClaudeSettings, ctx: &Ctx) -> HarnessView {
    let Some(paths) = Paths::resolve(settings, ctx) else {
        let error =
            HarnessError::new(ErrorCode::InvalidInput, false).with_param("field", "configDir");
        return view::failed(error);
    };
    match shim::status_at(&paths).state {
        ShimState::Unreadable => {
            view::failed(HarnessError::new(ErrorCode::SettingsUnreadable, false))
        }
        ShimState::NotInstalled | ShimState::Foreign => view::needs_setup(false),
        ShimState::Outdated => view::needs_setup(true),
        ShimState::Installed => match datafile::read(&paths.rate_limits) {
            Ok(None) => view::waiting_first_session(),
            Ok(Some(file)) => view::from_data(file, ctx.now_secs),
            Err(error) => view::failed(error),
        },
    }
}
