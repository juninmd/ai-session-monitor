//! English wording of everything the Rust side shows: desktop notifications, the tray tooltip,
//! the tray menu and the startup failure toasts. Translate the text between the quotes only.
//!
//! Rules: keep every `{placeholder}` exactly as written (it is replaced at run time; a
//! template may reorder them but may not drop or rename one); keep product names, file names
//! and codes as they are; digits stay Latin; no markup. Windows cuts the tray tooltip at 127
//! characters, so the `tip_*` and `duration_*` entries must stay short. The comment above
//! each entry says where it appears. The field list lives in `messages.rs`.

use super::Messages;

pub(super) const MESSAGES: Messages = Messages {
    // Notification title when a window ran out of quota. harness = product name (kept as is).
    // Placeholders: {harness}.
    window_out_title: "{harness}: limit reached",
    // Notification body when a window ran out. window = the window description (see the
    // window_* entries, with the optional label); reset = reset_in or reset_unknown. The window
    // starts the sentence. Placeholders: {window}, {reset}.
    window_out_body: "{window} ran out. {reset}",
    // Sentence inserted as reset in the out and threshold bodies. span = a duration such as '1
    // h 53 min' (see span_*). Placeholders: {span}.
    reset_in: "Resets in {span}.",
    // Sentence inserted as reset when the source did not report a reset time. No placeholders.
    reset_unknown: "The reset time was not reported.",
    // Notification title when a window that was out has quota again (also the clock-driven
    // reset). harness = product name. Placeholders: {harness}.
    window_reset_title: "{harness}: quota available",
    // Body when the window has quota again and the next reset is unknown. share = share_free or
    // share_used. Placeholders: {window}, {share}.
    window_reset_body: "{window} has quota available again ({share}).",
    // Same as window_reset_body, plus the next reset. span = a duration (see span_*).
    // Placeholders: {window}, {share}, {span}.
    window_reset_body_next: "{window} has quota available again ({share}). Next reset in {span}.",
    // The available share inside window_reset_body. pct = whole number 0-100, without the
    // percent sign. Placeholders: {pct}.
    share_free: "{pct}% free",
    // The used share inside window_reset_body, when the user chose to show used quota. pct =
    // whole number 0-100. Placeholders: {pct}.
    share_used: "{pct}% used",
    // Notification title when usage crossed an alert threshold. level = the threshold, whole
    // number (for example 80). Placeholders: {harness}, {level}.
    threshold_title: "{harness}: {level}% of the limit used",
    // Body of the threshold alert. pct = the current used percentage, whole number; reset =
    // reset_in or reset_unknown. Placeholders: {window}, {pct}, {reset}.
    threshold_body: "{window} has used {pct}%. {reset}",
    // Notification title when the current pace would exhaust the window before it renews.
    // Placeholders: {harness}.
    pace_title: "{harness}: quota will run out before it renews",
    // Body of the pace alert. span = how long before the reset the quota is projected to run
    // out (see span_*). Placeholders: {window}, {span}.
    pace_body: "{window}: at the current pace it runs out about {span} before it resets.",
    // Body when a window reset by the clock with no new reading (the title is
    // window_reset_title). Placeholders: {window}.
    window_lapsed_body: "{window} reset and has quota available again.",
    // Notification title when the settings file was unreadable and was set aside. No
    // placeholders.
    settings_recovered_title: "Settings restored",
    // Body of that notification. file_name = the name of the saved copy (a file name, no
    // folder). Placeholders: {file_name}.
    settings_recovered_body: "The settings file was unreadable or invalid. A copy was kept \
                                 next to it ({file_name}) and the defaults were applied.",
    // Name of the 5-hour window, the start of a sentence in notifications (use the grammatical
    // form that fits there). No placeholders.
    window_five_hour: "5-hour window",
    // Name of the weekly window, as above. No placeholders.
    window_weekly: "Weekly window",
    // Name of the monthly window, as above. No placeholders.
    window_monthly: "Monthly window",
    // Name of the daily window, as above. No placeholders.
    window_daily: "Daily window",
    // Name of a spending budget, as above. No placeholders.
    window_budget: "Budget",
    // Name of a window of unknown kind, as above. No placeholders.
    window_other: "Window",
    // A window name followed by the source's own label for it. name = one of the window_*
    // entries; detail = text from the service (kept as is). Placeholders: {name}, {detail}.
    window_with_detail: "{name} ({detail})",
    // A duration shorter than one minute, as a whole phrase (notifications). No placeholders.
    span_less_than_minute: "less than 1 min",
    // A duration under one hour, in notifications. min = whole number 1-59. Placeholders:
    // {min}.
    span_minutes: "{min} min",
    // A whole number of hours under one day, in notifications. Placeholders: {hours}.
    span_hours: "{hours} h",
    // Hours and minutes under one day, in notifications. hours = 1-23; min = 1-59.
    // Placeholders: {hours}, {min}.
    span_hours_minutes: "{hours} h {min} min",
    // A whole number of days, in notifications. Placeholders: {days}.
    span_days: "{days} d",
    // Days and hours, in notifications. hours = 1-23. Placeholders: {days}, {hours}.
    span_days_hours: "{days} d {hours} h",
    // Tray tooltip when there is no reading yet. name = the product name or 'AI Session
    // Monitor'. Windows cuts the tooltip at 127 characters: keep it short. Placeholders:
    // {name}.
    tip_no_data: "{name}: no data",
    // Short window name in the tray tooltip: the 5-hour window. No placeholders.
    tip_window_five_hour: "5 hours",
    // Short window name in the tray tooltip: the weekly window. No placeholders.
    tip_window_weekly: "Week",
    // Short window name in the tray tooltip: the monthly window. No placeholders.
    tip_window_monthly: "Month",
    // Short window name in the tray tooltip: the daily window. No placeholders.
    tip_window_daily: "Day",
    // Short window name in the tray tooltip: a spending budget. No placeholders.
    tip_window_budget: "Budget",
    // Short window name in the tray tooltip: unknown kind. No placeholders.
    tip_window_other: "Window",
    // Start of the tooltip line. window = one of the tip_window_* entries. Placeholders:
    // {harness}, {window}.
    tip_head: "{harness} · {window}",
    // Tooltip line without a countdown. head = tip_head (or only the product name when space is
    // short); state = tip_out, tip_used or tip_available (or only a percentage). Placeholders:
    // {head}, {state}.
    tip_line: "{head}: {state}",
    // Tooltip line with a countdown. when = tip_resets_in or tip_back_in. Placeholders: {head},
    // {state}, {when}.
    tip_line_when: "{head}: {state} · {when}",
    // Tooltip state: the window has no quota left. No placeholders.
    tip_out: "out",
    // Tooltip state: used share. pct = a percentage with its sign, such as '28%' or '<1%'.
    // Placeholders: {pct}.
    tip_used: "{pct} used",
    // Tooltip state: available share. pct as above. Placeholders: {pct}.
    tip_available: "{pct} available",
    // Tooltip countdown to the next reset. duration = a tooltip duration (see duration_*), or
    // '1:53' in colon style. Placeholders: {duration}.
    tip_resets_in: "resets in {duration}",
    // Tooltip countdown when the window is out: when quota returns. Placeholders: {duration}.
    tip_back_in: "back in {duration}",
    // Tooltip duration under one hour. min = whole number. Placeholders: {min}.
    duration_minutes: "{min} min",
    // Tooltip duration in hours and minutes; min is always two digits ('1h 05m'). Placeholders:
    // {hours}, {min}.
    duration_hours_minutes: "{hours}h {min}m",
    // Tooltip duration in days and hours, for long windows. Placeholders: {days}, {hours}.
    duration_days_hours: "{days}d {hours}h",
    // Tooltip duration in decimal hours. value = a number with one decimal, using
    // decimal_separator. Placeholders: {value}.
    duration_decimal_hours: "{value} h",
    // The decimal separator of this language, used in duration_decimal_hours values (a single
    // character). No placeholders.
    decimal_separator: ".",
    // Tray menu entry (right click) that opens the panel. No placeholders.
    tray_open: "Open",
    // Tray menu entry that refreshes every source now. No placeholders.
    tray_refresh: "Refresh now",
    // Tray menu entry that quits the app. No placeholders.
    tray_quit: "Quit",
    // Notification body when turning on 'start with Windows' failed. The name in quotes must
    // match the setting's label in the UI. No placeholders.
    autostart_on_failed: "Could not turn on 'Start with Windows'.",
    // Notification body when turning off 'start with Windows' failed. No placeholders.
    autostart_off_failed: "Could not turn off 'Start with Windows'.",
    // Notification title when the app could not start (it closes right after). No placeholders.
    startup_title: "AI Session Monitor failed to start",
    // Startup failure reason: the user's profile folders were not found. No placeholders.
    startup_paths: "Could not locate your profile folders",
    // Startup failure reason: the data folder could not be created. No placeholders.
    startup_data_dir: "Could not create the app's data folder",
    // Startup failure reason: the panel window was not created. No placeholders.
    startup_window: "The panel window could not be created",
    // Startup failure reason: the tray icon could not be created. No placeholders.
    startup_tray: "Could not create the tray icon",
    // Startup failure reason: the settings file is broken and could not be set aside. No
    // placeholders.
    startup_settings_unreadable: "The settings file is unreadable and could not be moved aside \
                                     to start over",
    // Startup failure reason: the monitoring service refused to start. No placeholders.
    startup_service: "The monitoring service could not be started",
    // Body of the startup failure notification. what = one of the startup_* reasons; code = a
    // technical code in English (kept as is). Placeholders: {what}, {code}.
    startup_body: "{what} (code: {code}). The app will close.",
};
