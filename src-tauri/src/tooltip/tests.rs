use super::*;

const NOW: i64 = 1_800_000_000;

fn reading(
    harness: HarnessId,
    kind: WindowKind,
    window_minutes: Option<u32>,
    resets_in_min: Option<i64>,
    pct: f64,
    status: Status,
) -> TrayReading {
    TrayReading {
        harness,
        kind,
        window_minutes,
        resets_at: resets_in_min.map(|m| NOW + m * 60),
        available_pct: pct,
        status,
    }
}

fn five_hour(pct: f64, resets_in_min: Option<i64>, status: Status) -> TrayReading {
    reading(HarnessId::Claude, WindowKind::FiveHour, Some(300), resets_in_min, pct, status)
}

const LANGUAGES: [Language; 10] = [
    Language::En,
    Language::ZhCn,
    Language::Hi,
    Language::Es,
    Language::Fr,
    Language::Ar,
    Language::Bn,
    Language::PtBr,
    Language::Ru,
    Language::Ur,
];

fn english(time_format: TimeFormat, week_format: WeekFormat) -> GeneralSettings {
    GeneralSettings { language: Language::En, ..general(time_format, week_format) }
}

fn general(time_format: TimeFormat, week_format: WeekFormat) -> GeneralSettings {
    GeneralSettings {
        time_format,
        week_format,
        language: Language::PtBr,
        ..GeneralSettings::default()
    }
}

fn default_text(reading: &TrayReading) -> String {
    text(Some(reading), None, &general(TimeFormat::Hm, WeekFormat::Hours), NOW)
}

#[test]
fn names_the_harness_window_percentage_and_time_to_reset() {
    let got = default_text(&five_hour(72.4, Some(113), Status::Ok));
    assert_eq!(got, "Claude Code · 5 horas: 72% disponível · reseta em 1h 53m");
}

#[test]
fn show_used_flips_the_wording_and_the_number() {
    let on =
        GeneralSettings { show_used: true, language: Language::PtBr, ..GeneralSettings::default() };
    let r = five_hour(72.4, Some(113), Status::Ok);
    assert_eq!(
        text(Some(&r), None, &on, NOW),
        "Claude Code · 5 horas: 28% usado · reseta em 1h 53m"
    );
    let sliver = five_hour(99.7, None, Status::Ok);
    assert_eq!(text(Some(&sliver), None, &on, NOW), "Claude Code · 5 horas: <1% usado");
    let out = five_hour(0.0, Some(10), Status::Out);
    assert!(text(Some(&out), None, &on, NOW).contains("esgotado"));
}

#[test]
fn honours_the_time_format_setting() {
    let r = five_hour(72.0, Some(113), Status::Ok);
    let colon = text(Some(&r), None, &general(TimeFormat::Colon, WeekFormat::Hours), NOW);
    let decimal = text(Some(&r), None, &general(TimeFormat::Decimal, WeekFormat::Hours), NOW);
    assert!(colon.ends_with("reseta em 1:53"), "{colon}");
    assert!(decimal.ends_with("reseta em 1,9 h"), "{decimal}");
}

#[test]
fn honours_the_week_format_for_multi_day_windows() {
    let r =
        reading(HarnessId::Codex, WindowKind::Weekly, Some(10_080), Some(6432), 40.0, Status::Ok);
    let hours = text(Some(&r), None, &general(TimeFormat::Hm, WeekFormat::Hours), NOW);
    let days = text(Some(&r), None, &general(TimeFormat::Hm, WeekFormat::DaysHours), NOW);
    assert_eq!(hours, "Codex · Semana: 40% disponível · reseta em 107h 12m");
    assert_eq!(days, "Codex · Semana: 40% disponível · reseta em 4d 11h");
}

#[test]
fn an_exhausted_window_says_when_it_comes_back() {
    let got = default_text(&five_hour(0.0, Some(113), Status::Out));
    assert_eq!(got, "Claude Code · 5 horas: esgotado · volta em 1h 53m");
}

#[test]
fn a_window_without_a_reset_time_omits_the_countdown() {
    let got = default_text(&five_hour(72.0, None, Status::Ok));
    assert_eq!(got, "Claude Code · 5 horas: 72% disponível");
}

#[test]
fn a_reset_in_the_past_counts_down_zero_instead_of_going_negative() {
    let got = default_text(&five_hour(72.0, Some(-5), Status::Ok));
    assert!(got.ends_with("reseta em 0 min"), "{got}");
}

#[test]
fn a_sliver_is_not_rounded_to_zero() {
    assert!(default_text(&five_hour(0.4, Some(10), Status::Crit)).contains("<1% disponível"));
}

#[test]
fn no_reading_reads_sem_dados() {
    let g = general(TimeFormat::Hm, WeekFormat::Hours);
    assert_eq!(text(None, None, &g, NOW), "AI Session Monitor: sem dados");
    assert_eq!(text(None, Some(HarnessId::Codex), &g, NOW), "Codex: sem dados");
}

#[test]
fn every_combination_fits_the_windows_limit() {
    let kinds = [
        WindowKind::FiveHour,
        WindowKind::Weekly,
        WindowKind::Monthly,
        WindowKind::Daily,
        WindowKind::Budget,
        WindowKind::Other,
    ];
    let times = [TimeFormat::Hm, TimeFormat::Colon, TimeFormat::Decimal];
    let weeks = [WeekFormat::Hours, WeekFormat::DaysHours];
    for language in LANGUAGES {
        for harness in HarnessId::ALL {
            for kind in kinds {
                for status in [Status::Ok, Status::Out] {
                    for (time, week) in times.into_iter().flat_map(|t| weeks.map(|w| (t, w))) {
                        let r = reading(harness, kind, None, Some(10_000_000), 99.6, status);
                        let general = GeneralSettings { language, ..general(time, week) };
                        let got = text(Some(&r), None, &general, NOW);
                        assert!(utf16_len(&got) <= MAX_UTF16, "{got} ({})", utf16_len(&got));
                    }
                }
            }
        }
    }
}

#[test]
fn fit_prefers_the_first_candidate_that_fits() {
    let long = "x".repeat(MAX_UTF16 + 1);
    let picked = fit(&[long, "curto".to_owned(), "menor".to_owned()]);
    assert_eq!(picked, "curto");
}

#[test]
fn fit_cuts_an_oversized_text_with_an_ellipsis_without_splitting_a_pair() {
    let picked = fit(&["😀".repeat(100)]);
    assert!(utf16_len(&picked) <= MAX_UTF16, "{}", utf16_len(&picked));
    assert!(picked.ends_with('…'));
    assert!(picked.chars().filter(|c| *c == '😀').count() >= 60);
}

#[test]
fn fit_of_nothing_is_just_an_ellipsis() {
    assert_eq!(fit(&[]), "…");
}

#[test]
fn utf16_len_counts_code_units_not_characters() {
    assert_eq!(utf16_len("Mês"), 3);
    assert_eq!(utf16_len("·"), 1);
    assert_eq!(utf16_len("😀"), 2);
}

#[test]
fn durations_follow_the_flyout_formatting() {
    use TimeFormat::{Colon, Decimal, Hm};
    use WeekFormat::{DaysHours, Hours};
    let cases = [
        (113.0, Hm, Hours, false, "1h 53m"),
        (60.0, Hm, Hours, false, "1h 00m"),
        (45.0, Hm, Hours, false, "45 min"),
        (0.0, Hm, Hours, false, "0 min"),
        (112.5, Hm, Hours, false, "1h 53m"),
        (113.0, Colon, Hours, false, "1:53"),
        (5.0, Colon, Hours, false, "0:05"),
        (113.0, Decimal, Hours, false, "1,9 h"),
        // x.25 h is an exact tie: the flyout (toFixed) rounds it up, so the tooltip must too.
        (15.0, Decimal, Hours, false, "0,3 h"),
        (75.0, Decimal, Hours, false, "1,3 h"),
        (135.0, Decimal, Hours, false, "2,3 h"),
        // Neighbours of the tie that already agreed and must not move.
        (45.0, Decimal, Hours, false, "0,8 h"),
        (9.0, Decimal, Hours, false, "0,1 h"),
        // Saturates at u32::MAX minutes, itself a :15 tie: the nudge must not overflow.
        (f64::MAX, Decimal, Hours, false, "71582788,3 h"),
        (6432.0, Hm, Hours, true, "107h 12m"),
        (6432.0, Colon, Hours, true, "107:12"),
        (6432.0, Decimal, Hours, true, "107,2 h"),
        (6432.0, Hm, DaysHours, true, "4d 11h"),
        (12_000.0, Hm, Hours, true, "8d 8h"),
        (300.0, Hm, DaysHours, true, "5h 00m"),
        (3000.0, Hm, DaysHours, false, "50h 00m"),
        (-20.0, Hm, Hours, false, "0 min"),
        (f64::NAN, Hm, Hours, false, "0 min"),
    ];
    for (minutes, time, week, long, expected) in cases {
        assert_eq!(
            format_duration(minutes, time, week, long, Language::PtBr),
            expected,
            "{minutes} {time:?}"
        );
    }
}

#[test]
fn the_percentage_rounds_like_the_flyout() {
    // The rows of `formatPercent` in src/i18n/format.test.ts, plus 72.6, which a floor shows as 72%.
    let cases = [
        (0.0, "0%"),
        (0.4, "<1%"),
        (1.0, "1%"),
        (52.4, "52%"),
        (52.5, "53%"),
        (72.6, "73%"),
        (100.0, "100%"),
    ];
    for (pct, expected) in cases {
        assert_eq!(format_percent(pct), expected, "{pct}");
    }
}

#[test]
fn english_reads_naturally() {
    let hm = english(TimeFormat::Hm, WeekFormat::Hours);
    let r = five_hour(72.4, Some(113), Status::Ok);
    assert_eq!(
        text(Some(&r), None, &hm, NOW),
        "Claude Code · 5 hours: 72% available · resets in 1h 53m"
    );
    let used = GeneralSettings { show_used: true, ..hm.clone() };
    assert_eq!(
        text(Some(&r), None, &used, NOW),
        "Claude Code · 5 hours: 28% used · resets in 1h 53m"
    );
    let out = five_hour(0.0, Some(113), Status::Out);
    assert_eq!(text(Some(&out), None, &hm, NOW), "Claude Code · 5 hours: out · back in 1h 53m");
    let week =
        reading(HarnessId::Codex, WindowKind::Weekly, Some(10_080), Some(6432), 40.0, Status::Ok);
    assert_eq!(
        text(Some(&week), None, &hm, NOW),
        "Codex · Week: 40% available · resets in 107h 12m"
    );
    assert_eq!(text(None, None, &hm, NOW), "AI Session Monitor: no data");
    assert_eq!(text(None, Some(HarnessId::Codex), &hm, NOW), "Codex: no data");
}

#[test]
fn the_decimal_separator_follows_the_language() {
    use TimeFormat::Decimal;
    assert_eq!(format_duration(113.0, Decimal, WeekFormat::Hours, false, Language::En), "1.9 h");
    assert_eq!(format_duration(113.0, Decimal, WeekFormat::Hours, false, Language::PtBr), "1,9 h");
}
