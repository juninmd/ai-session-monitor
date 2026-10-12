//! The burndown history wired into the service: sampled from fresh readings, off when the user
//! says so, written rarely, flushed on shutdown, cleared on request and never a startup failure.

use usage_model::{ErrorCode, HarnessId};

use crate::collector::{FakeCollector, Step};
use crate::rig::{Rig, rig, seconds};
use crate::views::ready;

const CODEX: HarnessId = HarnessId::Codex;

fn climbing() -> Rig {
    let steps = [10.0, 20.0, 30.0, 40.0].map(|used| Step::View(ready(CODEX, used)));
    rig()
        .collector(FakeCollector::scripted(CODEX, steps.to_vec(), Step::View(ready(CODEX, 40.0))))
        .build()
}

fn used(rig: &Rig) -> Vec<f64> {
    let history = rig.service.history(CODEX, None).unwrap_or_else(|e| panic!("history: {e:?}"));
    history.epochs.iter().flat_map(|e| e.points.iter().map(|p| p.used_pct)).collect()
}

fn history_file(rig: &Rig) -> std::path::PathBuf {
    rig.data_dir.join("history.json")
}

#[tokio::test(start_paused = true)]
async fn fresh_readings_become_points_of_one_epoch() {
    let mut rig = climbing();
    rig.run().await;
    rig.at(seconds(200)).await;

    assert_eq!(used(&rig), [10.0, 20.0, 30.0, 40.0]);
    let history = rig.service.history(CODEX, Some(1)).unwrap_or_else(|e| panic!("history: {e:?}"));
    assert_eq!(history.epochs.len(), 1);
    assert_eq!(history.epochs[0].window_id, "five_hour");
}

#[tokio::test(start_paused = true)]
async fn switched_off_nothing_is_recorded_or_served_and_nothing_is_written() {
    let mut rig = climbing();
    rig.edit(|s| s.general.history_enabled = false);
    rig.run().await;
    rig.at(seconds(200)).await;
    rig.service.shutdown_and_wait(seconds(2)).await;

    assert!(used(&rig).is_empty());
    assert!(!history_file(&rig).exists());
    rig.edit(|s| s.general.history_enabled = true);
    assert!(used(&rig).is_empty(), "readings taken while off were never kept");
}

#[tokio::test(start_paused = true)]
async fn the_file_is_written_at_most_once_a_minute_and_on_shutdown() {
    let mut rig = climbing();
    rig.run().await;
    rig.at(seconds(30)).await;
    assert!(!history_file(&rig).exists(), "no write inside the first minute");

    rig.at(seconds(130)).await;
    assert!(history_file(&rig).exists(), "the debounced write happened");

    rig.at(seconds(190)).await;
    rig.service.shutdown_and_wait(seconds(2)).await;
    let text =
        std::fs::read_to_string(history_file(&rig)).unwrap_or_else(|e| panic!("history file: {e}"));
    assert!(text.contains("40.0"), "the shutdown flush kept the last reading: {text}");
}

#[tokio::test(start_paused = true)]
async fn clearing_empties_the_view_and_the_file() {
    let mut rig = climbing();
    rig.run().await;
    rig.at(seconds(200)).await;
    rig.service.shutdown_and_wait(seconds(2)).await;
    assert!(history_file(&rig).exists());

    rig.service.clear_history().unwrap_or_else(|e| panic!("clear: {e:?}"));
    assert!(used(&rig).is_empty());
    assert!(!history_file(&rig).exists());
    rig.service.clear_history().unwrap_or_else(|e| panic!("clearing nothing: {e:?}"));
}

#[tokio::test(start_paused = true)]
async fn a_limit_outside_one_to_eight_is_refused_without_the_value() {
    let rig = climbing();
    for bad in [0, 9, 255] {
        let error = rig
            .service
            .history(CODEX, Some(bad))
            .err()
            .unwrap_or_else(|| panic!("accepted a bad limit"));
        assert_eq!(error.code, ErrorCode::InvalidInput);
        assert_eq!(error.params.get("field").map(String::as_str), Some("limit"));
        assert!(!format!("{error:?}").contains(&bad.to_string()));
    }
    assert!(rig.service.history(CODEX, Some(8)).is_ok());
}

#[tokio::test(start_paused = true)]
async fn a_damaged_history_file_never_stops_the_start_or_raises_a_notice() {
    let mut rig = rig().file("history.json", b"\x00\x01 password=hunter2 {{").build();
    rig.run().await;
    rig.at(seconds(10)).await;

    assert!(rig.sink.notices().is_empty());
    assert!(rig.data_dir.join("history.corrupt.json").exists());
    assert!(!rig.sink.dump().contains("hunter2"));
}
