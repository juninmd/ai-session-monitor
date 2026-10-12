use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc;
use std::thread;

use super::*;

#[test]
fn every_sequential_trigger_delivers_once() {
    let relay = Relay::default();
    let mut calls = 0;
    relay.trigger(|| calls += 1);
    relay.trigger(|| calls += 1);
    assert_eq!(calls, 2);
}

#[test]
fn a_trigger_during_a_delivery_is_folded_into_one_rerun_by_the_delivering_thread() {
    let relay = Relay::default();
    let calls = AtomicU64::new(0);
    let (started_tx, started_rx) = mpsc::channel::<()>();
    let (release_tx, release_rx) = mpsc::channel::<()>();
    thread::scope(|scope| {
        let (relay, calls) = (&relay, &calls);
        let first = scope.spawn(move || {
            relay.trigger(|| {
                if calls.fetch_add(1, Ordering::SeqCst) == 0 {
                    started_tx.send(()).expect("signal");
                    release_rx.recv().expect("release");
                }
            });
        });
        started_rx.recv().expect("first delivery started");
        // Returns at once: the first thread owns delivery and will rerun for this change.
        relay.trigger(|| {
            calls.fetch_add(100, Ordering::SeqCst);
        });
        assert_eq!(calls.load(Ordering::SeqCst), 1, "the second caller must not deliver itself");
        release_tx.send(()).expect("release");
        first.join().expect("join");
    });
    assert_eq!(calls.load(Ordering::SeqCst), 2, "one rerun, carried by the first thread");
}

#[test]
fn deliveries_never_overlap_and_the_last_one_sees_the_last_change() {
    let relay = Relay::default();
    let value = AtomicU64::new(0);
    let seen = AtomicU64::new(0);
    let inside = AtomicBool::new(false);
    thread::scope(|scope| {
        for _ in 0..8 {
            scope.spawn(|| {
                for _ in 0..500 {
                    value.fetch_add(1, Ordering::SeqCst);
                    relay.trigger(|| {
                        assert!(!inside.swap(true, Ordering::SeqCst), "overlapping delivery");
                        seen.store(value.load(Ordering::SeqCst), Ordering::SeqCst);
                        inside.store(false, Ordering::SeqCst);
                    });
                }
            });
        }
    });
    assert_eq!(value.load(Ordering::SeqCst), 4000);
    assert_eq!(seen.load(Ordering::SeqCst), 4000);
}

#[test]
fn a_panicking_delivery_does_not_wedge_later_ones() {
    let relay = Relay::default();
    let result = catch_unwind(AssertUnwindSafe(|| relay.trigger(|| panic!("sink bug"))));
    assert!(result.is_err());
    let mut delivered = false;
    relay.trigger(|| delivered = true);
    assert!(delivered);
}
