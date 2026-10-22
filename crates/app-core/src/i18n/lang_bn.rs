//! Bengali wording of everything the Rust side shows: desktop notifications, the tray tooltip,
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
    window_out_title: "{harness}: সীমা শেষ",
    // Notification body when a window ran out. window = the window description (see the
    // window_* entries, with the optional label); reset = reset_in or reset_unknown. The window
    // starts the sentence. Placeholders: {window}, {reset}.
    window_out_body: "{window}: কোটা শেষ হয়ে গেছে। {reset}",
    // Sentence inserted as reset in the out and threshold bodies. span = a duration such as '1
    // h 53 min' (see span_*). Placeholders: {span}.
    reset_in: "{span} পরে রিসেট হবে।",
    // Sentence inserted as reset when the source did not report a reset time. No placeholders.
    reset_unknown: "রিসেটের সময় জানানো হয়নি।",
    // Notification title when a window that was out has quota again (also the clock-driven
    // reset). harness = product name. Placeholders: {harness}.
    window_reset_title: "{harness}: কোটা আবার পাওয়া যাচ্ছে",
    // Body when the window has quota again and the next reset is unknown. share = share_free or
    // share_used. Placeholders: {window}, {share}.
    window_reset_body: "{window}: কোটা আবার পাওয়া যাচ্ছে ({share})।",
    // Same as window_reset_body, plus the next reset. span = a duration (see span_*).
    // Placeholders: {window}, {share}, {span}.
    window_reset_body_next: "{window}: কোটা আবার পাওয়া যাচ্ছে ({share})। পরবর্তী রিসেট {span} পরে।",
    // The available share inside window_reset_body. pct = whole number 0-100, without the
    // percent sign. Placeholders: {pct}.
    share_free: "{pct}% খালি",
    // The used share inside window_reset_body, when the user chose to show used quota. pct =
    // whole number 0-100. Placeholders: {pct}.
    share_used: "{pct}% ব্যবহৃত",
    // Notification title when usage crossed an alert threshold. level = the threshold, whole
    // number (for example 80). Placeholders: {harness}, {level}.
    threshold_title: "{harness}: সীমার {level}% ব্যবহৃত",
    // Body of the threshold alert. pct = the current used percentage, whole number; reset =
    // reset_in or reset_unknown. Placeholders: {window}, {pct}, {reset}.
    threshold_body: "{window}: {pct}% ব্যবহৃত হয়েছে। {reset}",
    // Notification title when the current pace would exhaust the window before it renews.
    // Placeholders: {harness}.
    pace_title: "{harness}: নবায়নের আগেই কোটা শেষ হয়ে যাবে",
    // Body of the pace alert. span = how long before the reset the quota is projected to run
    // out (see span_*). Placeholders: {window}, {span}.
    pace_body: "{window}: বর্তমান গতিতে রিসেটের প্রায় {span} আগে কোটা শেষ হয়ে যাবে।",
    // Body when a window reset by the clock with no new reading (the title is
    // window_reset_title). Placeholders: {window}.
    window_lapsed_body: "{window}: রিসেট হয়েছে এবং কোটা আবার পাওয়া যাচ্ছে।",
    // Notification title when the settings file was unreadable and was set aside. No
    // placeholders.
    settings_recovered_title: "সেটিংস পুনরুদ্ধার করা হয়েছে",
    // Body of that notification. file_name = the name of the saved copy (a file name, no
    // folder). Placeholders: {file_name}.
    settings_recovered_body: "সেটিংস ফাইলটি পড়া যায়নি বা অবৈধ ছিল। এর পাশে একটি কপি \
                                 ({file_name}) রাখা হয়েছে এবং ডিফল্ট মান প্রয়োগ করা হয়েছে।",
    // Name of the 5-hour window, the start of a sentence in notifications (use the grammatical
    // form that fits there). No placeholders.
    window_five_hour: "5 ঘণ্টার উইন্ডো",
    // Name of the weekly window, as above. No placeholders.
    window_weekly: "সাপ্তাহিক উইন্ডো",
    // Name of the monthly window, as above. No placeholders.
    window_monthly: "মাসিক উইন্ডো",
    // Name of the daily window, as above. No placeholders.
    window_daily: "দৈনিক উইন্ডো",
    // Name of a spending budget, as above. No placeholders.
    window_budget: "বাজেট",
    // Name of a window of unknown kind, as above. No placeholders.
    window_other: "উইন্ডো",
    // A window name followed by the source's own label for it. name = one of the window_*
    // entries; detail = text from the service (kept as is). Placeholders: {name}, {detail}.
    window_with_detail: "{name} ({detail})",
    // A duration shorter than one minute, as a whole phrase (notifications). No placeholders.
    span_less_than_minute: "1 মিনিটের কম",
    // A duration under one hour, in notifications. min = whole number 1-59. Placeholders:
    // {min}.
    span_minutes: "{min} মিনিট",
    // A whole number of hours under one day, in notifications. Placeholders: {hours}.
    span_hours: "{hours} ঘণ্টা",
    // Hours and minutes under one day, in notifications. hours = 1-23; min = 1-59.
    // Placeholders: {hours}, {min}.
    span_hours_minutes: "{hours} ঘণ্টা {min} মিনিট",
    // A whole number of days, in notifications. Placeholders: {days}.
    span_days: "{days} দিন",
    // Days and hours, in notifications. hours = 1-23. Placeholders: {days}, {hours}.
    span_days_hours: "{days} দিন {hours} ঘণ্টা",
    // Tray tooltip when there is no reading yet. name = the product name or 'AI Session
    // Monitor'. Windows cuts the tooltip at 127 characters: keep it short. Placeholders:
    // {name}.
    tip_no_data: "{name}: কোনো তথ্য নেই",
    // Short window name in the tray tooltip: the 5-hour window. No placeholders.
    tip_window_five_hour: "5 ঘণ্টা",
    // Short window name in the tray tooltip: the weekly window. No placeholders.
    tip_window_weekly: "সপ্তাহ",
    // Short window name in the tray tooltip: the monthly window. No placeholders.
    tip_window_monthly: "মাস",
    // Short window name in the tray tooltip: the daily window. No placeholders.
    tip_window_daily: "দিন",
    // Short window name in the tray tooltip: a spending budget. No placeholders.
    tip_window_budget: "বাজেট",
    // Short window name in the tray tooltip: unknown kind. No placeholders.
    tip_window_other: "উইন্ডো",
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
    tip_out: "শেষ",
    // Tooltip state: used share. pct = a percentage with its sign, such as '28%' or '<1%'.
    // Placeholders: {pct}.
    tip_used: "{pct} ব্যবহৃত",
    // Tooltip state: available share. pct as above. Placeholders: {pct}.
    tip_available: "{pct} উপলব্ধ",
    // Tooltip countdown to the next reset. duration = a tooltip duration (see duration_*), or
    // '1:53' in colon style. Placeholders: {duration}.
    tip_resets_in: "{duration} পরে রিসেট",
    // Tooltip countdown when the window is out: when quota returns. Placeholders: {duration}.
    tip_back_in: "{duration} পরে ফিরবে",
    // Tooltip duration under one hour. min = whole number. Placeholders: {min}.
    duration_minutes: "{min} মি.",
    // Tooltip duration in hours and minutes; min is always two digits ('1h 05m'). Placeholders:
    // {hours}, {min}.
    duration_hours_minutes: "{hours}ঘ {min}মি",
    // Tooltip duration in days and hours, for long windows. Placeholders: {days}, {hours}.
    duration_days_hours: "{days}দি {hours}ঘ",
    // Tooltip duration in decimal hours. value = a number with one decimal, using
    // decimal_separator. Placeholders: {value}.
    duration_decimal_hours: "{value} ঘ",
    // The decimal separator of this language, used in duration_decimal_hours values (a single
    // character). No placeholders.
    decimal_separator: ".",
    // Tray menu entry (right click) that opens the panel. No placeholders.
    tray_open: "খুলুন",
    // Tray menu entry that refreshes every source now. No placeholders.
    tray_refresh: "এখনই রিফ্রেশ করুন",
    // Tray menu entry that quits the app. No placeholders.
    tray_quit: "বন্ধ করুন",
    // Notification body when turning on 'start with Windows' failed. The name in quotes must
    // match the setting's label in the UI. No placeholders.
    autostart_on_failed: "'Windows-এর সঙ্গে চালু হোক' চালু করা যায়নি।",
    // Notification body when turning off 'start with Windows' failed. No placeholders.
    autostart_off_failed: "'Windows-এর সঙ্গে চালু হোক' বন্ধ করা যায়নি।",
    // Notification title when the app could not start (it closes right after). No placeholders.
    startup_title: "AI Session Monitor চালু হয়নি",
    // Startup failure reason: the user's profile folders were not found. No placeholders.
    startup_paths: "আপনার প্রোফাইল ফোল্ডার খুঁজে পাওয়া যায়নি",
    // Startup failure reason: the data folder could not be created. No placeholders.
    startup_data_dir: "অ্যাপের ডেটা ফোল্ডার তৈরি করা যায়নি",
    // Startup failure reason: the panel window was not created. No placeholders.
    startup_window: "প্যানেল উইন্ডো তৈরি করা যায়নি",
    // Startup failure reason: the tray icon could not be created. No placeholders.
    startup_tray: "ট্রে আইকন তৈরি করা যায়নি",
    // Startup failure reason: the settings file is broken and could not be set aside. No
    // placeholders.
    startup_settings_unreadable: "সেটিংস ফাইলটি পড়া যাচ্ছে না এবং নতুন করে শুরু করার জন্য \
                                     সরানো যায়নি",
    // Startup failure reason: the monitoring service refused to start. No placeholders.
    startup_service: "মনিটরিং সার্ভিস চালু করা যায়নি",
    // Body of the startup failure notification. what = one of the startup_* reasons; code = a
    // technical code in English (kept as is). Placeholders: {what}, {code}.
    startup_body: "{what} (কোড: {code})। অ্যাপটি বন্ধ হয়ে যাবে।",
};
