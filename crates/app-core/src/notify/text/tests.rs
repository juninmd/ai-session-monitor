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
        assert_eq!(span(messages(Language::PtBr), secs), expected, "{secs} s");
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
fn window(kind: WindowKind, label: Option<&str>, resets_at: Option<i64>) -> UsageWindow {
    let mut window = UsageWindow::new("w", kind);
    window.label = label.map(str::to_owned);
    window.resets_at = resets_at;
    window
}

#[test]
fn english_reads_naturally() {
    let en = Language::En;
    let weekly = window(WindowKind::Weekly, None, Some(113 * 60));
    let out = window_out(en, HarnessId::Codex, &weekly, 0);
    assert_eq!(out.title, "Codex: limit reached");
    assert_eq!(out.body, "Weekly window ran out. Resets in 1 h 53 min.");

    let budget = window(WindowKind::Budget, Some("team-a"), None);
    let out = window_out(en, HarnessId::Litellm, &budget, 0);
    assert_eq!(out.body, "Budget (team-a) ran out. The reset time was not reported.");

    let five = window(WindowKind::FiveHour, None, Some(5 * 3600));
    let back = window_reset(en, HarnessId::Claude, &five, 88.0, false, 0);
    assert_eq!(back.title, "Claude Code: quota available");
    assert_eq!(back.body, "5-hour window has quota available again (88% free). Next reset in 5 h.");
    let back = window_reset(en, HarnessId::Claude, &five, 88.0, true, 0);
    assert!(back.body.contains("(12% used)"), "{}", back.body);

    let hit = threshold(en, HarnessId::Claude, &five, 80, 82.4, 0);
    assert_eq!(hit.title, "Claude Code: 80% of the limit used");
    assert_eq!(hit.body, "5-hour window has used 82%. Resets in 5 h.");

    let risk = pace_risk(en, HarnessId::Claude, &five, 45.0);
    assert_eq!(risk.title, "Claude Code: quota will run out before it renews");
    assert_eq!(
        risk.body,
        "5-hour window: at the current pace it runs out about 45 min before it resets."
    );

    let lapsed = window_lapsed(en, HarnessId::Claude, &five);
    assert_eq!(lapsed.body, "5-hour window reset and has quota available again.");
    let recovered = settings_recovered(en, "settings.corrupt-1.json");
    assert_eq!(recovered.title, "Settings restored");
    assert!(recovered.body.contains("(settings.corrupt-1.json)"));
}

#[test]
fn portuguese_titles_and_spans_are_unchanged() {
    let pt = Language::PtBr;
    let five = window(WindowKind::FiveHour, None, Some(4 * 86_400 + 11 * 3600));
    assert_eq!(window_out(pt, HarnessId::Claude, &five, 0).title, "Claude Code: limite esgotado");
    assert_eq!(
        window_out(pt, HarnessId::Claude, &five, 0).body,
        "A janela de 5 horas esgotou. Reinicia em 4 d 11 h."
    );
    assert_eq!(
        threshold(pt, HarnessId::Zai, &five, 90, 91.0, 0).title,
        "z.ai: 90% do limite usado"
    );
    assert_eq!(
        pace_risk(pt, HarnessId::Claude, &five, 113.0).body,
        "No ritmo atual, A janela de 5 horas esgota cerca de 1 h 53 min antes de reiniciar."
    );
    assert_eq!(
        pace_risk(pt, HarnessId::Claude, &five, 1.0).title,
        "Claude Code: a cota deve acabar antes de renovar"
    );
    let recovered = settings_recovered(pt, "a.json");
    assert_eq!(recovered.title, "Configurações restauradas");
    assert_eq!(
        recovered.body,
        "O arquivo de configurações estava ilegível ou inválido. Uma cópia foi guardada ao \
         lado dele (a.json) e os valores padrão foram aplicados."
    );
}

#[test]
fn a_label_cannot_inject_a_placeholder() {
    let hostile = window(WindowKind::Weekly, Some("{reset}"), Some(60));
    let out = window_out(Language::En, HarnessId::Claude, &hostile, 0);
    assert_eq!(out.body, "Weekly window ({reset}) ran out. Resets in 1 min.");
}

#[test]
fn every_language_produces_complete_notifications() {
    use usage_model::settings::Language::*;
    let five = window(WindowKind::Weekly, Some("x"), Some(7200));
    for language in [En, ZhCn, Hi, Es, Fr, Ar, Bn, PtBr, Ru, Ur] {
        let notes = [
            window_out(language, HarnessId::Claude, &five, 0),
            window_reset(language, HarnessId::Claude, &five, 50.0, false, 0),
            threshold(language, HarnessId::Claude, &five, 80, 81.0, 0),
            pace_risk(language, HarnessId::Claude, &five, 30.0),
            window_lapsed(language, HarnessId::Claude, &five),
            settings_recovered(language, "s.json"),
        ];
        for note in notes {
            let shown = format!("{} {}", note.title, note.body);
            assert!(!shown.contains(['{', '}']), "{language:?}: {shown}");
        }
    }
}
