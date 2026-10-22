//! Finding the extension's token in fabricated `apps.json`/`hosts.json` files under a temp home.

use usage_model::{ErrorCode, ViewState};

use super::fake_server::{FakeServer, Reply};
use super::{APPS_TOKEN, HOSTS_TOKEN, copilot_dir, fixture, poll, put, settings};
use crate::local::{Reason, candidate_dirs, find_in, find_token};

fn dirs(home: &std::path::Path) -> Vec<std::path::PathBuf> {
    candidate_dirs(home, cfg!(windows))
}

#[test]
fn apps_json_wins_over_hosts_json() {
    let home = tempfile::tempdir().unwrap();
    put(home.path(), "apps.json", &fixture("apps.json"));
    put(home.path(), "hosts.json", &fixture("hosts.json"));
    assert_eq!(find_in(&dirs(home.path())).unwrap().expose(), APPS_TOKEN);
}

#[test]
fn hosts_json_is_the_fallback_and_enterprise_hosts_are_skipped() {
    let home = tempfile::tempdir().unwrap();
    put(home.path(), "hosts.json", &fixture("hosts.json"));
    assert_eq!(find_in(&dirs(home.path())).unwrap().expose(), HOSTS_TOKEN);
}

#[test]
fn an_apps_json_without_a_token_falls_through_to_hosts_json() {
    let home = tempfile::tempdir().unwrap();
    put(home.path(), "apps.json", r#"{"github.com:Iv1.x": {"user": "fab-user"}}"#);
    put(home.path(), "hosts.json", &fixture("hosts.json"));
    assert_eq!(find_in(&dirs(home.path())).unwrap().expose(), HOSTS_TOKEN);
}

#[test]
fn a_missing_directory_and_missing_files_are_not_found() {
    let home = tempfile::tempdir().unwrap();
    assert_eq!(find_in(&dirs(home.path())).unwrap_err(), Reason::NotFound);
    std::fs::create_dir_all(copilot_dir(home.path())).unwrap();
    assert_eq!(find_in(&dirs(home.path())).unwrap_err(), Reason::NotFound);
}

#[test]
fn files_without_a_usable_github_com_entry_have_no_token() {
    let home = tempfile::tempdir().unwrap();
    put(home.path(), "hosts.json", r#"{"ghe.example.test": {"oauth_token": "gho_other"}}"#);
    assert_eq!(find_in(&dirs(home.path())).unwrap_err(), Reason::NoToken);
}

#[test]
fn garbage_and_oversized_files_are_unreadable_never_a_panic() {
    let home = tempfile::tempdir().unwrap();
    for text in
        ["", "not json", "{\"github.com\":", &format!("{{\"pad\":\"{}\"}}", "x".repeat(70_000))]
    {
        put(home.path(), "apps.json", text);
        assert_eq!(
            find_in(&dirs(home.path())).unwrap_err(),
            Reason::Unreadable,
            "{:?}",
            text.len()
        );
    }
}

#[test]
fn a_directory_in_place_of_a_file_is_unreadable() {
    let home = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(copilot_dir(home.path()).join("apps.json")).unwrap();
    assert_eq!(find_in(&dirs(home.path())).unwrap_err(), Reason::Unreadable);
}

#[test]
fn a_byte_order_mark_from_a_windows_editor_is_accepted() {
    let home = tempfile::tempdir().unwrap();
    let dir = copilot_dir(home.path());
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("apps.json"),
        [b"\xEF\xBB\xBF".as_slice(), fixture("apps.json").as_bytes()].concat(),
    )
    .unwrap();
    assert_eq!(find_in(&dirs(home.path())).unwrap().expose(), APPS_TOKEN);
}

#[tokio::test]
async fn the_async_entry_point_reads_under_the_injected_home_only() {
    let home = tempfile::tempdir().unwrap();
    put(home.path(), "apps.json", &fixture("apps.json"));
    assert_eq!(find_token(home.path().to_path_buf()).await.unwrap().expose(), APPS_TOKEN);
    let elsewhere = tempfile::tempdir().unwrap();
    assert_eq!(find_token(elsewhere.path().to_path_buf()).await.unwrap_err(), Reason::NotFound);
}

#[tokio::test]
async fn each_failure_reason_reaches_the_view_as_a_fixed_identifier() {
    let server = FakeServer::always(|| Reply::json(200, &fixture("individual_pro.json")));
    let cases = [
        (None, "not_found"),
        (Some(r#"{"ghe.example.test": {"oauth_token": "gho_x"}}"#), "no_token"),
        (Some("not json"), "unreadable"),
    ];
    for (text, reason) in cases {
        let home = tempfile::tempdir().unwrap();
        if let Some(text) = text {
            put(home.path(), "hosts.json", text);
        }
        let view = poll(&settings(true), None, home.path(), &server).await;
        assert_eq!(view.state, ViewState::Error, "{reason}");
        let error = view.error.unwrap();
        assert_eq!((error.code, error.retryable), (ErrorCode::NotSignedIn, false));
        assert_eq!(error.params["reason"], reason);
        assert_eq!(error.params.len(), 1, "nothing but the reason");
    }
    server.assert_untouched();
}
