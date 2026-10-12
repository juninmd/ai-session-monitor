//! Many threads saving at once: one winner on disk, in memory and in the last event.

use std::sync::Arc;
use std::time::Duration;

use app_core::AppService;
use usage_model::ErrorCode;

use crate::doubles::FixedClock;
use crate::rig::{Rig, rig};
use crate::sink::RecordingSink;

const THREADS: u32 = 8;
const SAVES: u32 = 25;

/// The sink pauses inside `settings_changed`, so overlapping deliveries would show.
fn racing_rig() -> Rig {
    let sink = RecordingSink::pausing_in_settings(Duration::from_micros(300));
    rig().clock(Arc::new(FixedClock)).sink(sink).build()
}

fn save_interval(service: &AppService, secs: u32) {
    let mut settings = service.settings();
    settings.general.interval_secs = secs;
    service.save_settings(settings).unwrap_or_else(|e| panic!("rejected: {e:?}"));
}

/// The race above only overlaps when timing allows; this one parks a delivery to force it.
#[test]
fn a_save_during_a_delivery_returns_at_once_and_the_rerun_carries_the_newest_settings() {
    let (sink, gate) = RecordingSink::gated_first_settings();
    let rig = rig().clock(Arc::new(FixedClock)).sink(sink).build();
    let service = &rig.service;
    std::thread::scope(|scope| {
        let first = scope.spawn(|| save_interval(service, 60));
        gate.wait_until_parked();

        save_interval(service, 120);
        assert!(rig.sink.settings().is_empty(), "the second save must not deliver by itself");
        assert_eq!(rig.sink.max_overlap(), 1, "and must not enter the busy sink");

        gate.release();
        first.join().unwrap_or_else(|_| panic!("the first save panicked"));
    });

    let intervals: Vec<u32> = rig.sink.settings().iter().map(|s| s.general.interval_secs).collect();
    assert_eq!(intervals, [60, 120], "save order, and the last event is the newest settings");
    assert_eq!(rig.sink.max_overlap(), 1);
}

#[test]
fn concurrent_saves_leave_the_file_the_memory_and_the_last_event_equal() {
    let rig = racing_rig();
    std::thread::scope(|scope| {
        for thread in 0..THREADS {
            let service = &rig.service;
            scope.spawn(move || {
                for save in 0..SAVES {
                    let mut settings = service.settings();
                    settings.general.interval_secs = 30 + thread * SAVES + save;
                    service.save_settings(settings).unwrap_or_else(|e| panic!("rejected: {e:?}"));
                }
            });
        }
    });

    let in_memory = rig.service.settings();
    assert_eq!(rig.saved_settings(), in_memory, "the file holds the save that won");
    let events = rig.sink.settings();
    assert_eq!(events.last(), Some(&in_memory), "and the last event announced it");
    assert!(!events.is_empty() && events.len() <= (THREADS * SAVES) as usize);
    assert_eq!(rig.sink.max_overlap(), 1, "the sink is never entered twice at once");
}

#[test]
fn rejected_saves_racing_with_accepted_ones_change_nothing() {
    let rig = racing_rig();
    std::thread::scope(|scope| {
        for thread in 0..THREADS {
            let service = &rig.service;
            scope.spawn(move || {
                for save in 0..SAVES {
                    let mut settings = service.settings();
                    if (thread + save) % 2 == 0 {
                        settings.general.interval_secs = 30 + thread * SAVES + save;
                        service
                            .save_settings(settings)
                            .unwrap_or_else(|e| panic!("rejected: {e:?}"));
                    } else {
                        // 7777 would clamp to 3600, which no accepted save uses: it would show.
                        settings.general.interval_secs = 7777;
                        settings.litellm.base_url = Some("ftp://example.com".to_owned());
                        let error = service.save_settings(settings).expect_err("not https");
                        assert_eq!(error.code, ErrorCode::InvalidUrl);
                    }
                }
            });
        }
    });

    let in_memory = rig.service.settings();
    assert!(in_memory.litellm.base_url.is_none());
    assert_eq!(rig.saved_settings(), in_memory);
    let events = rig.sink.settings();
    assert_eq!(events.last(), Some(&in_memory));
    assert!(events.iter().all(|s| s.litellm.base_url.is_none() && s.general.interval_secs < 3600));
    assert_eq!(rig.sink.max_overlap(), 1);
}
