use std::sync::Mutex;

use super::*;

#[cfg(windows)]
mod registry;

/// Records every call. Windows keeps two facts about an entry: the Run value (`registered`) and
/// the Task Manager verdict beside it (`approved`). Like `auto-launch`, `is_enabled` needs both,
/// `enable` sets both and `disable` removes the Run value only.
struct Fake {
    registered: Mutex<bool>,
    approved: Mutex<bool>,
    calls: Mutex<Vec<&'static str>>,
    fail_on: Option<&'static str>,
}

impl Fake {
    fn new(registered: bool, approved: bool) -> Self {
        Self {
            registered: Mutex::new(registered),
            approved: Mutex::new(approved),
            calls: Mutex::new(Vec::new()),
            fail_on: None,
        }
    }

    /// No Run value. Windows holds no verdict against a name it has never seen.
    fn absent() -> Self {
        Self::new(false, true)
    }

    fn healthy() -> Self {
        Self::new(true, true)
    }

    /// The Run value is there and the user switched it off in Task Manager.
    fn user_disabled() -> Self {
        Self::new(true, false)
    }

    fn failing(self, call: &'static str) -> Self {
        Self { fail_on: Some(call), ..self }
    }

    fn record(&self, call: &'static str) -> Result<(), AutostartError> {
        self.calls.lock().expect("lock").push(call);
        if self.fail_on == Some(call) { Err(AutostartError("boom".into())) } else { Ok(()) }
    }

    fn calls(&self) -> Vec<&'static str> {
        self.calls.lock().expect("lock").clone()
    }

    fn registered(&self) -> bool {
        *self.registered.lock().expect("lock")
    }

    fn approved(&self) -> bool {
        *self.approved.lock().expect("lock")
    }
}

impl Autostart for Fake {
    fn is_enabled(&self) -> Result<bool, AutostartError> {
        self.record("is_enabled")?;
        Ok(self.registered() && self.approved())
    }

    fn is_registered(&self) -> Result<bool, AutostartError> {
        self.record("is_registered")?;
        Ok(self.registered())
    }

    fn enable(&self) -> Result<(), AutostartError> {
        self.record("enable")?;
        *self.registered.lock().expect("lock") = true;
        *self.approved.lock().expect("lock") = true;
        Ok(())
    }

    fn disable(&self) -> Result<(), AutostartError> {
        self.record("disable")?;
        *self.registered.lock().expect("lock") = false;
        Ok(())
    }
}

#[test]
fn the_plan_only_writes_when_the_state_differs() {
    assert_eq!(plan(true, false), Plan::Enable);
    assert_eq!(plan(false, true), Plan::Disable);
    assert_eq!(plan(true, true), Plan::Keep);
    assert_eq!(plan(false, false), Plan::Keep);
}

#[test]
fn turning_it_on_registers_once() {
    let fake = Fake::absent();
    assert_eq!(reconcile(&fake, true, Origin::UserChange).expect("reconcile"), Plan::Enable);
    assert_eq!(fake.calls(), ["is_enabled", "enable"]);
}

#[test]
fn turning_it_off_removes_the_registration() {
    let fake = Fake::healthy();
    assert_eq!(reconcile(&fake, false, Origin::UserChange).expect("reconcile"), Plan::Disable);
    assert_eq!(fake.calls(), ["is_enabled", "disable"]);
}

#[test]
fn reconciling_twice_is_idempotent() {
    let fake = Fake::absent();
    reconcile(&fake, true, Origin::UserChange).expect("first");
    assert_eq!(reconcile(&fake, true, Origin::UserChange).expect("second"), Plan::Keep);
    assert_eq!(fake.calls(), ["is_enabled", "enable", "is_enabled"]);
}

#[test]
fn a_matching_state_touches_nothing() {
    let fake = Fake::absent();
    assert_eq!(reconcile(&fake, false, Origin::UserChange).expect("reconcile"), Plan::Keep);
    assert_eq!(fake.calls(), ["is_enabled"]);
}

#[test]
fn an_unreadable_state_stops_before_any_write() {
    for (origin, read) in [(Origin::UserChange, "is_enabled"), (Origin::Startup, "is_registered")] {
        let fake = Fake::absent().failing(read);
        assert!(reconcile(&fake, true, origin).is_err(), "{origin:?}");
        assert_eq!(fake.calls(), [read], "{origin:?}");
    }
}

#[test]
fn a_failed_write_is_reported() {
    let fake = Fake::absent().failing("enable");
    let err = reconcile(&fake, true, Origin::UserChange).expect_err("must fail");
    assert_eq!(err.to_string(), "boom");
}

#[test]
fn a_startup_pass_keeps_an_entry_the_user_switched_off_in_task_manager() {
    let fake = Fake::user_disabled();
    assert_eq!(reconcile(&fake, true, Origin::Startup).expect("reconcile"), Plan::Keep);
    assert_eq!(fake.calls(), ["is_registered"], "a start-up pass only reads the Run value");
    assert!(!fake.approved(), "the user's Task Manager decision survives the start");
}

#[test]
fn the_same_entry_under_a_user_change_is_enabled_again() {
    let fake = Fake::user_disabled();
    assert_eq!(reconcile(&fake, true, Origin::UserChange).expect("reconcile"), Plan::Enable);
    assert_eq!(fake.calls(), ["is_enabled", "enable"]);
    assert!(fake.approved(), "switching the setting on is the user asking for it back");
}

#[test]
fn a_startup_pass_writes_a_run_value_that_is_missing() {
    let fake = Fake::absent();
    assert_eq!(reconcile(&fake, true, Origin::Startup).expect("reconcile"), Plan::Enable);
    assert_eq!(fake.calls(), ["is_registered", "enable"]);
}

#[test]
fn a_startup_pass_removes_a_leftover_value_while_the_setting_is_off() {
    // Also when Task Manager has it off: the value is still there to be switched back on.
    for fake in [Fake::healthy(), Fake::user_disabled()] {
        assert_eq!(reconcile(&fake, false, Origin::Startup).expect("reconcile"), Plan::Disable);
        assert_eq!(fake.calls(), ["is_registered", "disable"]);
        assert!(!fake.registered());
    }
}

#[test]
fn a_startup_pass_with_everything_in_order_writes_nothing() {
    for (wanted, fake) in [(true, Fake::healthy()), (false, Fake::absent())] {
        assert_eq!(reconcile(&fake, wanted, Origin::Startup).expect("reconcile"), Plan::Keep);
        assert_eq!(fake.calls(), ["is_registered"], "wanted {wanted}");
    }
}

#[test]
fn a_late_task_applies_the_latest_request_not_its_own() {
    let fake = Arc::new(Fake::absent());
    let applier = Applier::new(fake.clone());
    applier.request(true, Origin::UserChange);
    applier.request(false, Origin::UserChange);
    // Two queued tasks run in either order; neither may resurrect the superseded "on".
    assert_eq!(applier.apply().expect("first"), Plan::Keep);
    assert_eq!(applier.apply().expect("second"), Plan::Keep);
    assert!(!fake.registered());
    assert!(!fake.calls().contains(&"enable"));
}

#[test]
fn the_applier_follows_a_change_of_mind() {
    let fake = Arc::new(Fake::absent());
    let applier = Applier::new(fake.clone());
    applier.request(true, Origin::UserChange);
    assert_eq!(applier.apply().expect("on"), Plan::Enable);
    applier.request(false, Origin::UserChange);
    assert_eq!(applier.apply().expect("off"), Plan::Disable);
    assert!(!fake.registered());
}

#[test]
fn a_failed_apply_does_not_poison_the_next_one() {
    let fake = Arc::new(Fake::absent().failing("enable"));
    let applier = Applier::new(fake.clone());
    applier.request(true, Origin::UserChange);
    assert!(applier.apply().is_err());
    applier.request(false, Origin::UserChange);
    assert_eq!(applier.apply().expect("recovered"), Plan::Keep);
}

#[test]
fn the_applier_carries_the_origin_of_the_latest_request() {
    let fake = Arc::new(Fake::user_disabled());
    let applier = Applier::new(fake.clone());
    applier.request(true, Origin::Startup);
    assert_eq!(applier.apply().expect("startup"), Plan::Keep);
    applier.request(true, Origin::UserChange);
    assert_eq!(applier.apply().expect("user change"), Plan::Enable);
}

#[test]
fn the_applier_remembers_a_failure_until_an_apply_succeeds() {
    let fake = Arc::new(Fake::absent().failing("enable"));
    let applier = Applier::new(fake.clone());
    assert!(!applier.failed(), "nothing has been tried yet");
    applier.request(true, Origin::UserChange);
    assert!(applier.apply().is_err());
    assert!(applier.failed());
    applier.request(false, Origin::UserChange);
    applier.apply().expect("nothing left to write");
    assert!(!applier.failed());
}
