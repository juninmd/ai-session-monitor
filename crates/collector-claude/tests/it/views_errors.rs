//! The Error views: what happens when a file is damaged or the input is unusable, and that
//! nothing sticks once the file is fine again.

use collector_claude::collect;
use serde_json::json;
use usage_model::settings::ClaudeSettings;
use usage_model::{ErrorCode, HarnessView, ViewState};

use crate::common::{NOW, ok_both, settings_foreign, wire};
use crate::setup::{installed, recording, view_of};

fn assert_error(view: &HarnessView, code: ErrorCode, retryable: bool) {
    assert_eq!(view.state, ViewState::Error, "{view:?}");
    let error = view.error.as_ref().unwrap();
    assert_eq!((error.code, error.retryable), (code, retryable), "{view:?}");
    assert!(view.windows.is_empty() && view.notices.is_empty() && view.source.is_none());
}

#[tokio::test]
async fn settings_that_cannot_be_edited_are_unreadable_and_left_as_they_were() {
    for text in ["{ not json", "[1, 2]", "", "{\"a\":1,}", "{/* note */}"] {
        let env = recording();
        env.write_settings(text);
        assert_error(&view_of(&env).await, ErrorCode::SettingsUnreadable, false);
        assert_eq!(env.settings_text(), text, "collecting only ever reads");
    }
}

#[tokio::test]
async fn a_recording_that_is_not_json_is_a_retryable_bad_response() {
    let env = installed();
    for text in ["", "not json", "{\"v\":1,", "[1,", "\u{0}\u{0}"] {
        env.write_rate_limits(text);
        assert_error(&view_of(&env).await, ErrorCode::BadResponse, true);
    }
}

#[tokio::test]
async fn a_recording_of_another_shape_means_the_format_changed() {
    let env = installed();
    let shapes = [
        json!({"v": 2, "capturedAt": NOW, "rateLimits": null}),
        json!({"capturedAt": NOW, "rateLimits": null}),
        json!({"v": 1, "rateLimits": null}),
        json!({"v": 1, "capturedAt": NOW, "rateLimits": [1]}),
        json!({"v": 1, "capturedAt": NOW, "rateLimits": "x"}),
        json!({"v": 1, "capturedAt": NOW, "rateLimits": {"monthly": {"used_percentage": 1}}}),
        json!({"v": 1, "capturedAt": NOW, "rateLimits": {"five_hour": 5, "seven_day": []}}),
        json!([1, 2, 3]),
        json!("x"),
    ];
    for shape in shapes {
        env.write_rate_limits(&shape.to_string());
        assert_error(&view_of(&env).await, ErrorCode::EndpointChanged, false);
    }
}

#[tokio::test]
async fn an_oversized_recording_is_refused_without_being_read_whole() {
    let env = installed();
    env.write_rate_limits(&" ".repeat(64 * 1024 + 1));
    assert_error(&view_of(&env).await, ErrorCode::ResponseTooLarge, false);
    env.write_rate_limits(&format!("{}{}", ok_both(), " ".repeat(64 * 1024 - ok_both().len())));
    assert_eq!(view_of(&env).await.state, ViewState::Ready, "exactly at the limit is fine");
}

#[tokio::test]
async fn a_directory_where_the_recording_belongs_is_a_retryable_internal_error() {
    let env = installed();
    std::fs::create_dir_all(env.rate_limits()).unwrap();
    assert_error(&view_of(&env).await, ErrorCode::Internal, true);
}

#[tokio::test]
async fn a_byte_order_mark_from_a_windows_editor_is_tolerated() {
    let env = installed();
    env.write_rate_limits(&format!("\u{feff}{}", ok_both()));
    assert_eq!(view_of(&env).await.state, ViewState::Ready);
}

#[tokio::test]
async fn an_unusable_config_dir_is_an_input_error_naming_the_field() {
    let env = recording();
    for dir in ["relative/dir", ".claude", "C:drive-relative", "bad\0dir"] {
        let settings = ClaudeSettings { config_dir: Some(dir.to_owned()), ..env.settings() };
        let view = collect(&settings, &env.ctx()).await;
        assert_error(&view, ErrorCode::InvalidInput, false);
        assert_eq!(view.error.unwrap().params["field"], "configDir", "{dir:?}");
    }
}

#[tokio::test]
async fn a_damaged_file_is_forgotten_as_soon_as_it_is_fine_again() {
    let env = recording();
    env.write_rate_limits("{ torn");
    assert_error(&view_of(&env).await, ErrorCode::BadResponse, true);
    env.write_rate_limits(&ok_both());
    assert_eq!(view_of(&env).await.state, ViewState::Ready);

    env.write_settings("{ torn");
    assert_error(&view_of(&env).await, ErrorCode::SettingsUnreadable, false);
    env.write_settings(settings_foreign());
    assert_eq!(view_of(&env).await.state, ViewState::NeedsSetup);
}

#[tokio::test]
async fn damaged_settings_win_over_a_good_recording() {
    let env = recording();
    env.write_settings("{ torn");
    assert_error(&view_of(&env).await, ErrorCode::SettingsUnreadable, false);
}

#[tokio::test]
async fn what_was_in_a_damaged_file_never_reaches_the_view() {
    let env = recording();
    env.write_rate_limits("tok=SEGREDO-NO-ARQUIVO {");
    assert!(!wire(&view_of(&env).await).contains("SEGREDO"));
    env.write_rate_limits(&ok_both());
    env.write_settings("{\"token\": \"SEGREDO-NO-ARQUIVO\" ");
    assert!(!wire(&view_of(&env).await).contains("SEGREDO"));
}
