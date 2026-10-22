//! Spanish (neutral international) wording of everything the Rust side shows: desktop notifications, the tray tooltip,
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
    window_out_title: "{harness}: límite agotado",
    // Notification body when a window ran out. window = the window description (see the
    // window_* entries, with the optional label); reset = reset_in or reset_unknown. The window
    // starts the sentence. Placeholders: {window}, {reset}.
    window_out_body: "{window} se agotó. {reset}",
    // Sentence inserted as reset in the out and threshold bodies. span = a duration such as '1
    // h 53 min' (see span_*). Placeholders: {span}.
    reset_in: "Se restablece en {span}.",
    // Sentence inserted as reset when the source did not report a reset time. No placeholders.
    reset_unknown: "No se informó la hora de restablecimiento.",
    // Notification title when a window that was out has quota again (also the clock-driven
    // reset). harness = product name. Placeholders: {harness}.
    window_reset_title: "{harness}: cuota disponible",
    // Body when the window has quota again and the next reset is unknown. share = share_free or
    // share_used. Placeholders: {window}, {share}.
    window_reset_body: "{window} vuelve a tener cuota disponible ({share}).",
    // Same as window_reset_body, plus the next reset. span = a duration (see span_*).
    // Placeholders: {window}, {share}, {span}.
    window_reset_body_next: "{window} vuelve a tener cuota disponible ({share}). Próximo restablecimiento en {span}.",
    // The available share inside window_reset_body. pct = whole number 0-100, without the
    // percent sign. Placeholders: {pct}.
    share_free: "{pct}% libre",
    // The used share inside window_reset_body, when the user chose to show used quota. pct =
    // whole number 0-100. Placeholders: {pct}.
    share_used: "{pct}% usado",
    // Notification title when usage crossed an alert threshold. level = the threshold, whole
    // number (for example 80). Placeholders: {harness}, {level}.
    threshold_title: "{harness}: {level}% del límite usado",
    // Body of the threshold alert. pct = the current used percentage, whole number; reset =
    // reset_in or reset_unknown. Placeholders: {window}, {pct}, {reset}.
    threshold_body: "{window} ya usó {pct}%. {reset}",
    // Notification title when the current pace would exhaust the window before it renews.
    // Placeholders: {harness}.
    pace_title: "{harness}: la cuota se agotará antes de renovarse",
    // Body of the pace alert. span = how long before the reset the quota is projected to run
    // out (see span_*). Placeholders: {window}, {span}.
    pace_body: "{window}: al ritmo actual se agota unos {span} antes de restablecerse.",
    // Body when a window reset by the clock with no new reading (the title is
    // window_reset_title). Placeholders: {window}.
    window_lapsed_body: "{window} se restableció y vuelve a tener cuota disponible.",
    // Notification title when the settings file was unreadable and was set aside. No
    // placeholders.
    settings_recovered_title: "Configuración restaurada",
    // Body of that notification. file_name = the name of the saved copy (a file name, no
    // folder). Placeholders: {file_name}.
    settings_recovered_body: "El archivo de configuración estaba ilegible o no era válido. Se conservó una copia junto a él ({file_name}) y se aplicaron los valores predeterminados.",
    // Name of the 5-hour window, the start of a sentence in notifications (use the grammatical
    // form that fits there). No placeholders.
    window_five_hour: "La ventana de 5 horas",
    // Name of the weekly window, as above. No placeholders.
    window_weekly: "La ventana semanal",
    // Name of the monthly window, as above. No placeholders.
    window_monthly: "La ventana mensual",
    // Name of the daily window, as above. No placeholders.
    window_daily: "La ventana diaria",
    // Name of a spending budget, as above. No placeholders.
    window_budget: "El presupuesto",
    // Name of a window of unknown kind, as above. No placeholders.
    window_other: "La ventana",
    // A window name followed by the source's own label for it. name = one of the window_*
    // entries; detail = text from the service (kept as is). Placeholders: {name}, {detail}.
    window_with_detail: "{name} ({detail})",
    // A duration shorter than one minute, as a whole phrase (notifications). No placeholders.
    span_less_than_minute: "menos de 1 min",
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
    tip_no_data: "{name}: sin datos",
    // Short window name in the tray tooltip: the 5-hour window. No placeholders.
    tip_window_five_hour: "5 horas",
    // Short window name in the tray tooltip: the weekly window. No placeholders.
    tip_window_weekly: "Semana",
    // Short window name in the tray tooltip: the monthly window. No placeholders.
    tip_window_monthly: "Mes",
    // Short window name in the tray tooltip: the daily window. No placeholders.
    tip_window_daily: "Día",
    // Short window name in the tray tooltip: a spending budget. No placeholders.
    tip_window_budget: "Presupuesto",
    // Short window name in the tray tooltip: unknown kind. No placeholders.
    tip_window_other: "Ventana",
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
    tip_out: "agotado",
    // Tooltip state: used share. pct = a percentage with its sign, such as '28%' or '<1%'.
    // Placeholders: {pct}.
    tip_used: "{pct} usado",
    // Tooltip state: available share. pct as above. Placeholders: {pct}.
    tip_available: "{pct} disponible",
    // Tooltip countdown to the next reset. duration = a tooltip duration (see duration_*), or
    // '1:53' in colon style. Placeholders: {duration}.
    tip_resets_in: "se restablece en {duration}",
    // Tooltip countdown when the window is out: when quota returns. Placeholders: {duration}.
    tip_back_in: "vuelve en {duration}",
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
    decimal_separator: ",",
    // Tray menu entry (right click) that opens the panel. No placeholders.
    tray_open: "Abrir",
    // Tray menu entry that refreshes every source now. No placeholders.
    tray_refresh: "Actualizar ahora",
    // Tray menu entry that quits the app. No placeholders.
    tray_quit: "Salir",
    // Notification body when turning on 'start with Windows' failed. The name in quotes must
    // match the setting's label in the UI. No placeholders.
    autostart_on_failed: "No se pudo activar «Iniciar con Windows».",
    // Notification body when turning off 'start with Windows' failed. No placeholders.
    autostart_off_failed: "No se pudo desactivar «Iniciar con Windows».",
    // Notification title when the app could not start (it closes right after). No placeholders.
    startup_title: "AI Session Monitor no pudo iniciarse",
    // Startup failure reason: the user's profile folders were not found. No placeholders.
    startup_paths: "No se encontraron las carpetas de tu perfil",
    // Startup failure reason: the data folder could not be created. No placeholders.
    startup_data_dir: "No se pudo crear la carpeta de datos de la aplicación",
    // Startup failure reason: the panel window was not created. No placeholders.
    startup_window: "No se pudo crear la ventana del panel",
    // Startup failure reason: the tray icon could not be created. No placeholders.
    startup_tray: "No se pudo crear el icono de la bandeja",
    // Startup failure reason: the settings file is broken and could not be set aside. No
    // placeholders.
    startup_settings_unreadable: "El archivo de configuración es ilegible y no se pudo apartar para empezar de nuevo",
    // Startup failure reason: the monitoring service refused to start. No placeholders.
    startup_service: "No se pudo iniciar el servicio de monitoreo",
    // Body of the startup failure notification. what = one of the startup_* reasons; code = a
    // technical code in English (kept as is). Placeholders: {what}, {code}.
    startup_body: "{what} (código: {code}). La aplicación se cerrará.",
};
