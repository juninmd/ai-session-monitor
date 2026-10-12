//! pt-BR wording of the notifications. Window labels come from remote services, so they are
//! stripped of control characters and cut short before they reach a desktop toast.

use usage_model::{HarnessId, UsageWindow, WindowKind};

use super::{Notification, NotificationKind};

const MAX_LABEL_CHARS: usize = 40;

fn harness_name(id: HarnessId) -> &'static str {
    match id {
        HarnessId::Claude => "Claude Code",
        HarnessId::Codex => "Codex",
        HarnessId::Antigravity => "Antigravity",
        HarnessId::Opencode => "OpenCode",
        HarnessId::Kimi => "Kimi Code",
        HarnessId::Zai => "z.ai",
        HarnessId::Openrouter => "OpenRouter",
        HarnessId::Deepseek => "DeepSeek",
        HarnessId::Deepinfra => "DeepInfra",
        HarnessId::Moonshot => "Moonshot",
        HarnessId::Novita => "Novita",
        HarnessId::Ollama => "Ollama Cloud",
        HarnessId::Minimax => "MiniMax",
        HarnessId::Litellm => "LiteLLM",
    }
}

pub(super) fn window_out(harness: HarnessId, window: &UsageWindow, now_secs: i64) -> Notification {
    let (article, noun) = subject(window);
    let reset = match window.resets_at {
        Some(at) => format!("Reinicia em {}.", span(at - now_secs)),
        None => "O horário de reinício não foi informado.".to_owned(),
    };
    Notification {
        kind: NotificationKind::WindowOut,
        harness,
        window_id: window.id.clone(),
        title: format!("{}: limite esgotado", harness_name(harness)),
        body: format!("{article} {noun} esgotou. {reset}"),
    }
}

pub(super) fn window_reset(
    harness: HarnessId,
    window: &UsageWindow,
    available_pct: f64,
    show_used: bool,
    now_secs: i64,
) -> Notification {
    let (article, noun) = subject(window);
    let next = window
        .resets_at
        .map(|at| format!(" Próximo reinício em {}.", span(at - now_secs)))
        .unwrap_or_default();
    let free = available_pct.round().clamp(0.0, 100.0) as u8;
    let share = if show_used { format!("{}% usado", 100 - free) } else { format!("{free}% livre") };
    Notification {
        kind: NotificationKind::WindowReset,
        harness,
        window_id: window.id.clone(),
        title: format!("{}: limite disponível", harness_name(harness)),
        body: format!("{article} {noun} voltou a ter limite disponível ({share}).{next}"),
    }
}

/// A usage threshold was crossed; `level` is the threshold, `used` the reading that crossed it.
pub(super) fn threshold(
    harness: HarnessId,
    window: &UsageWindow,
    level: u8,
    used: f64,
    now_secs: i64,
) -> Notification {
    let (article, noun) = subject(window);
    let reset = match window.resets_at {
        Some(at) => format!("Reinicia em {}.", span(at - now_secs)),
        None => "O horário de reinício não foi informado.".to_owned(),
    };
    let shown = used.round().clamp(0.0, 999.0) as u16;
    Notification {
        kind: NotificationKind::ThresholdCrossed,
        harness,
        window_id: window.id.clone(),
        title: format!("{}: {level}% do limite usado", harness_name(harness)),
        body: format!("{article} {noun} já usou {shown}%. {reset}"),
    }
}

/// The reset came by the clock with no reading behind it, so the text claims no free percentage.
pub(super) fn window_lapsed(harness: HarnessId, window: &UsageWindow) -> Notification {
    let (article, noun) = subject(window);
    Notification {
        kind: NotificationKind::WindowReset,
        harness,
        window_id: window.id.clone(),
        title: format!("{}: limite disponível", harness_name(harness)),
        body: format!("{article} {noun} reiniciou e voltou a ter limite disponível."),
    }
}

/// Shown when the settings file had to be set aside; `file_name` is the copy's name, no path.
pub(crate) fn settings_recovered(file_name: &str) -> Notification {
    Notification {
        kind: NotificationKind::SettingsRecovered,
        harness: HarnessId::Claude,
        window_id: String::new(),
        title: "Configurações restauradas".to_owned(),
        body: format!(
            "O arquivo de configurações estava ilegível ou inválido. Uma cópia foi guardada \
             ao lado dele ({file_name}) e os valores padrão foram aplicados."
        ),
    }
}

/// Article and noun phrase for the window, for example `("A", "janela semanal")`.
fn subject(window: &UsageWindow) -> (&'static str, String) {
    let (article, base) = match window.kind {
        WindowKind::FiveHour => ("A", "janela de 5 horas"),
        WindowKind::Weekly => ("A", "janela semanal"),
        WindowKind::Monthly => ("A", "janela mensal"),
        WindowKind::Daily => ("A", "janela diária"),
        WindowKind::Budget => ("O", "orçamento"),
        WindowKind::Other => ("A", "janela"),
    };
    let detail = window.label.as_deref().or(window.group.as_deref()).map(clean);
    match detail.filter(|detail| !detail.is_empty()) {
        Some(detail) => (article, format!("{base} ({detail})")),
        None => (article, base.to_owned()),
    }
}

fn clean(raw: &str) -> String {
    let printable: String = raw.chars().filter(|c| !c.is_control()).collect();
    let printable = printable.trim();
    if printable.chars().count() > MAX_LABEL_CHARS {
        let cut: String = printable.chars().take(MAX_LABEL_CHARS).collect();
        format!("{}…", cut.trim_end())
    } else {
        printable.to_owned()
    }
}

/// `45 min`, `1 h 53 min`, `2 h`, `4 d 11 h`; rounded up to the minute so it never reads "0 min".
fn span(secs: i64) -> String {
    let minutes = secs.max(0).saturating_add(59) / 60;
    match minutes {
        0 => "menos de 1 min".to_owned(),
        1..=59 => format!("{minutes} min"),
        60..=1439 => match minutes % 60 {
            0 => format!("{} h", minutes / 60),
            rest => format!("{} h {rest} min", minutes / 60),
        },
        _ => match (minutes / 60) % 24 {
            0 => format!("{} d", minutes / 1440),
            hours => format!("{} d {hours} h", minutes / 1440),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spans_read_naturally_and_round_up() {
        let cases = [
            (-5, "menos de 1 min"),
            (0, "menos de 1 min"),
            (1, "1 min"),
            (60, "1 min"),
            (61, "2 min"),
            (45 * 60, "45 min"),
            (59 * 60 + 1, "1 h"),
            (60 * 60, "1 h"),
            (113 * 60, "1 h 53 min"),
            (23 * 3600 + 59 * 60, "23 h 59 min"),
            (24 * 3600, "1 d"),
            (4 * 86_400 + 11 * 3600, "4 d 11 h"),
            (i64::MAX, "106751991167300 d 15 h"),
        ];
        for (secs, expected) in cases {
            assert_eq!(span(secs), expected, "{secs} s");
        }
    }

    #[test]
    fn labels_lose_control_characters_and_are_cut() {
        assert_eq!(clean("  team-a \n\t"), "team-a");
        assert_eq!(clean("a\u{7}b\u{1b}[31mc"), "ab[31mc");
        let long = "x".repeat(100);
        let cleaned = clean(&long);
        assert_eq!(cleaned.chars().count(), MAX_LABEL_CHARS + 1);
        assert!(cleaned.ends_with('…'));
        assert_eq!(clean("ação"), "ação");
    }
}
