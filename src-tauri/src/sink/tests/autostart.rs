//! A startup entry whose update fails: told once, retried on the tick, within a cap.

use super::*;

fn syncs(rig: &Rig) -> Vec<String> {
    rig.rec.calls().into_iter().filter(|call| call.starts_with("autostart:")).collect()
}

fn told(rig: &Rig) -> Vec<String> {
    rig.rec.toasts.lock().expect("lock").iter().map(|toast| toast.body.clone()).collect()
}

fn count(syncs: &[String], call: &str) -> usize {
    syncs.iter().filter(|seen| *seen == call).count()
}

const ON_FAILED: &str = "Não foi possível ligar 'Iniciar com o Windows'.";
const OFF_FAILED: &str = "Não foi possível desligar 'Iniciar com o Windows'.";

#[test]
fn a_failed_update_is_retried_on_the_tick_and_told_once() {
    let rig = rig();
    rig.rec.autostart_failures.store(1, Ordering::SeqCst);
    rig.sink.prime(&settings(None, true));
    for _ in 0..3 {
        rig.sink.tick();
    }
    assert_eq!(syncs(&rig), ["autostart:true:Startup", "autostart:true:Startup"]);
    assert_eq!(told(&rig), [ON_FAILED]);
}

#[test]
fn the_retries_stop_after_three_and_the_user_is_told_once() {
    let rig = rig();
    rig.rec.autostart_failures.store(u32::MAX, Ordering::SeqCst);
    rig.sink.prime(&settings(None, true));
    for _ in 0..10 {
        rig.sink.tick();
    }
    // Saving the same value again is not a new attempt either.
    rig.sink.settings_changed(&settings(None, true));
    for _ in 0..3 {
        rig.sink.tick();
    }
    assert_eq!(syncs(&rig).len(), 4, "the first attempt and three retries");
    assert_eq!(told(&rig), [ON_FAILED]);
}

#[test]
fn a_new_change_starts_a_failure_streak_of_its_own() {
    let rig = rig();
    rig.rec.autostart_failures.store(u32::MAX, Ordering::SeqCst);
    rig.sink.prime(&settings(None, true));
    for _ in 0..10 {
        rig.sink.tick();
    }
    rig.sink.settings_changed(&settings(None, false));
    for _ in 0..10 {
        rig.sink.tick();
    }
    let seen = syncs(&rig);
    assert_eq!(count(&seen, "autostart:true:Startup"), 4, "{seen:?}");
    // A retry is the same request again: it keeps the origin, so a start-up pass never turns
    // into a user's change.
    assert_eq!(count(&seen, "autostart:false:UserChange"), 4, "{seen:?}");
    assert_eq!(told(&rig), [ON_FAILED, OFF_FAILED]);
}

#[test]
fn a_streak_that_recovered_does_not_silence_the_next_one() {
    let rig = rig();
    rig.rec.autostart_failures.store(1, Ordering::SeqCst);
    rig.sink.prime(&settings(None, true));
    rig.sink.tick();
    rig.sink.tick();
    rig.rec.autostart_failures.store(1, Ordering::SeqCst);
    rig.sink.settings_changed(&settings(None, false));
    rig.sink.tick();
    assert_eq!(told(&rig), [ON_FAILED, OFF_FAILED]);
}

#[test]
fn a_healthy_update_is_never_retried_or_reported() {
    let rig = rig();
    rig.sink.prime(&settings(None, true));
    for _ in 0..5 {
        rig.sink.tick();
    }
    assert_eq!(syncs(&rig), ["autostart:true:Startup"]);
    assert!(told(&rig).is_empty());
}

#[test]
fn the_failure_toast_is_in_the_chosen_language() {
    let rig = rig();
    rig.rec.autostart_failures.store(1, Ordering::SeqCst);
    let mut english = settings(None, true);
    english.general.language = Language::En;
    rig.sink.prime(&english);
    rig.sink.tick();
    assert_eq!(told(&rig), ["Could not turn on 'Start with Windows'."]);
}
