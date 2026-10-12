use std::path::Path;

use super::*;

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|arg| (*arg).to_owned()).collect()
}

fn parse(list: &[&str]) -> ServerFlags {
    ServerFlags::parse(&args(list))
}

fn token(flags: &ServerFlags) -> Option<&str> {
    flags.csrf_token.as_ref().map(Secret::expose)
}

#[test]
fn reads_the_flags_of_a_server_started_the_way_the_app_starts_it() {
    let flags = parse(&[
        r"C:\Users\someone\AppData\Local\Programs\Antigravity\resources\app\extensions\antigravity\bin\language_server_windows_x64.exe",
        "--enable_lsp",
        "--csrf_token",
        "0f8f7c1e-aaaa-bbbb-cccc-000000000001",
        "--extension_server_port",
        "50123",
        "--extension_server_csrf_token",
        "0f8f7c1e-aaaa-bbbb-cccc-000000000002",
        "--https_server_port",
        "0",
        "--lsp_port",
        "50200",
        "--app_data_dir",
        "antigravity",
        "--random_port",
    ]);
    assert_eq!(token(&flags), Some("0f8f7c1e-aaaa-bbbb-cccc-000000000001"));
    assert_eq!(
        flags.extension_csrf_token.as_ref().map(Secret::expose),
        Some("0f8f7c1e-aaaa-bbbb-cccc-000000000002")
    );
    assert_eq!(flags.extension_port, Some(50123));
    assert_eq!(flags.lsp_port, Some(50200));
    // 0 means "pick any port": there is nothing to record.
    assert_eq!(flags.https_port, None);
    assert_eq!(flags.app_data_dir.as_deref(), Some("antigravity"));
    assert!(flags.has_usable_token());
}

#[test]
fn one_or_two_dashes_equals_or_space_and_hyphen_or_underscore_all_work() {
    let flags = parse(&[
        "-csrf_token=abc",
        "--https-server-port=4443",
        "--LSP_PORT",
        "7",
        "-extension-server-port=9",
    ]);
    assert_eq!(token(&flags), Some("abc"));
    assert_eq!(
        (flags.https_port, flags.lsp_port, flags.extension_port),
        (Some(4443), Some(7), Some(9))
    );
}

#[test]
fn a_value_with_spaces_or_quotes_is_one_argument_and_stays_whole() {
    let flags = parse(&["--app_data_dir", "antigravity ide data", "--csrf_token", "ab\"cd'ef"]);
    assert_eq!(flags.app_data_dir.as_deref(), Some("antigravity ide data"));
    assert_eq!(token(&flags), Some("ab\"cd'ef"));
    assert!(flags.has_usable_token(), "quotes are visible ASCII and fit a header");
}

#[test]
fn a_token_may_contain_equals_signs() {
    assert_eq!(token(&parse(&["--csrf_token=ab=cd=="])), Some("ab=cd=="));
    assert_eq!(token(&parse(&["--csrf_token", "ab=cd"])), Some("ab=cd"));
    assert_eq!(token(&parse(&["--csrf_token=="])), Some("="));
}

#[test]
fn a_unicode_token_is_read_but_cannot_travel_as_a_header() {
    let flags = parse(&["--csrf_token=tok-\u{e7}\u{e3}o-\u{1f511}"]);
    assert_eq!(token(&flags), Some("tok-\u{e7}\u{e3}o-\u{1f511}"));
    assert!(!flags.has_usable_token());
    assert_eq!(usable(flags.csrf_token.as_ref()), None);
}

#[test]
fn missing_flags_and_missing_values_leave_the_fields_empty() {
    assert_eq!(parse(&[]), ServerFlags::default());
    assert_eq!(parse(&["language_server.exe", "--enable_lsp"]), ServerFlags::default());
    assert_eq!(token(&parse(&["--csrf_token"])), None);
    assert_eq!(token(&parse(&["--csrf_token="])), None);
    assert_eq!(token(&parse(&["--csrf_token", ""])), None);
    // A following flag is not the value of the one before it.
    let flags = parse(&["--csrf_token", "--https_server_port", "443"]);
    assert_eq!((token(&flags), flags.https_port), (None, Some(443)));
    assert!(!parse(&[]).has_usable_token());
}

#[test]
fn bare_words_and_unknown_flags_are_ignored() {
    let flags =
        parse(&["csrf_token", "abc", "--unknown", "x", "--csrf_tokenx=y", "---csrf_token=z"]);
    assert_eq!(flags, ServerFlags::default());
}

#[test]
fn the_last_occurrence_wins_but_an_unusable_port_never_erases_a_good_one() {
    let flags = parse(&["--csrf_token", "first", "--csrf_token=second"]);
    assert_eq!(token(&flags), Some("second"));
    let flags = parse(&[
        "--https_server_port",
        "443",
        "--https_server_port",
        "0",
        "--https_server_port=nope",
    ]);
    assert_eq!(flags.https_port, Some(443));
    assert_eq!(
        parse(&["--https_server_port=8443", "--https_server_port=9443"]).https_port,
        Some(9443)
    );
}

#[test]
fn ports_are_1_to_65535() {
    let port = |text: &str| parse(&["--lsp_port", text]).lsp_port;
    assert_eq!(port("1"), Some(1));
    assert_eq!(port("65535"), Some(65535));
    assert_eq!(port(" 443 "), Some(443));
    for bad in ["0", "65536", "-1", "+", "4.5", "abc", "", "99999999999999999999"] {
        assert_eq!(port(bad), None, "{bad:?}");
    }
}

#[test]
fn only_the_first_128_arguments_are_read() {
    let mut long: Vec<String> = (0..100).map(|index| format!("--ignored{index}")).collect();
    long.extend(args(&["--csrf_token", "inside"]));
    assert_eq!(token(&ServerFlags::parse(&long)), Some("inside"));
    let mut long: Vec<String> = (0..MAX_ARGS).map(|index| format!("--ignored{index}")).collect();
    long.extend(args(&["--csrf_token", "beyond"]));
    assert_eq!(token(&ServerFlags::parse(&long)), None);
}

#[test]
fn debug_output_never_shows_a_token() {
    let flags = parse(&[
        "--csrf_token",
        "visible-secret-1",
        "--extension_server_csrf_token=visible-secret-2",
    ]);
    let shown = format!("{flags:?} {flags:#?}");
    assert!(!shown.contains("visible-secret"), "{shown}");
}

#[test]
fn a_token_must_fit_a_header_value() {
    let usable_text = |text: &str| usable(Some(&Secret::new(text))).is_some();
    assert!(usable_text("0f8f7c1e-aaaa-bbbb-cccc-000000000001"));
    assert!(usable_text("a=b+c/d_e~f"));
    assert!(usable_text(&"x".repeat(MAX_TOKEN_BYTES)));
    for bad in ["", " ", "has space", "tab\there", "line\nbreak", "nul\0", "del\u{7f}", "\u{e7}"] {
        assert!(!usable_text(bad), "{bad:?}");
    }
    assert!(!usable_text(&"x".repeat(MAX_TOKEN_BYTES + 1)));
    assert!(usable(None).is_none());
}

#[test]
fn the_app_data_dir_flag_decides_which_app_the_server_belongs_to() {
    let is = |list: &[&str]| {
        let flags = parse(list);
        // An install path that says Antigravity must not override an explicit other app.
        let exe = Path::new("C:/Programs/Antigravity/bin/language_server.exe");
        flags.is_antigravity(Some(exe), &args(list))
    };
    assert!(is(&["--app_data_dir", "antigravity"]));
    assert!(is(&["--app_data_dir", "antigravity-ide"]));
    assert!(is(&["--app_data_dir=Antigravity"]));
    assert!(!is(&["--app_data_dir", "windsurf"]));
    assert!(!is(&["--app_data_dir", "codeium"]));
    assert!(!is(&["--app_data_dir", "my-antigravity"]));
}

#[test]
fn without_the_flag_the_install_path_has_to_name_the_app() {
    let none = ServerFlags::default();
    let exe = |text: &str| none.is_antigravity(Some(Path::new(text)), &[]);
    assert!(exe(
        "C:/Users/x/AppData/Local/Programs/Antigravity/resources/app/extensions/antigravity/bin/language_server_windows_x64.exe"
    ));
    assert!(exe(
        "/Applications/Antigravity IDE.app/Contents/Resources/app/extensions/antigravity/bin/language_server_macos_arm"
    ));
    assert!(exe("/opt/antigravity-ide/bin/language_server_linux_x64"));
    assert!(!exe(
        "C:/Users/x/AppData/Local/Programs/Windsurf/resources/app/extensions/windsurf/bin/language_server_windows_x64.exe"
    ));
    // A user or a folder that merely mentions the name is not the app.
    assert!(!exe("C:/Users/antigravity_fan/Windsurf/bin/language_server.exe"));
    assert!(!exe("/home/me/not-antigravity/bin/language_server"));
    // The first argument stands in when the OS hides the executable path.
    let first = args(&["/opt/Antigravity/bin/language_server_linux_x64", "--csrf_token", "x"]);
    assert!(none.is_antigravity(None, &first));
    assert!(!none.is_antigravity(None, &args(&["/opt/other/bin/language_server_linux_x64"])));
    assert!(!none.is_antigravity(None, &[]));
}
