use super::*;

fn at(base: Instant, ms: u64) -> Instant {
    base + Duration::from_millis(ms)
}

#[test]
fn a_blur_right_after_opening_is_ignored() {
    let t0 = Instant::now();
    let mut gate = Gate::default();
    gate.shown(t0);
    assert!(!gate.blurred(at(t0, 100), true));
    assert!(gate.blurred(at(t0, 300), true));
}

#[test]
fn a_blur_of_a_hidden_flyout_is_ignored_and_not_remembered() {
    let t0 = Instant::now();
    let mut gate = Gate::default();
    assert!(!gate.blurred(t0, false));
    gate.press(at(t0, 10), false);
    assert_eq!(gate.release(at(t0, 20), false), ClickAction::Show);
}

#[test]
fn a_plain_click_opens_a_hidden_flyout() {
    let t0 = Instant::now();
    let mut gate = Gate::default();
    gate.press(t0, false);
    assert_eq!(gate.release(at(t0, 90), false), ClickAction::Show);
}

#[test]
fn a_click_on_a_focused_flyout_closes_it_without_reopening() {
    let t0 = Instant::now();
    let mut gate = Gate::default();
    gate.shown(t0);
    gate.press(at(t0, 1000), true);
    assert!(gate.blurred(at(t0, 1010), true));
    assert_eq!(gate.release(at(t0, 1100), false), ClickAction::Ignore);
}

#[test]
fn the_blur_may_arrive_before_the_press_is_seen() {
    let t0 = Instant::now();
    let mut gate = Gate::default();
    gate.shown(t0);
    assert!(gate.blurred(at(t0, 1000), true));
    gate.press(at(t0, 1020), false);
    assert_eq!(gate.release(at(t0, 1120), false), ClickAction::Ignore);
}

#[test]
fn a_slow_closing_click_still_does_not_reopen() {
    let t0 = Instant::now();
    let mut gate = Gate::default();
    gate.shown(t0);
    gate.press(at(t0, 1000), true);
    assert!(gate.blurred(at(t0, 1005), true));
    assert_eq!(gate.release(at(t0, 1800), false), ClickAction::Ignore);
}

#[test]
fn a_click_that_left_the_focus_alone_hides_it() {
    let t0 = Instant::now();
    let mut gate = Gate::default();
    gate.shown(t0);
    gate.press(at(t0, 1000), true);
    assert_eq!(gate.release(at(t0, 1080), true), ClickAction::Hide);
}

#[test]
fn a_second_click_right_after_a_closing_click_opens_again() {
    let t0 = Instant::now();
    let mut gate = Gate::default();
    gate.shown(t0);
    gate.press(at(t0, 1000), true);
    assert!(gate.blurred(at(t0, 1005), true));
    assert_eq!(gate.release(at(t0, 1060), false), ClickAction::Ignore);
    gate.press(at(t0, 1120), false);
    assert_eq!(gate.release(at(t0, 1200), false), ClickAction::Show);
}

#[test]
fn an_old_blur_does_not_suppress_a_later_click() {
    let t0 = Instant::now();
    let mut gate = Gate::default();
    gate.shown(t0);
    assert!(gate.blurred(at(t0, 1000), true));
    gate.press(at(t0, 5000), false);
    assert_eq!(gate.release(at(t0, 5080), false), ClickAction::Show);
}

#[test]
fn a_release_without_a_press_falls_back_to_the_visible_state() {
    let t0 = Instant::now();
    let mut gate = Gate::default();
    assert_eq!(gate.release(t0, true), ClickAction::Hide);
    assert_eq!(gate.release(at(t0, 10), false), ClickAction::Show);
}

#[test]
fn opening_clears_a_pending_blur_record() {
    let t0 = Instant::now();
    let mut gate = Gate::default();
    gate.shown(t0);
    assert!(gate.blurred(at(t0, 1000), true));
    gate.shown(at(t0, 1100));
    gate.press(at(t0, 1150), false);
    assert_eq!(gate.release(at(t0, 1200), false), ClickAction::Show);
}

#[test]
fn a_slow_click_that_follows_the_blur_still_does_not_reopen() {
    let t0 = Instant::now();
    let mut gate = Gate::default();
    gate.shown(t0);
    assert!(gate.blurred(at(t0, 1000), true));
    // Blur first (the Windows order), release past the guard: only the press remembers the blur.
    gate.press(at(t0, 1020), false);
    assert_eq!(gate.release(at(t0, 1500), false), ClickAction::Ignore);
}
