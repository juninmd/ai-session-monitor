//! The throttle of the OAuth source. Collectors are stateless and the scheduler cannot slow down
//! one sub-source of a harness, so the time and outcome of the last network attempt are kept in a
//! small file next to the shim's data. Nothing secret is stored: the plan name, the windows and
//! an error code.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use usage_model::HarnessError;

use super::Extras;
use super::usage::bounded;

/// docs/ipc.md: the Claude OAuth endpoint is polled no faster than every 300 s.
pub(crate) const MIN_WAIT_SECS: i64 = 300;
const MAX_WAIT_SECS: i64 = 3600;
const MAX_FILE_BYTES: usize = 128 * 1024;
/// The only error params the HTTP client produces; anything else in the file is dropped.
const PARAM_KEYS: [&str; 3] = ["httpStatus", "retryAfterSecs", "host"];
const MAX_PARAM_CHARS: usize = 64;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CacheFile {
    v: u32,
    attempted_at: i64,
    outcome: Stored,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum Stored {
    Ok { extras: Extras },
    Failed { error: HarnessError },
}

/// The outcome of the last attempt when it is recent enough to stand in for a new one.
pub(crate) fn fresh(path: &Path, now: i64) -> Option<Result<Extras, HarnessError>> {
    let file: CacheFile = usage_support::read_json_capped(path, MAX_FILE_BYTES).ok()?;
    if file.v != 1 {
        return None;
    }
    let wait = match &file.outcome {
        Stored::Ok { .. } => MIN_WAIT_SECS,
        Stored::Failed { error } => retry_after(error).clamp(MIN_WAIT_SECS, MAX_WAIT_SECS),
    };
    // A timestamp in the future (clock moved back) proves nothing about how long ago it was.
    let age = now.checked_sub(file.attempted_at)?;
    if !(0..wait).contains(&age) {
        return None;
    }
    Some(match file.outcome {
        Stored::Ok { extras } => Ok(Extras { plan: extras.plan, windows: bounded(extras.windows) }),
        Stored::Failed { error } => Err(scrub(error)),
    })
}

/// Best effort: without the file the endpoint is simply asked again on the next tick.
pub(crate) fn store(path: &Path, now: i64, outcome: &Result<Extras, HarnessError>) {
    let outcome = match outcome {
        Ok(extras) => Stored::Ok { extras: extras.clone() },
        Err(error) => Stored::Failed { error: error.clone() },
    };
    let file = CacheFile { v: 1, attempted_at: now, outcome };
    if let Ok(bytes) = serde_json::to_vec(&file) {
        let _ = usage_support::atomic_write(path, &bytes);
    }
}

fn retry_after(error: &HarnessError) -> i64 {
    error.params.get("retryAfterSecs").and_then(|secs| secs.parse().ok()).unwrap_or(0)
}

fn scrub(error: HarnessError) -> HarnessError {
    let params: BTreeMap<String, String> = error
        .params
        .into_iter()
        .filter(|(key, value)| {
            PARAM_KEYS.contains(&key.as_str()) && value.chars().count() <= MAX_PARAM_CHARS
        })
        .collect();
    HarnessError { params, ..error }
}

#[cfg(test)]
mod tests {
    use usage_model::{ErrorCode, UsageWindow, WindowKind};

    use super::*;

    const NOW: i64 = 1_800_000_000;

    fn extras() -> Extras {
        let mut window = UsageWindow::new("opus:weekly", WindowKind::Weekly);
        window.group = Some("opus".into());
        window.used_pct = Some(30.0);
        window.resets_at = Some(NOW + 3600);
        Extras { plan: Some("Max 5x".into()), windows: vec![window] }
    }

    fn rate_limited(retry_after: Option<&str>) -> HarnessError {
        let error = HarnessError::new(ErrorCode::RateLimited, true).with_param("httpStatus", "429");
        match retry_after {
            Some(secs) => error.with_param("retryAfterSecs", secs),
            None => error,
        }
    }

    #[test]
    fn a_success_stands_in_for_five_minutes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("claude").join("oauth-usage.json");
        assert!(fresh(&path, NOW).is_none(), "nothing stored yet");
        store(&path, NOW, &Ok(extras()));
        assert_eq!(fresh(&path, NOW).unwrap().unwrap(), extras());
        assert_eq!(fresh(&path, NOW + 299).unwrap().unwrap(), extras());
        assert!(fresh(&path, NOW + 300).is_none(), "the poll interval has passed");
    }

    #[test]
    fn a_failure_is_remembered_too_so_a_broken_endpoint_is_not_hammered() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("oauth-usage.json");
        store(
            &path,
            NOW,
            &Err(HarnessError::new(ErrorCode::ServerError, true).with_param("httpStatus", "503")),
        );
        let cached = fresh(&path, NOW + 120).unwrap().unwrap_err();
        assert_eq!((cached.code, cached.retryable), (ErrorCode::ServerError, true));
        assert_eq!(cached.params["httpStatus"], "503");
        assert!(fresh(&path, NOW + 300).is_none());
    }

    #[test]
    fn retry_after_can_stretch_the_wait_but_never_shorten_it() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("oauth-usage.json");
        store(&path, NOW, &Err(rate_limited(Some("1200"))));
        assert!(fresh(&path, NOW + 1199).is_some());
        assert!(fresh(&path, NOW + 1200).is_none());

        store(&path, NOW, &Err(rate_limited(Some("5"))));
        assert!(fresh(&path, NOW + 299).is_some(), "5 s asked, 300 s kept");

        store(&path, NOW, &Err(rate_limited(Some("999999"))));
        assert!(
            fresh(&path, NOW + 3599).is_some() && fresh(&path, NOW + 3600).is_none(),
            "capped at an hour"
        );
    }

    #[test]
    fn a_timestamp_from_the_future_is_not_trusted() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("oauth-usage.json");
        store(&path, NOW + 10, &Ok(extras()));
        assert!(fresh(&path, NOW).is_none());
        store(&path, i64::MIN, &Ok(extras()));
        assert!(fresh(&path, i64::MAX).is_none(), "no overflow either");
    }

    #[test]
    fn unusable_files_are_ignored() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("oauth-usage.json");
        for body in [
            "".to_owned(),
            "not json".to_owned(),
            r#"{"v":2,"attemptedAt":1800000000,"outcome":{"status":"ok","extras":{"plan":null,"windows":[]}}}"#.to_owned(),
            r#"{"v":1,"attemptedAt":1800000000,"outcome":{"status":"mystery"}}"#.to_owned(),
            r#"{"v":1,"attemptedAt":"yesterday","outcome":{"status":"failed"}}"#.to_owned(),
            " ".repeat(MAX_FILE_BYTES + 1),
        ] {
            std::fs::write(&path, &body).unwrap();
            assert!(fresh(&path, NOW).is_none(), "{}", &body[..body.len().min(40)]);
        }
    }

    #[test]
    fn edited_error_params_and_windows_are_cleaned_on_the_way_in() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("oauth-usage.json");
        let mut error =
            HarnessError::new(ErrorCode::Network, true).with_param("host", "api.example.dev");
        error = error
            .with_param("token", "sk-should-not-survive")
            .with_param("httpStatus", "9".repeat(500));
        store(&path, NOW, &Err(error));
        let cached = fresh(&path, NOW).unwrap().unwrap_err();
        assert_eq!(cached.params.keys().map(String::as_str).collect::<Vec<_>>(), ["host"]);

        let mut dirty = extras();
        dirty.windows[0].used_pct = Some(900.0);
        store(&path, NOW, &Ok(dirty));
        assert_eq!(fresh(&path, NOW).unwrap().unwrap().windows[0].used_pct, None);
    }
}
