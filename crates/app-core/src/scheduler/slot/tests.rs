use tokio::task::JoinHandle;
use usage_model::{Ctx, ErrorCode, HarnessError, Settings};

use super::*;

const SIXTY: Duration = Duration::from_secs(60);

const fn secs(n: u64) -> Duration {
    Duration::from_secs(n)
}

fn slot(enabled: bool) -> Slot {
    Slot::new(HarnessId::Claude, None, enabled)
}

/// An enabled slot whose first call is due at `now`, as `Scheduler::run` arms it.
fn armed(now: Instant) -> Slot {
    let mut slot = slot(true);
    slot.due = Some(now);
    slot
}

fn failing(retryable: bool) -> Verdict {
    let error = HarnessError::new(ErrorCode::Network, retryable);
    judge(&HarnessView::error(HarnessId::Claude, error))
}

fn running() -> (AbortHandle, JoinHandle<()>) {
    let task = tokio::spawn(std::future::pending::<()>());
    (task.abort_handle(), task)
}

fn delay(slot: &Slot, from: Instant) -> u64 {
    slot.due.expect("scheduled").duration_since(from).as_secs()
}

#[test]
fn every_view_state_maps_to_one_verdict() {
    let table = [
        (ViewState::Loading, Verdict::Healthy),
        (ViewState::Ready, Verdict::Healthy),
        (ViewState::NoData, Verdict::Healthy),
        (ViewState::NeedsConsent, Verdict::Parked),
        (ViewState::NeedsSetup, Verdict::Parked),
        (ViewState::Disabled, Verdict::Parked),
        (ViewState::Error, Verdict::Failed { retryable: true }),
    ];
    for (state, expected) in table {
        assert_eq!(judge(&HarnessView::with_state(HarnessId::Codex, state)), expected, "{state:?}");
    }
}

#[test]
fn the_error_decides_whether_a_failure_is_retryable() {
    assert_eq!(failing(true), Verdict::Failed { retryable: true });
    assert_eq!(failing(false), Verdict::Failed { retryable: false });
}

#[test]
fn only_an_armed_idle_enabled_slot_is_due() {
    let now = Instant::now();
    assert!(armed(now).is_due(now));
    assert!(!armed(now).is_due(now - secs(1)), "not before its time");
    assert!(!slot(true).is_due(now), "nothing scheduled");
    let mut off = armed(now);
    off.enabled = false;
    assert!(!off.is_due(now), "a disabled harness is never started");
}

#[tokio::test]
async fn a_slot_with_a_call_in_flight_is_neither_due_nor_refreshable() {
    let now = Instant::now();
    // With a collector: without one `may_refresh` is false for another reason.
    let mut busy = Slot::new(HarnessId::Claude, Some(stub()), true);
    busy.due = Some(now);
    assert!(busy.is_due(now) && busy.may_refresh(now + SIXTY), "control: idle, it is both");

    let (handle, task) = running();
    busy.running = Some(handle);
    assert!(!busy.is_due(now));
    assert!(!busy.may_refresh(now + SIXTY));
    task.abort();
}

#[test]
fn success_schedules_the_next_call_one_cadence_after_it_ended() {
    let now = Instant::now();
    let mut slot = armed(now);
    slot.settle(Verdict::Healthy, SIXTY, now);
    assert_eq!((slot.failures, delay(&slot, now)), (0, 60));
    assert!(!slot.is_due(now + SIXTY - Duration::from_millis(1)));
    assert!(slot.is_due(now + SIXTY));
}

#[test]
fn failures_back_off_exponentially_and_one_success_resets_them() {
    let now = Instant::now();
    let mut slot = armed(now);
    let mut delays = Vec::new();
    for _ in 0..6 {
        slot.settle(failing(true), SIXTY, now);
        delays.push(delay(&slot, now));
    }
    assert_eq!(delays, [120, 240, 480, 900, 900, 900]);
    assert_eq!(slot.failures, 6);

    slot.settle(Verdict::Healthy, SIXTY, now);
    assert_eq!((slot.failures, delay(&slot, now)), (0, 60));
    slot.settle(failing(true), SIXTY, now);
    assert_eq!(delay(&slot, now), 120, "the progression starts over");
}

#[test]
fn an_error_that_retrying_cannot_fix_waits_the_maximum_at_once() {
    let now = Instant::now();
    let mut slot = armed(now);
    slot.settle(failing(false), SIXTY, now);
    assert_eq!(delay(&slot, now), 900);
    let mut slow = armed(now);
    slow.settle(failing(false), secs(3600), now);
    assert_eq!(delay(&slow, now), 3600, "never faster than the normal cadence");
}

#[test]
fn a_parked_result_stops_polling_and_forgets_the_failures() {
    let now = Instant::now();
    let mut slot = armed(now);
    slot.settle(failing(true), SIXTY, now);
    slot.settle(Verdict::Parked, SIXTY, now);
    assert_eq!((slot.due, slot.failures), (None, 0));
    assert!(!slot.is_due(now + secs(86_400)), "a parked harness is never polled");
}

#[tokio::test]
async fn settling_clears_the_in_flight_marker() {
    let now = Instant::now();
    let mut slot = armed(now);
    let (handle, task) = running();
    slot.running = Some(handle);
    slot.settle(Verdict::Healthy, SIXTY, now);
    assert!(slot.running.is_none());
    task.abort();
}

#[test]
fn a_refresh_is_honoured_only_ten_seconds_after_the_last_start() {
    let now = Instant::now();
    let mut slot = Slot::new(HarnessId::Claude, Some(stub()), true);
    assert!(slot.may_refresh(now), "never started: fresh install");
    slot.last_start = Some(now);
    assert!(!slot.may_refresh(now));
    assert!(!slot.may_refresh(now + secs(10) - Duration::from_millis(1)));
    assert!(slot.may_refresh(now + secs(10)));
}

#[test]
fn a_refresh_is_refused_for_a_slot_that_is_off_or_has_no_collector() {
    let now = Instant::now();
    assert!(!Slot::new(HarnessId::Claude, Some(stub()), false).may_refresh(now));
    assert!(!slot(true).may_refresh(now), "no collector to ask");
}

#[test]
fn a_new_interval_moves_the_pending_call_of_a_healthy_slot_only() {
    let start = Instant::now();
    let mut healthy = armed(start);
    healthy.last_start = Some(start);
    healthy.settle(Verdict::Healthy, SIXTY, start);
    healthy.retime(SIXTY, secs(30), start + secs(5));
    assert_eq!(delay(&healthy, start), 30, "counted from the last start");
    healthy.retime(secs(30), secs(3600), start + secs(5));
    assert_eq!(delay(&healthy, start), 3600);
    healthy.retime(secs(3600), secs(60), start + secs(100));
    assert_eq!(delay(&healthy, start), 100, "already overdue: due now, not in the past");

    let mut backing_off = armed(start);
    backing_off.last_start = Some(start);
    backing_off.settle(failing(true), SIXTY, start);
    let due = backing_off.due;
    backing_off.retime(SIXTY, secs(30), start);
    assert_eq!(backing_off.due, due, "a backoff is not shortened by a settings change");

    let mut parked = armed(start);
    parked.last_start = Some(start);
    parked.settle(Verdict::Parked, SIXTY, start);
    parked.retime(SIXTY, secs(30), start);
    assert_eq!(parked.due, None);

    let mut unchanged = armed(start);
    unchanged.last_start = Some(start);
    unchanged.settle(Verdict::Healthy, SIXTY, start + secs(1));
    unchanged.retime(SIXTY, SIXTY, start + secs(2));
    assert_eq!(delay(&unchanged, start), 61, "same cadence: untouched");
}

/// A collector that is never called: the slot only needs to know one exists.
fn stub() -> Arc<dyn Collector> {
    use crate::collectors::CollectFuture;
    use crate::secrets::SecretStore;

    struct Idle;
    impl Collector for Idle {
        fn id(&self) -> HarnessId {
            HarnessId::Claude
        }

        fn collect<'a>(
            &'a self,
            _settings: &'a Settings,
            _secrets: &'a dyn SecretStore,
            _ctx: &'a Ctx,
        ) -> CollectFuture<'a> {
            Box::pin(std::future::pending())
        }
    }
    Arc::new(Idle)
}
