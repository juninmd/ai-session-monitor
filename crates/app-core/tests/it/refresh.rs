//! `refresh`: rate-limited per harness, never an error, never queued.

use std::time::Duration;

use usage_model::HarnessId;

use crate::collector::{FakeCollector, Step};
use crate::rig::{rig, seconds, secs, settle};
use crate::views::ready;

const CLAUDE: HarnessId = HarnessId::Claude;
const CODEX: HarnessId = HarnessId::Codex;

/// A rig whose scheduled polls stay out of the way for the first hour.
fn quiet() -> crate::rig::RigBuilder {
    rig().settings(|s| s.general.interval_secs = 3600)
}

#[tokio::test(start_paused = true)]
async fn a_refresh_is_dropped_inside_ten_seconds_and_honoured_from_then_on() {
    let rig = quiet().start().await;
    let claude = rig.fake(CLAUDE);

    rig.at(seconds(5)).await;
    rig.service.refresh(Some(CLAUDE));
    rig.at(Duration::from_millis(9_999)).await;
    rig.service.refresh(Some(CLAUDE));
    settle().await;
    assert_eq!(claude.count(), 1, "inside the gap: dropped, and not an error");

    rig.at(seconds(10)).await;
    rig.service.refresh(Some(CLAUDE));
    settle().await;
    assert_eq!(claude.count(), 2, "exactly ten seconds after the last start");

    rig.at(Duration::from_millis(19_999)).await;
    rig.service.refresh(Some(CLAUDE));
    settle().await;
    assert_eq!(claude.count(), 2, "the gap counts from the new read");
    rig.at(seconds(20)).await;
    rig.service.refresh(Some(CLAUDE));
    settle().await;
    assert_eq!(claude.offsets_ms(rig.start), secs(&[0, 10, 20]));
}

#[tokio::test(start_paused = true)]
async fn the_gap_is_kept_per_harness() {
    let rig = quiet().start().await;
    rig.at(seconds(10)).await;

    rig.service.refresh(Some(CLAUDE));
    settle().await;
    assert_eq!((rig.fake(CLAUDE).count(), rig.fake(CODEX).count()), (2, 1));

    rig.service.refresh(Some(CODEX));
    settle().await;
    assert_eq!((rig.fake(CLAUDE).count(), rig.fake(CODEX).count()), (2, 2));

    rig.at(seconds(11)).await;
    rig.service.refresh(Some(CLAUDE));
    settle().await;
    assert_eq!(rig.fake(CLAUDE).count(), 2, "one second after its own last read");
}

#[tokio::test(start_paused = true)]
async fn refreshing_everything_applies_the_gap_to_each_harness() {
    let rig = quiet().start().await;
    rig.at(seconds(10)).await;
    rig.service.refresh(None);
    settle().await;
    for id in HarnessId::ALL {
        assert_eq!(rig.fake(id).count(), 2, "{id:?}");
    }

    rig.at(seconds(12)).await;
    rig.service.refresh(None);
    settle().await;
    for id in HarnessId::ALL {
        assert_eq!(rig.fake(id).count(), 2, "{id:?} was read two seconds ago");
    }
}

#[tokio::test(start_paused = true)]
async fn a_burst_of_refreshes_reads_once() {
    let rig = quiet().start().await;
    rig.at(seconds(10)).await;
    for _ in 0..100 {
        rig.service.refresh(Some(CLAUDE));
    }
    settle().await;

    assert_eq!(rig.fake(CLAUDE).count(), 2);
}

#[tokio::test(start_paused = true)]
async fn a_refresh_during_a_slow_read_is_dropped_not_queued() {
    let slow = Step::After(seconds(20), ready(CLAUDE, 10.0));
    let rig = quiet().collector(FakeCollector::always(CLAUDE, slow)).start().await;

    rig.at(seconds(15)).await;
    rig.service.refresh(Some(CLAUDE));
    rig.at(seconds(21)).await;
    assert_eq!(rig.fake(CLAUDE).count(), 1, "busy when asked, and not remembered afterwards");

    rig.service.refresh(Some(CLAUDE));
    settle().await;
    assert_eq!(rig.fake(CLAUDE).count(), 2, "idle now, and long past the gap");
}

#[tokio::test(start_paused = true)]
async fn a_refresh_before_run_reads_once_not_twice() {
    let mut rig = quiet().build();
    rig.service.refresh(None);
    rig.run().await;

    for id in HarnessId::ALL {
        assert_eq!(rig.fake(id).count(), 1, "{id:?}: the request and the first poll are one read");
    }
}
