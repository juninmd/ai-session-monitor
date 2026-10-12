//! Setups that need the crate under test, so they cannot live in `common` (which the unit tests
//! include too).

use collector_claude::{collect, install_shim};
use usage_model::HarnessView;

use crate::common::{Env, ok_both};

/// A consented environment with the shim installed and nothing recorded yet.
pub fn installed() -> Env {
    let env = Env::new();
    install_shim(&env.settings(), &env.ctx(), &env.shim_source()).unwrap();
    env
}

/// Installed, with a fresh recording of both windows.
pub fn recording() -> Env {
    let env = installed();
    env.write_rate_limits(&ok_both());
    env
}

pub async fn view_of(env: &Env) -> HarnessView {
    collect(&env.settings(), &env.ctx()).await
}

pub async fn view_at(env: &Env, now: i64) -> HarnessView {
    collect(&env.settings(), &env.ctx_at(now)).await
}
