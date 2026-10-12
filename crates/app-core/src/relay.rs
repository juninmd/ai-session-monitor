//! Delivers "something changed" to a sink from any thread without holding a lock during the call,
//! and without letting deliveries overlap or finish out of order.

use std::sync::Mutex;

use crate::sync::lock;

#[derive(Default)]
struct State {
    dirty: bool,
    draining: bool,
}

#[derive(Default)]
pub(crate) struct Relay {
    state: Mutex<State>,
}

impl Relay {
    /// Guarantees a `deliver` that starts after this call. A caller that arrives while another
    /// thread is delivering leaves the work to it: that thread delivers once more before it
    /// stops, so the last delivery always runs after the last change. `deliver` must read the
    /// state it reports at call time, not capture it.
    pub(crate) fn trigger(&self, mut deliver: impl FnMut()) {
        {
            let mut state = lock(&self.state);
            state.dirty = true;
            if state.draining {
                return;
            }
            state.draining = true;
        }
        let mut guard = Draining { relay: self, armed: true };
        loop {
            {
                let mut state = lock(&self.state);
                if !state.dirty {
                    state.draining = false;
                    guard.armed = false;
                    return;
                }
                state.dirty = false;
            }
            deliver();
        }
    }
}

/// A panicking sink must not leave the relay stuck in the draining state.
struct Draining<'a> {
    relay: &'a Relay,
    armed: bool,
}

impl Drop for Draining<'_> {
    fn drop(&mut self) {
        if self.armed {
            lock(&self.relay.state).draining = false;
        }
    }
}

#[cfg(test)]
mod tests;
