//! Keeps polling alive. `AppService::run` ends by panic when something under it breaks (a sink
//! that panics, say); without a restart every tab would stay frozen until the app is relaunched.

use std::time::Duration;

use tokio::task::{JoinError, JoinHandle};
use tokio::time::sleep;

use crate::diag;

#[cfg(test)]
mod tests;

const FIRST_WAIT: Duration = Duration::from_secs(2);
const MAX_WAIT: Duration = Duration::from_secs(30);

/// Runs the task `spawn` starts and starts it again after a panic or a cancellation, waiting 2 s
/// and doubling up to 30 s. A normal return is `shutdown`: supervision ends with it.
pub async fn supervise(mut spawn: impl FnMut() -> JoinHandle<()>) {
    let mut wait = FIRST_WAIT;
    loop {
        let Some(why) = abnormal(&spawn().await) else { return };
        diag::warn("polling stopped", &format_args!("{why}; restarting in {} s", wait.as_secs()));
        sleep(wait).await;
        wait = (wait * 2).min(MAX_WAIT);
    }
}

/// Why the task ended, unless it ended the way `shutdown` makes it end. The panic payload is not
/// repeated: it can hold anything the panicking code formatted.
fn abnormal(joined: &Result<(), JoinError>) -> Option<&'static str> {
    match joined {
        Ok(()) => None,
        Err(error) if error.is_panic() => Some("it panicked"),
        Err(_) => Some("it was cancelled"),
    }
}
