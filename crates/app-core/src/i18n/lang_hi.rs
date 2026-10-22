//! Hindi wording of everything the Rust side shows: desktop notifications, the tray tooltip,
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
    window_out_title: "{harness}: सीमा पूरी हो गई",
    // Notification body when a window ran out. window = the window description (see the
    // window_* entries, with the optional label); reset = reset_in or reset_unknown. The window
    // starts the sentence. Placeholders: {window}, {reset}.
    window_out_body: "{window} समाप्त हो गया। {reset}",
    // Sentence inserted as reset in the out and threshold bodies. span = a duration such as '1
    // h 53 min' (see span_*). Placeholders: {span}.
    reset_in: "{span} में रीसेट होगा।",
    // Sentence inserted as reset when the source did not report a reset time. No placeholders.
    reset_unknown: "रीसेट का समय नहीं बताया गया।",
    // Notification title when a window that was out has quota again (also the clock-driven
    // reset). harness = product name. Placeholders: {harness}.
    window_reset_title: "{harness}: कोटा उपलब्ध है",
    // Body when the window has quota again and the next reset is unknown. share = share_free or
    // share_used. Placeholders: {window}, {share}.
    window_reset_body: "{window} में फिर से कोटा उपलब्ध है ({share})।",
    // Same as window_reset_body, plus the next reset. span = a duration (see span_*).
    // Placeholders: {window}, {share}, {span}.
    window_reset_body_next: "{window} में फिर से कोटा उपलब्ध है ({share})। अगला रीसेट {span} में।",
    // The available share inside window_reset_body. pct = whole number 0-100, without the
    // percent sign. Placeholders: {pct}.
    share_free: "{pct}% बचा",
    // The used share inside window_reset_body, when the user chose to show used quota. pct =
    // whole number 0-100. Placeholders: {pct}.
    share_used: "{pct}% उपयोग हुआ",
    // Notification title when usage crossed an alert threshold. level = the threshold, whole
    // number (for example 80). Placeholders: {harness}, {level}.
    threshold_title: "{harness}: सीमा का {level}% उपयोग हो चुका",
    // Body of the threshold alert. pct = the current used percentage, whole number; reset =
    // reset_in or reset_unknown. Placeholders: {window}, {pct}, {reset}.
    threshold_body: "{window} में {pct}% उपयोग हो चुका है। {reset}",
    // Notification title when the current pace would exhaust the window before it renews.
    // Placeholders: {harness}.
    pace_title: "{harness}: नवीनीकरण से पहले कोटा खत्म हो जाएगा",
    // Body of the pace alert. span = how long before the reset the quota is projected to run
    // out (see span_*). Placeholders: {window}, {span}.
    pace_body: "{window}: मौजूदा गति से यह रीसेट से लगभग {span} पहले खत्म हो जाएगा।",
    // Body when a window reset by the clock with no new reading (the title is
    // window_reset_title). Placeholders: {window}.
    window_lapsed_body: "{window} रीसेट हो गया और कोटा फिर से उपलब्ध है।",
    // Notification title when the settings file was unreadable and was set aside. No
    // placeholders.
    settings_recovered_title: "सेटिंग्स बहाल की गईं",
    // Body of that notification. file_name = the name of the saved copy (a file name, no
    // folder). Placeholders: {file_name}.
    settings_recovered_body: "सेटिंग्स फ़ाइल पढ़ने योग्य नहीं थी या अमान्य थी। उसके बगल में एक प्रति ({file_name}) रखी गई है और डिफ़ॉल्ट मान लागू किए गए हैं।",
    // Name of the 5-hour window, the start of a sentence in notifications (use the grammatical
    // form that fits there). No placeholders.
    window_five_hour: "5 घंटे की विंडो",
    // Name of the weekly window, as above. No placeholders.
    window_weekly: "साप्ताहिक विंडो",
    // Name of the monthly window, as above. No placeholders.
    window_monthly: "मासिक विंडो",
    // Name of the daily window, as above. No placeholders.
    window_daily: "दैनिक विंडो",
    // Name of a spending budget, as above. No placeholders.
    window_budget: "बजट",
    // Name of a window of unknown kind, as above. No placeholders.
    window_other: "विंडो",
    // A window name followed by the source's own label for it. name = one of the window_*
    // entries; detail = text from the service (kept as is). Placeholders: {name}, {detail}.
    window_with_detail: "{name} ({detail})",
    // A duration shorter than one minute, as a whole phrase (notifications). No placeholders.
    span_less_than_minute: "1 मिनट से कम",
    // A duration under one hour, in notifications. min = whole number 1-59. Placeholders:
    // {min}.
    span_minutes: "{min} मिनट",
    // A whole number of hours under one day, in notifications. Placeholders: {hours}.
    span_hours: "{hours} घंटे",
    // Hours and minutes under one day, in notifications. hours = 1-23; min = 1-59.
    // Placeholders: {hours}, {min}.
    span_hours_minutes: "{hours} घंटे {min} मिनट",
    // A whole number of days, in notifications. Placeholders: {days}.
    span_days: "{days} दिन",
    // Days and hours, in notifications. hours = 1-23. Placeholders: {days}, {hours}.
    span_days_hours: "{days} दिन {hours} घंटे",
    // Tray tooltip when there is no reading yet. name = the product name or 'AI Session
    // Monitor'. Windows cuts the tooltip at 127 characters: keep it short. Placeholders:
    // {name}.
    tip_no_data: "{name}: कोई डेटा नहीं",
    // Short window name in the tray tooltip: the 5-hour window. No placeholders.
    tip_window_five_hour: "5 घंटे",
    // Short window name in the tray tooltip: the weekly window. No placeholders.
    tip_window_weekly: "सप्ताह",
    // Short window name in the tray tooltip: the monthly window. No placeholders.
    tip_window_monthly: "महीना",
    // Short window name in the tray tooltip: the daily window. No placeholders.
    tip_window_daily: "दिन",
    // Short window name in the tray tooltip: a spending budget. No placeholders.
    tip_window_budget: "बजट",
    // Short window name in the tray tooltip: unknown kind. No placeholders.
    tip_window_other: "विंडो",
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
    tip_out: "खत्म",
    // Tooltip state: used share. pct = a percentage with its sign, such as '28%' or '<1%'.
    // Placeholders: {pct}.
    tip_used: "{pct} उपयोग",
    // Tooltip state: available share. pct as above. Placeholders: {pct}.
    tip_available: "{pct} उपलब्ध",
    // Tooltip countdown to the next reset. duration = a tooltip duration (see duration_*), or
    // '1:53' in colon style. Placeholders: {duration}.
    tip_resets_in: "{duration} में रीसेट",
    // Tooltip countdown when the window is out: when quota returns. Placeholders: {duration}.
    tip_back_in: "{duration} में वापस",
    // Tooltip duration under one hour. min = whole number. Placeholders: {min}.
    duration_minutes: "{min} मिनट",
    // Tooltip duration in hours and minutes; min is always two digits ('1h 05m'). Placeholders:
    // {hours}, {min}.
    duration_hours_minutes: "{hours} घं {min} मि",
    // Tooltip duration in days and hours, for long windows. Placeholders: {days}, {hours}.
    duration_days_hours: "{days} दिन {hours} घं",
    // Tooltip duration in decimal hours. value = a number with one decimal, using
    // decimal_separator. Placeholders: {value}.
    duration_decimal_hours: "{value} घं",
    // The decimal separator of this language, used in duration_decimal_hours values (a single
    // character). No placeholders.
    decimal_separator: ".",
    // Tray menu entry (right click) that opens the panel. No placeholders.
    tray_open: "खोलें",
    // Tray menu entry that refreshes every source now. No placeholders.
    tray_refresh: "अभी रीफ़्रेश करें",
    // Tray menu entry that quits the app. No placeholders.
    tray_quit: "बाहर निकलें",
    // Notification body when turning on 'start with Windows' failed. The name in quotes must
    // match the setting's label in the UI. No placeholders.
    autostart_on_failed: "'Windows के साथ शुरू करें' चालू नहीं किया जा सका।",
    // Notification body when turning off 'start with Windows' failed. No placeholders.
    autostart_off_failed: "'Windows के साथ शुरू करें' बंद नहीं किया जा सका।",
    // Notification title when the app could not start (it closes right after). No placeholders.
    startup_title: "AI Session Monitor शुरू नहीं हो सका",
    // Startup failure reason: the user's profile folders were not found. No placeholders.
    startup_paths: "आपके प्रोफ़ाइल फ़ोल्डर नहीं मिले",
    // Startup failure reason: the data folder could not be created. No placeholders.
    startup_data_dir: "ऐप का डेटा फ़ोल्डर नहीं बनाया जा सका",
    // Startup failure reason: the panel window was not created. No placeholders.
    startup_window: "पैनल विंडो नहीं बनाई जा सकी",
    // Startup failure reason: the tray icon could not be created. No placeholders.
    startup_tray: "ट्रे आइकन नहीं बनाया जा सका",
    // Startup failure reason: the settings file is broken and could not be set aside. No
    // placeholders.
    startup_settings_unreadable: "सेटिंग्स फ़ाइल पढ़ने योग्य नहीं है और नए सिरे से शुरू करने के लिए उसे हटाया नहीं जा सका",
    // Startup failure reason: the monitoring service refused to start. No placeholders.
    startup_service: "मॉनिटरिंग सेवा शुरू नहीं की जा सकी",
    // Body of the startup failure notification. what = one of the startup_* reasons; code = a
    // technical code in English (kept as is). Placeholders: {what}, {code}.
    startup_body: "{what} (कोड: {code})। ऐप बंद हो जाएगा।",
};
