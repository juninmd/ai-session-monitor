//! Simplified Chinese wording of everything the Rust side shows: desktop notifications, the tray tooltip,
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
    window_out_title: "{harness}：额度已用完",
    // Notification body when a window ran out. window = the window description (see the
    // window_* entries, with the optional label); reset = reset_in or reset_unknown. The window
    // starts the sentence. Placeholders: {window}, {reset}.
    window_out_body: "{window}已用完。{reset}",
    // Sentence inserted as reset in the out and threshold bodies. span = a duration such as '1
    // h 53 min' (see span_*). Placeholders: {span}.
    reset_in: "将在 {span}后重置。",
    // Sentence inserted as reset when the source did not report a reset time. No placeholders.
    reset_unknown: "来源未提供重置时间。",
    // Notification title when a window that was out has quota again (also the clock-driven
    // reset). harness = product name. Placeholders: {harness}.
    window_reset_title: "{harness}：额度已恢复",
    // Body when the window has quota again and the next reset is unknown. share = share_free or
    // share_used. Placeholders: {window}, {share}.
    window_reset_body: "{window}的额度已恢复（{share}）。",
    // Same as window_reset_body, plus the next reset. span = a duration (see span_*).
    // Placeholders: {window}, {share}, {span}.
    window_reset_body_next: "{window}的额度已恢复（{share}）。下次重置在 {span}后。",
    // The available share inside window_reset_body. pct = whole number 0-100, without the
    // percent sign. Placeholders: {pct}.
    share_free: "剩余 {pct}%",
    // The used share inside window_reset_body, when the user chose to show used quota. pct =
    // whole number 0-100. Placeholders: {pct}.
    share_used: "已用 {pct}%",
    // Notification title when usage crossed an alert threshold. level = the threshold, whole
    // number (for example 80). Placeholders: {harness}, {level}.
    threshold_title: "{harness}：已用 {level}% 的额度",
    // Body of the threshold alert. pct = the current used percentage, whole number; reset =
    // reset_in or reset_unknown. Placeholders: {window}, {pct}, {reset}.
    threshold_body: "{window}已用 {pct}%。{reset}",
    // Notification title when the current pace would exhaust the window before it renews.
    // Placeholders: {harness}.
    pace_title: "{harness}：额度将在重置前用完",
    // Body of the pace alert. span = how long before the reset the quota is projected to run
    // out (see span_*). Placeholders: {window}, {span}.
    pace_body: "{window}：按当前速度，将在重置前约 {span}用完。",
    // Body when a window reset by the clock with no new reading (the title is
    // window_reset_title). Placeholders: {window}.
    window_lapsed_body: "{window}已重置，额度已恢复。",
    // Notification title when the settings file was unreadable and was set aside. No
    // placeholders.
    settings_recovered_title: "设置已恢复",
    // Body of that notification. file_name = the name of the saved copy (a file name, no
    // folder). Placeholders: {file_name}.
    settings_recovered_body: "设置文件无法读取或无效。已在其旁边保留一份副本（{file_name}），并应用了默认设置。",
    // Name of the 5-hour window, the start of a sentence in notifications (use the grammatical
    // form that fits there). No placeholders.
    window_five_hour: "5 小时窗口",
    // Name of the weekly window, as above. No placeholders.
    window_weekly: "每周窗口",
    // Name of the monthly window, as above. No placeholders.
    window_monthly: "每月窗口",
    // Name of the daily window, as above. No placeholders.
    window_daily: "每日窗口",
    // Name of a spending budget, as above. No placeholders.
    window_budget: "预算",
    // Name of a window of unknown kind, as above. No placeholders.
    window_other: "窗口",
    // A window name followed by the source's own label for it. name = one of the window_*
    // entries; detail = text from the service (kept as is). Placeholders: {name}, {detail}.
    window_with_detail: "{name}（{detail}）",
    // A duration shorter than one minute, as a whole phrase (notifications). No placeholders.
    span_less_than_minute: "不到 1 分钟",
    // A duration under one hour, in notifications. min = whole number 1-59. Placeholders:
    // {min}.
    span_minutes: "{min} 分钟",
    // A whole number of hours under one day, in notifications. Placeholders: {hours}.
    span_hours: "{hours} 小时",
    // Hours and minutes under one day, in notifications. hours = 1-23; min = 1-59.
    // Placeholders: {hours}, {min}.
    span_hours_minutes: "{hours} 小时 {min} 分钟",
    // A whole number of days, in notifications. Placeholders: {days}.
    span_days: "{days} 天",
    // Days and hours, in notifications. hours = 1-23. Placeholders: {days}, {hours}.
    span_days_hours: "{days} 天 {hours} 小时",
    // Tray tooltip when there is no reading yet. name = the product name or 'AI Session
    // Monitor'. Windows cuts the tooltip at 127 characters: keep it short. Placeholders:
    // {name}.
    tip_no_data: "{name}：暂无数据",
    // Short window name in the tray tooltip: the 5-hour window. No placeholders.
    tip_window_five_hour: "5 小时",
    // Short window name in the tray tooltip: the weekly window. No placeholders.
    tip_window_weekly: "每周",
    // Short window name in the tray tooltip: the monthly window. No placeholders.
    tip_window_monthly: "每月",
    // Short window name in the tray tooltip: the daily window. No placeholders.
    tip_window_daily: "每日",
    // Short window name in the tray tooltip: a spending budget. No placeholders.
    tip_window_budget: "预算",
    // Short window name in the tray tooltip: unknown kind. No placeholders.
    tip_window_other: "窗口",
    // Start of the tooltip line. window = one of the tip_window_* entries. Placeholders:
    // {harness}, {window}.
    tip_head: "{harness} · {window}",
    // Tooltip line without a countdown. head = tip_head (or only the product name when space is
    // short); state = tip_out, tip_used or tip_available (or only a percentage). Placeholders:
    // {head}, {state}.
    tip_line: "{head}：{state}",
    // Tooltip line with a countdown. when = tip_resets_in or tip_back_in. Placeholders: {head},
    // {state}, {when}.
    tip_line_when: "{head}：{state} · {when}",
    // Tooltip state: the window has no quota left. No placeholders.
    tip_out: "已用完",
    // Tooltip state: used share. pct = a percentage with its sign, such as '28%' or '<1%'.
    // Placeholders: {pct}.
    tip_used: "已用 {pct}",
    // Tooltip state: available share. pct as above. Placeholders: {pct}.
    tip_available: "剩余 {pct}",
    // Tooltip countdown to the next reset. duration = a tooltip duration (see duration_*), or
    // '1:53' in colon style. Placeholders: {duration}.
    tip_resets_in: "{duration}后重置",
    // Tooltip countdown when the window is out: when quota returns. Placeholders: {duration}.
    tip_back_in: "{duration}后恢复",
    // Tooltip duration under one hour. min = whole number. Placeholders: {min}.
    duration_minutes: "{min} 分钟",
    // Tooltip duration in hours and minutes; min is always two digits ('1h 05m'). Placeholders:
    // {hours}, {min}.
    duration_hours_minutes: "{hours}小时{min}分",
    // Tooltip duration in days and hours, for long windows. Placeholders: {days}, {hours}.
    duration_days_hours: "{days}天{hours}小时",
    // Tooltip duration in decimal hours. value = a number with one decimal, using
    // decimal_separator. Placeholders: {value}.
    duration_decimal_hours: "{value} 小时",
    // The decimal separator of this language, used in duration_decimal_hours values (a single
    // character). No placeholders.
    decimal_separator: ".",
    // Tray menu entry (right click) that opens the panel. No placeholders.
    tray_open: "打开",
    // Tray menu entry that refreshes every source now. No placeholders.
    tray_refresh: "立即刷新",
    // Tray menu entry that quits the app. No placeholders.
    tray_quit: "退出",
    // Notification body when turning on 'start with Windows' failed. The name in quotes must
    // match the setting's label in the UI. No placeholders.
    autostart_on_failed: "无法开启“随 Windows 启动”。",
    // Notification body when turning off 'start with Windows' failed. No placeholders.
    autostart_off_failed: "无法关闭“随 Windows 启动”。",
    // Notification title when the app could not start (it closes right after). No placeholders.
    startup_title: "AI Session Monitor 启动失败",
    // Startup failure reason: the user's profile folders were not found. No placeholders.
    startup_paths: "找不到你的用户配置文件夹",
    // Startup failure reason: the data folder could not be created. No placeholders.
    startup_data_dir: "无法创建应用的数据文件夹",
    // Startup failure reason: the panel window was not created. No placeholders.
    startup_window: "无法创建面板窗口",
    // Startup failure reason: the tray icon could not be created. No placeholders.
    startup_tray: "无法创建托盘图标",
    // Startup failure reason: the settings file is broken and could not be set aside. No
    // placeholders.
    startup_settings_unreadable: "设置文件无法读取，也无法移开以重新开始",
    // Startup failure reason: the monitoring service refused to start. No placeholders.
    startup_service: "无法启动监控服务",
    // Body of the startup failure notification. what = one of the startup_* reasons; code = a
    // technical code in English (kept as is). Placeholders: {what}, {code}.
    startup_body: "{what}（代码：{code}）。应用将关闭。",
};
