use serde::{Deserialize, Serialize};

use super::{DEFAULT_ALERT_THRESHOLDS, TimeFormat, WeekFormat};
use crate::HarnessId;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GeneralSettings {
    /// How often collectors poll while the app runs.
    pub interval_secs: u32,
    /// Start with Windows. Off until the user opts in because it writes a Run registry entry.
    pub autostart: bool,
    /// Harness the tray ring follows. `None` means the lowest available percentage of all.
    pub tray_source: Option<HarnessId>,
    /// Presentation only: does not change collection, notifications or the tray source.
    pub visible_tabs: Vec<HarnessId>,
    /// Available percentage at or below which a window turns to the warning colour.
    pub warn_at_pct: u8,
    pub notify_on_out: bool,
    pub notify_on_reset: bool,
    /// Used percentages (1..=99) that raise one toast each time a window crosses them. Empty
    /// switches the alerts off. Sorted, deduplicated, at most `MAX_ALERT_THRESHOLDS`.
    pub alert_thresholds: Vec<u8>,
    /// Poll less often while every window is far from its limit and its reset is far away.
    pub adaptive: bool,
    /// Keep a local record of past quota windows (percentages and times only) for the burndown.
    pub history_enabled: bool,
    pub time_format: TimeFormat,
    pub week_format: WeekFormat,
    /// Present quota as "used" instead of "available". Wording only: stored numbers stay used %.
    pub show_used: bool,
    /// The flyout opens by itself once, on the very first launch.
    pub first_run_done: bool,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            interval_secs: 60,
            autostart: false,
            tray_source: None,
            visible_tabs: HarnessId::ALL.to_vec(),
            warn_at_pct: 25,
            notify_on_out: false,
            notify_on_reset: false,
            alert_thresholds: DEFAULT_ALERT_THRESHOLDS.to_vec(),
            adaptive: true,
            history_enabled: true,
            time_format: TimeFormat::Hm,
            week_format: WeekFormat::Hours,
            show_used: false,
            first_run_done: false,
        }
    }
}
