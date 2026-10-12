//! One poll: check the gates, find the server, ask it, read the answers.

use std::time::Duration;

use usage_model::settings::AntigravitySettings;
use usage_model::{Ctx, ErrorCode, HarnessError, HarnessId, HarnessView};

use crate::discovery::{Discovery, Target};
use crate::errors::most_specific;
use crate::quota::{Read, merge, parse_status, parse_summary};
use crate::rpc::{Limits, Method, Rpc};
use crate::view;

/// The scheduler cuts a poll off at 30 s; ending sooner keeps the error specific.
const DEFAULT_BUDGET: Duration = Duration::from_secs(24);

/// Tunables that tests shrink.
#[derive(Clone, Debug)]
pub(crate) struct Config {
    pub limits: Limits,
    /// Everything a poll may take, discovery included.
    pub budget: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self { limits: Limits::default(), budget: DEFAULT_BUDGET }
    }
}

pub(crate) async fn collect_with<D: Discovery>(
    settings: &AntigravitySettings,
    ctx: &Ctx,
    discovery: &D,
    config: &Config,
) -> HarnessView {
    let id = HarnessId::Antigravity;
    // The gates come first and touch nothing: no process table, no port, no file. `needs_setup`
    // does not apply to this source, which has no configuration and no secret of its own.
    if !settings.enabled {
        return HarnessView::disabled(id);
    }
    if !settings.consent {
        return HarnessView::needs_consent(id);
    }
    match tokio::time::timeout(config.budget, poll(ctx, discovery, config)).await {
        Ok(Ok(read)) => view::build(read, ctx.now_secs),
        Ok(Err(error)) => HarnessView::error(id, error),
        Err(_elapsed) => HarnessView::error(id, HarnessError::new(ErrorCode::Timeout, true)),
    }
}

async fn poll<D: Discovery>(
    ctx: &Ctx,
    discovery: &D,
    config: &Config,
) -> Result<Read, HarnessError> {
    let targets = discovery.find().await?;
    let rpc = Rpc::new(&config.limits)?;
    let mut failure: Option<HarnessError> = None;
    for target in &targets {
        match ask(&rpc, target, ctx.now_secs).await {
            Ok(read) => return Ok(read),
            Err(error) => {
                failure = Some(match failure {
                    Some(earlier) => most_specific(earlier, error),
                    None => error,
                });
            }
        }
    }
    Err(failure.map_or_else(|| HarnessError::new(ErrorCode::NotRunning, true), settle))
}

/// A server that answers the calls we know with 404 no longer has them.
fn settle(error: HarnessError) -> HarnessError {
    match error.code {
        ErrorCode::NotFound => HarnessError { code: ErrorCode::EndpointChanged, ..error },
        _ => error,
    }
}

/// The status first: it proves this is the server (right port, right token) and carries the plan.
/// The summary then adds the stated windows; the IDE's server has none (404). A status in a shape
/// we do not know still leaves the summary a chance, since the quota is what matters; any other
/// failure of the status (refusal, silence, not even JSON) says the summary would fail too.
async fn ask(rpc: &Rpc, target: &Target, now: i64) -> Result<Read, HarnessError> {
    let status = match rpc.call(target, Method::UserStatus).await {
        Ok(body) => parse_status(&body, now),
        Err(error) => Err(error),
    };
    if let Err(error) = &status
        && !is_unknown_shape(error)
    {
        return status;
    }
    let summary = match rpc.call(target, Method::QuotaSummary).await {
        Ok(body) => parse_summary(&body, now),
        Err(error) => Err(error),
    };
    match (status, summary) {
        (Ok(status), Ok(summary)) => Ok(merge(Some(status), Some(summary))),
        (Ok(status), Err(error))
            if error.code == ErrorCode::NotFound || !status.windows.is_empty() =>
        {
            Ok(merge(Some(status), None))
        }
        // Nothing to show and the reason is not "this server has no summary".
        (Ok(_), Err(error)) => Err(error),
        (Err(_), Ok(summary)) => Ok(merge(None, Some(summary))),
        (Err(first), Err(second)) => Err(most_specific(first, second)),
    }
}

/// The server answered, but not with something this collector reads.
fn is_unknown_shape(error: &HarnessError) -> bool {
    matches!(error.code, ErrorCode::NotFound | ErrorCode::EndpointChanged)
}

#[cfg(test)]
mod tests;
