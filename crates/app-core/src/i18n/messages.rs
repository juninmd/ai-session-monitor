//! The list of every string the Rust side shows. One field per string; each language file
//! (`lang_*.rs`) fills all of them. `{name}` placeholders are declared next to each field and
//! checked by the tests: a language that drops or misspells one fails them.

/// One field of [`Messages`]: its name, its template and the placeholders it must contain.
pub struct Entry {
    pub field: &'static str,
    pub template: &'static str,
    pub placeholders: &'static [&'static str],
}

macro_rules! messages {
    ($($(#[$doc:meta])* $field:ident [$($ph:literal),*];)*) => {
        /// Every user-visible string produced in Rust, for one language.
        pub struct Messages {
            $($(#[$doc])* pub $field: &'static str,)*
        }

        impl Messages {
            /// Every field with the placeholders its template must contain.
            pub fn entries(&self) -> Vec<Entry> {
                vec![$(Entry {
                    field: stringify!($field),
                    template: self.$field,
                    placeholders: &[$($ph),*],
                }),*]
            }
        }
    };
}

messages! {
    /// Notification title when a window ran out of quota. harness = product name (kept as is).
    window_out_title ["harness"];
    /// Notification body when a window ran out. window = the window description (see the
    /// window_* entries, with the optional label); reset = reset_in or reset_unknown. The
    /// window starts the sentence.
    window_out_body ["window", "reset"];
    /// Sentence inserted as reset in the out and threshold bodies. span = a duration such as '1
    /// h 53 min' (see span_*).
    reset_in ["span"];
    /// Sentence inserted as reset when the source did not report a reset time.
    reset_unknown [];
    /// Notification title when a window that was out has quota again (also the clock-driven
    /// reset). harness = product name.
    window_reset_title ["harness"];
    /// Body when the window has quota again and the next reset is unknown. share = share_free
    /// or share_used.
    window_reset_body ["window", "share"];
    /// Same as window_reset_body, plus the next reset. span = a duration (see span_*).
    window_reset_body_next ["window", "share", "span"];
    /// The available share inside window_reset_body. pct = whole number 0-100, without the
    /// percent sign.
    share_free ["pct"];
    /// The used share inside window_reset_body, when the user chose to show used quota. pct =
    /// whole number 0-100.
    share_used ["pct"];
    /// Notification title when usage crossed an alert threshold. level = the threshold, whole
    /// number (for example 80).
    threshold_title ["harness", "level"];
    /// Body of the threshold alert. pct = the current used percentage, whole number; reset =
    /// reset_in or reset_unknown.
    threshold_body ["window", "pct", "reset"];
    /// Notification title when the current pace would exhaust the window before it renews.
    pace_title ["harness"];
    /// Body of the pace alert. span = how long before the reset the quota is projected to run
    /// out (see span_*).
    pace_body ["window", "span"];
    /// Body when a window reset by the clock with no new reading (the title is
    /// window_reset_title).
    window_lapsed_body ["window"];
    /// Notification title when the settings file was unreadable and was set aside.
    settings_recovered_title [];
    /// Body of that notification. file_name = the name of the saved copy (a file name, no
    /// folder).
    settings_recovered_body ["file_name"];
    /// Name of the 5-hour window, the start of a sentence in notifications (use the grammatical
    /// form that fits there).
    window_five_hour [];
    /// Name of the weekly window, as above.
    window_weekly [];
    /// Name of the monthly window, as above.
    window_monthly [];
    /// Name of the daily window, as above.
    window_daily [];
    /// Name of a spending budget, as above.
    window_budget [];
    /// Name of a window of unknown kind, as above.
    window_other [];
    /// A window name followed by the source's own label for it. name = one of the window_*
    /// entries; detail = text from the service (kept as is).
    window_with_detail ["name", "detail"];
    /// A duration shorter than one minute, as a whole phrase (notifications).
    span_less_than_minute [];
    /// A duration under one hour, in notifications. min = whole number 1-59.
    span_minutes ["min"];
    /// A whole number of hours under one day, in notifications.
    span_hours ["hours"];
    /// Hours and minutes under one day, in notifications. hours = 1-23; min = 1-59.
    span_hours_minutes ["hours", "min"];
    /// A whole number of days, in notifications.
    span_days ["days"];
    /// Days and hours, in notifications. hours = 1-23.
    span_days_hours ["days", "hours"];
    /// Tray tooltip when there is no reading yet. name = the product name or 'AI Session
    /// Monitor'. Windows cuts the tooltip at 127 characters: keep it short.
    tip_no_data ["name"];
    /// Short window name in the tray tooltip: the 5-hour window.
    tip_window_five_hour [];
    /// Short window name in the tray tooltip: the weekly window.
    tip_window_weekly [];
    /// Short window name in the tray tooltip: the monthly window.
    tip_window_monthly [];
    /// Short window name in the tray tooltip: the daily window.
    tip_window_daily [];
    /// Short window name in the tray tooltip: a spending budget.
    tip_window_budget [];
    /// Short window name in the tray tooltip: unknown kind.
    tip_window_other [];
    /// Start of the tooltip line. window = one of the tip_window_* entries.
    tip_head ["harness", "window"];
    /// Tooltip line without a countdown. head = tip_head (or only the product name when space
    /// is short); state = tip_out, tip_used or tip_available (or only a percentage).
    tip_line ["head", "state"];
    /// Tooltip line with a countdown. when = tip_resets_in or tip_back_in.
    tip_line_when ["head", "state", "when"];
    /// Tooltip state: the window has no quota left.
    tip_out [];
    /// Tooltip state: used share. pct = a percentage with its sign, such as '28%' or '<1%'.
    tip_used ["pct"];
    /// Tooltip state: available share. pct as above.
    tip_available ["pct"];
    /// Tooltip countdown to the next reset. duration = a tooltip duration (see duration_*), or
    /// '1:53' in colon style.
    tip_resets_in ["duration"];
    /// Tooltip countdown when the window is out: when quota returns.
    tip_back_in ["duration"];
    /// Tooltip duration under one hour. min = whole number.
    duration_minutes ["min"];
    /// Tooltip duration in hours and minutes; min is always two digits ('1h 05m').
    duration_hours_minutes ["hours", "min"];
    /// Tooltip duration in days and hours, for long windows.
    duration_days_hours ["days", "hours"];
    /// Tooltip duration in decimal hours. value = a number with one decimal, using
    /// decimal_separator.
    duration_decimal_hours ["value"];
    /// The decimal separator of this language, used in duration_decimal_hours values (a single
    /// character).
    decimal_separator [];
    /// Tray menu entry (right click) that opens the panel.
    tray_open [];
    /// Tray menu entry that refreshes every source now.
    tray_refresh [];
    /// Tray menu entry that quits the app.
    tray_quit [];
    /// Notification body when turning on 'start with Windows' failed. The name in quotes must
    /// match the setting's label in the UI.
    autostart_on_failed [];
    /// Notification body when turning off 'start with Windows' failed.
    autostart_off_failed [];
    /// Notification title when the app could not start (it closes right after).
    startup_title [];
    /// Startup failure reason: the user's profile folders were not found.
    startup_paths [];
    /// Startup failure reason: the data folder could not be created.
    startup_data_dir [];
    /// Startup failure reason: the panel window was not created.
    startup_window [];
    /// Startup failure reason: the tray icon could not be created.
    startup_tray [];
    /// Startup failure reason: the settings file is broken and could not be set aside.
    startup_settings_unreadable [];
    /// Startup failure reason: the monitoring service refused to start.
    startup_service [];
    /// Body of the startup failure notification. what = one of the startup_* reasons; code = a
    /// technical code in English (kept as is).
    startup_body ["what", "code"];
}
