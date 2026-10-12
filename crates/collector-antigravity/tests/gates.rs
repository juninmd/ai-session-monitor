//! The public entry point honours its gates before it does anything else. A source that is off, or
//! not yet consented to, answers on the first poll without ever suspending, so it cannot have
//! waited on a process scan, a socket, a timer or a file. The consenting path is deliberately not
//! run here: it would read the real process table of the machine running the tests.

use std::future::Future;
use std::pin::{Pin, pin};
use std::task::{Context, Poll, Waker};

use collector_antigravity::{NOTICE_CODES, collect};
use usage_model::settings::AntigravitySettings;
use usage_model::{Ctx, HarnessId, HarnessView, ViewState};

/// Polls once. A future that is ready here never waited for I/O, a timer or a blocking thread.
fn first_poll<F: Future<Output = HarnessView>>(future: F) -> Option<HarnessView> {
    let mut future = pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    match future.as_mut().poll(&mut context) {
        Poll::Ready(view) => Some(view),
        Poll::Pending => None,
    }
}

type Outcome<T> = Result<T, Box<dyn std::error::Error>>;

/// The shape the app shell stores a collector's future in (`app-core`'s `CollectFuture`).
type Boxed<'a> = Pin<Box<dyn Future<Output = HarnessView> + Send + 'a>>;

fn boxed<'a>(settings: &'a AntigravitySettings, ctx: &'a Ctx) -> Boxed<'a> {
    Box::pin(collect(settings, ctx))
}

/// The view of a poll that must end on its first step, in a directory that must stay empty.
fn gated(settings: &AntigravitySettings) -> Outcome<HarnessView> {
    let dir = tempfile::tempdir()?;
    let ctx = Ctx::new(1_791_633_600, dir.path(), dir.path());
    let view = first_poll(boxed(settings, &ctx)).ok_or("a gated poll is ready at once")?;
    assert_eq!(std::fs::read_dir(dir.path())?.count(), 0, "a gated poll writes nothing");
    Ok(view)
}

#[test]
fn a_disabled_source_is_disabled_whatever_the_consent() -> Outcome<()> {
    for consent in [false, true] {
        let view = gated(&AntigravitySettings { enabled: false, consent })?;
        assert_eq!((view.id, view.state), (HarnessId::Antigravity, ViewState::Disabled));
    }
    Ok(())
}

#[test]
fn an_enabled_source_without_consent_asks_for_it() -> Outcome<()> {
    let view = gated(&AntigravitySettings { enabled: true, consent: false })?;
    assert_eq!((view.id, view.state), (HarnessId::Antigravity, ViewState::NeedsConsent));
    Ok(())
}

#[test]
fn the_default_settings_read_nothing_until_the_user_agrees() -> Outcome<()> {
    let view = gated(&AntigravitySettings::default())?;
    assert_eq!(view.state, ViewState::NeedsConsent);
    Ok(())
}

#[test]
fn a_gated_view_carries_no_data_and_no_error() -> Outcome<()> {
    for settings in [
        AntigravitySettings { enabled: false, consent: false },
        AntigravitySettings { enabled: true, consent: false },
    ] {
        let view = gated(&settings)?;
        assert!(view.windows.is_empty() && view.notices.is_empty() && view.facts.is_empty());
        assert!(view.source.is_none() && view.error.is_none() && view.read_at.is_none());
        assert!(view.plan.is_none() && view.meta.is_empty() && !view.stale);
    }
    Ok(())
}

#[test]
fn the_poll_can_be_boxed_and_handed_to_another_thread() {
    // Compiles only if the future is `Send`, as the scheduler needs; the gated poll that runs
    // there touches nothing.
    let settings = AntigravitySettings { enabled: false, consent: true };
    let ctx = Ctx::new(0, "unused", "unused");
    let future = boxed(&settings, &ctx);
    let view = std::thread::scope(|scope| scope.spawn(move || first_poll(future)).join());
    let state = view.ok().flatten().map(|view| view.state);
    assert_eq!(state, Some(ViewState::Disabled));
}

#[test]
fn the_only_notice_is_the_documented_one() {
    assert_eq!(NOTICE_CODES, ["antigravity_pool_unknown"]);
}
