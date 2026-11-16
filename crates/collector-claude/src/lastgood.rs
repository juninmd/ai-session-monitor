//! The per-model windows as an extra source last gave them, so that a failure of both sources
//! (the CLI and the OAuth API) does not make them vanish from the tab for as long as it lasts.
//!
//! They come back only next to the `claude_oauth_partial` notice, which says they could not be
//! refreshed and why, and only for an hour: a weekly percentage moves slowly, but it is not shown
//! as current for longer than that. Nothing secret is stored: ids, labels, percentages and reset
//! times, in a small file next to the shim's data.

use std::path::Path;

use serde::{Deserialize, Serialize};
use usage_model::settings::ClaudeSettings;
use usage_model::status::is_expired;
use usage_model::{Ctx, HarnessView, UsageWindow};

use crate::notice;
use crate::oauth::{blocking, bounded};
use crate::paths::Paths;

const MAX_AGE_SECS: i64 = 3600;
/// Unchanged windows are confirmed on disk this often, not at every poll.
const CONFIRM_AFTER_SECS: i64 = 300;
const MAX_FILE_BYTES: usize = 64 * 1024;
const META_KEY: &str = "extra_source";
const SOURCES: [&str; 2] = ["cli_usage", "oauth_usage"];

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Stored {
    v: u32,
    saved_at: i64,
    source: String,
    windows: Vec<UsageWindow>,
}

/// The statusLine reports these itself; every other window on a view came from an extra source.
fn is_extra(window: &UsageWindow) -> bool {
    !matches!(window.id.as_str(), "five_hour" | "weekly" | "spend_limit")
}

/// Remembers the extra windows of a view that has them; gives the remembered ones back to a view
/// whose extra sources both failed.
pub(crate) async fn settle(view: &mut HarnessView, settings: &ClaudeSettings, ctx: &Ctx) {
    let Some(paths) = Paths::resolve(settings, ctx) else { return };
    let (path, now) = (paths.extras_last, ctx.now_secs);
    let extras: Vec<UsageWindow> = view.windows.iter().filter(|w| is_extra(w)).cloned().collect();
    if !extras.is_empty() {
        let source = view.meta.get(META_KEY).cloned().unwrap_or_default();
        blocking(move || remember(&path, now, source, extras)).await;
    } else if view.notices.iter().any(|n| n.code == notice::OAUTH_PARTIAL)
        && let Some((source, windows)) = blocking(move || recall(&path, now)).await.flatten()
    {
        view.windows.extend(windows);
        view.meta.insert(META_KEY.to_owned(), source);
    }
}

/// Best effort: without the file a failure simply shows the statusLine windows alone.
fn remember(path: &Path, now: i64, source: String, windows: Vec<UsageWindow>) {
    if let Some(old) = load(path)
        && old.windows == windows
        && (0..CONFIRM_AFTER_SECS).contains(&now.saturating_sub(old.saved_at))
    {
        return;
    }
    let stored = Stored { v: 1, saved_at: now, source, windows };
    if let Ok(bytes) = serde_json::to_vec(&stored) {
        let _ = usage_support::atomic_write(path, &bytes);
    }
}

fn recall(path: &Path, now: i64) -> Option<(String, Vec<UsageWindow>)> {
    let stored = load(path)?;
    // A timestamp in the future (clock moved back) proves nothing about how long ago it was.
    let age = now.checked_sub(stored.saved_at)?;
    if !(0..=MAX_AGE_SECS).contains(&age) || !SOURCES.contains(&stored.source.as_str()) {
        return None;
    }
    // The file is editable by anyone with access to the data directory.
    let windows: Vec<UsageWindow> = bounded(stored.windows)
        .into_iter()
        .filter(|w| is_extra(w) && !is_expired(w, now))
        .collect();
    (!windows.is_empty()).then_some((stored.source, windows))
}

fn load(path: &Path) -> Option<Stored> {
    let stored: Stored = usage_support::read_json_capped(path, MAX_FILE_BYTES).ok()?;
    (stored.v == 1).then_some(stored)
}

#[cfg(test)]
mod tests {
    use usage_model::WindowKind;

    use super::*;
    use crate::oauth::weekly;

    const NOW: i64 = 1_800_000_000;

    fn opus(resets_at: i64) -> Vec<UsageWindow> {
        vec![weekly("Opus", Some(12.5), Some(resets_at))]
    }

    #[test]
    fn remembered_windows_come_back_for_an_hour_and_never_past_their_reset() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("claude").join("extras-last.json");
        assert!(recall(&path, NOW).is_none(), "nothing was ever read");

        remember(&path, NOW, "cli_usage".to_owned(), opus(NOW + 7200));
        let (source, windows) = recall(&path, NOW + 3600).unwrap();
        assert_eq!((source.as_str(), windows), ("cli_usage", opus(NOW + 7200)));
        assert!(recall(&path, NOW + 3601).is_none(), "an hour is as long as old numbers stand in");
        assert!(recall(&path, NOW - 1).is_none(), "a clock set back proves nothing");

        remember(&path, NOW, "cli_usage".to_owned(), opus(NOW + 60));
        assert!(recall(&path, NOW + 60).is_none(), "the window reset since it was read");
    }

    #[test]
    fn an_edited_file_cannot_plant_a_source_or_replace_the_status_line_windows() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("extras-last.json");
        remember(&path, NOW, "somewhere_else".to_owned(), opus(NOW + 7200));
        assert!(recall(&path, NOW).is_none());

        let mut planted = UsageWindow::new("five_hour", WindowKind::FiveHour);
        planted.used_pct = Some(1.0);
        remember(&path, NOW, "oauth_usage".to_owned(), vec![planted]);
        assert!(recall(&path, NOW).is_none());
    }

    #[test]
    fn unchanged_windows_are_not_rewritten_at_every_poll() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("extras-last.json");
        remember(&path, NOW, "cli_usage".to_owned(), opus(NOW + 7200));
        remember(&path, NOW + 299, "cli_usage".to_owned(), opus(NOW + 7200));
        assert_eq!(load(&path).unwrap().saved_at, NOW);
        remember(&path, NOW + 300, "cli_usage".to_owned(), opus(NOW + 7200));
        assert_eq!(load(&path).unwrap().saved_at, NOW + 300, "confirmed after five minutes");
        remember(&path, NOW + 301, "cli_usage".to_owned(), opus(NOW + 9000));
        assert_eq!(load(&path).unwrap().saved_at, NOW + 301, "a change is written at once");
    }
}
