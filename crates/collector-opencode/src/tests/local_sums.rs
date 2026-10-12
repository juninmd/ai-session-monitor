//! What the local estimate sums: window boundaries, message kinds and every way a number can be
//! missing or wrong. A wrong number is unknown, never a silent zero or a clamped value.

use std::fs;

use super::fake_db::{FakeDb, NOW_MS, assistant_json};
use super::local_support::{facts, populated, read, reason};
use crate::local::Reason;

fn single_row(data: &str) -> FakeDb {
    let fake = FakeDb::new();
    let writer = fake.writer();
    fake.add_raw(&writer, "assistant", NOW_MS - 1_000, data);
    drop(writer);
    fake
}

fn recent(body: &str) -> String {
    format!(r#"{{{body},"time":{{"created":{}}}}}"#, NOW_MS - 1_000)
}

#[tokio::test]
async fn sums_follow_the_window_boundaries_inclusively() {
    let fake = populated();
    let got = facts(read(&fake.path()).await);
    // Rows sit exactly on, and one millisecond outside, each of the 5 h / 7 d / 30 d marks.
    assert_eq!((got.cost_5h, got.cost_7d, got.cost_30d), (Some(0.3), Some(1.5), Some(6.3)));
    // input + output + reasoning; the cache counters in every row are not part of it.
    assert_eq!((got.tokens_5h, got.tokens_7d), (Some(480.0), Some(2_400.0)));
}

#[tokio::test]
async fn only_assistant_messages_count() {
    let fake = FakeDb::new();
    let writer = fake.writer();
    for kind in ["user", "tool", "system", "Assistant", ""] {
        fake.add_raw(
            &writer,
            kind,
            NOW_MS - 1_000,
            &assistant_json(NOW_MS - 1_000, 99.0, [9, 9, 9]),
        );
    }
    drop(writer);
    let got = facts(read(&fake.path()).await);
    assert_eq!((got.cost_5h, got.cost_30d, got.tokens_5h), (Some(0.0), Some(0.0), Some(0.0)));
}

#[tokio::test]
async fn an_empty_database_reports_a_real_zero() {
    let fake = FakeDb::new();
    let got = facts(read(&fake.path()).await);
    assert_eq!((got.cost_5h, got.cost_7d, got.cost_30d), (Some(0.0), Some(0.0), Some(0.0)));
    assert_eq!((got.tokens_5h, got.tokens_7d), (Some(0.0), Some(0.0)));
}

#[tokio::test]
async fn a_large_history_is_summed_exactly_in_one_pass() {
    let fake = FakeDb::new();
    let writer = fake.writer();
    writer.execute_batch("BEGIN").unwrap();
    // 20 000 messages spread evenly over exactly 30 days, one every 129.6 s.
    for n in 0..20_000_i64 {
        fake.add_assistant(&writer, NOW_MS - n * 129_600, 0.001, [3, 2, 1]);
    }
    writer.execute_batch("COMMIT").unwrap();
    drop(writer);
    let got = facts(read(&fake.path()).await);
    assert_eq!((got.cost_5h, got.cost_7d, got.cost_30d), (Some(0.139), Some(4.667), Some(20.0)));
    assert_eq!((got.tokens_5h, got.tokens_7d), (Some(834.0), Some(28_002.0)));
}

#[tokio::test]
async fn non_numeric_values_are_ignored_not_coerced() {
    let fake = FakeDb::new();
    let writer = fake.writer();
    let at = NOW_MS - 1_000;
    let odd = [
        recent(r#""cost":"0.5","tokens":{"input":"100","output":"50"}"#),
        recent(r#""cost":null,"tokens":null"#),
        recent(r#""cost":{"usd":1},"tokens":{"input":[1],"output":true}"#),
        recent(r#""cost":false,"tokens":{}"#),
    ];
    for data in &odd {
        fake.add_raw(&writer, "assistant", at, data);
    }
    fake.add_assistant(&writer, at, 0.25, [10, 5, 1]);
    drop(writer);
    let got = facts(read(&fake.path()).await);
    assert_eq!((got.cost_5h, got.tokens_5h), (Some(0.25), Some(16.0)));
}

#[tokio::test]
async fn fractional_numbers_are_numbers() {
    let body = r#""cost":0.5,"tokens":{"input":10.5,"output":5.25}"#;
    let data = format!(r#"{{{body},"time":{{"created":{}.5}}}}"#, NOW_MS - 1_000);
    let got = facts(read(&single_row(&data).path()).await);
    assert_eq!((got.cost_5h, got.tokens_5h), (Some(0.5), Some(15.75)));
}

#[tokio::test]
async fn a_text_timestamp_puts_a_row_in_no_window() {
    let fake = FakeDb::new();
    let writer = fake.writer();
    let text_time =
        r#"{"cost":50,"tokens":{"input":1,"output":1},"time":{"created":"9999999999999"}}"#;
    fake.add_raw(&writer, "assistant", NOW_MS - 1_000, text_time);
    fake.add_assistant(&writer, NOW_MS - 2_000, 0.25, [1, 1, 1]);
    drop(writer);
    let got = facts(read(&fake.path()).await);
    assert_eq!((got.cost_5h, got.cost_30d, got.tokens_7d), (Some(0.25), Some(0.25), Some(3.0)));
}

#[tokio::test]
async fn idle_for_a_month_is_a_real_zero_while_old_messages_still_have_a_time() {
    let fake = FakeDb::new();
    let writer = fake.writer();
    let sixty_days_ago = NOW_MS - 60 * 86_400_000;
    fake.add_assistant(&writer, sixty_days_ago, 9.0, [900, 90, 9]);
    drop(writer);
    let got = facts(read(&fake.path()).await);
    assert_eq!((got.cost_30d, got.tokens_7d), (Some(0.0), Some(0.0)));
}

#[tokio::test]
async fn assistant_messages_none_of_which_has_a_readable_time_mean_the_layout_changed() {
    let moved = r#"{"cost":0.5,"tokens":{"input":1,"output":1},"time":{"start":1799999999000}}"#;
    assert_eq!(reason(read(&single_row(moved).path()).await), Reason::Schema);
    let as_text =
        r#"{"cost":0.5,"tokens":{"input":1,"output":1},"time":{"created":"1799999999000"}}"#;
    assert_eq!(reason(read(&single_row(as_text).path()).await), Reason::Schema);
}

#[tokio::test]
async fn a_malformed_json_row_is_skipped_not_fatal() {
    let fake = FakeDb::new();
    let writer = fake.writer();
    fake.add_raw(&writer, "assistant", NOW_MS - 1_000, "{not json at all");
    fake.add_raw(&writer, "assistant", NOW_MS - 2_000, "");
    fake.add_assistant(&writer, NOW_MS - 3_000, 0.5, [1, 2, 3]);
    drop(writer);
    let got = facts(read(&fake.path()).await);
    assert_eq!((got.cost_5h, got.tokens_5h), (Some(0.5), Some(6.0)));
}

#[tokio::test]
async fn rows_without_any_usable_number_mean_the_layout_changed() {
    let data = recent(r#""usage":{"price":1,"in":5}"#);
    assert_eq!(reason(read(&single_row(&data).path()).await), Reason::Schema);
}

#[tokio::test]
async fn missing_numbers_stay_unknown_instead_of_becoming_zero() {
    let got = facts(read(&single_row(&recent(r#""cost":0.5"#)).path()).await);
    assert_eq!((got.cost_5h, got.tokens_5h, got.tokens_7d), (Some(0.5), None, None));
}

#[tokio::test]
async fn negative_sums_are_unknown_not_clamped() {
    let data = recent(r#""cost":-2.5,"tokens":{"input":5,"output":1}"#);
    let got = facts(read(&single_row(&data).path()).await);
    assert_eq!((got.cost_5h, got.cost_30d, got.tokens_5h), (None, None, Some(6.0)));
}

#[tokio::test]
async fn numbers_that_overflow_are_unknown_never_infinite_and_never_an_error() {
    // 1e999 is valid JSON that no double can hold.
    let data = recent(r#""cost":1e999,"tokens":{"input":5,"output":1}"#);
    let got = facts(read(&single_row(&data).path()).await);
    assert_eq!((got.cost_5h, got.tokens_5h), (None, Some(6.0)));

    // Finite sums whose micro-dollar rounding, or whose token total, would overflow.
    let data = recent(r#""cost":1e303,"tokens":{"input":5,"output":1}"#);
    let got = facts(read(&single_row(&data).path()).await);
    assert_eq!((got.cost_5h, got.tokens_5h), (None, Some(6.0)));
    let data = recent(r#""cost":0.5,"tokens":{"input":1e308,"output":1e308}"#);
    let got = facts(read(&single_row(&data).path()).await);
    assert_eq!((got.cost_5h, got.tokens_5h), (Some(0.5), None));
    // Nothing usable left in a window that has messages: the layout is not what we expect.
    let data = recent(r#""cost":1e303,"tokens":{"input":1e308,"output":1e308}"#);
    assert_eq!(reason(read(&single_row(&data).path()).await), Reason::Schema);

    // Integer sums past i64 would abort a plain SUM(); the read must survive and stay finite.
    let fake = FakeDb::new();
    let writer = fake.writer();
    let data = recent(r#""cost":9223372036854775807,"tokens":{"input":9223372036854775807}"#);
    for _ in 0..2 {
        fake.add_raw(&writer, "assistant", NOW_MS - 1_000, &data);
    }
    drop(writer);
    let got = facts(read(&fake.path()).await);
    assert!(got.cost_5h.is_some_and(|v| v.is_finite() && v > 1e19), "{:?}", got.cost_5h);
    assert!(got.tokens_5h.is_some_and(|v| v.is_finite() && v > 1e19), "{:?}", got.tokens_5h);
}

#[tokio::test]
async fn unicode_directories_work() {
    let fake = populated();
    let dir = fake.dir().join("pasta \u{e7}\u{e3}o \u{65e5}\u{672c} \u{1f511}");
    fs::create_dir(&dir).unwrap();
    fs::copy(fake.path(), dir.join("opencode.db")).unwrap();
    let got = facts(read(&dir.join("opencode.db")).await);
    assert_eq!(got.cost_30d, Some(6.3));
}
