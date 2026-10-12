use super::*;

#[test]
fn rect_contains_treats_the_far_edges_as_outside() {
    let r = Rect::new(10, 20, 100, 50);
    assert!(r.contains(10, 20));
    assert!(r.contains(109, 69));
    assert!(!r.contains(110, 69));
    assert!(!r.contains(109, 70));
    assert!(!r.contains(9, 20));
}

#[test]
fn pick_monitor_prefers_the_one_containing_the_point() {
    let monitors = [Rect::new(0, 0, 1920, 1080), Rect::new(1920, 0, 2560, 1440)];
    assert_eq!(pick_monitor((2000, 100), &monitors), Some(1));
    assert_eq!(pick_monitor((1919, 1079), &monitors), Some(0));
}

#[test]
fn pick_monitor_falls_back_to_the_nearest_one() {
    let monitors = [Rect::new(0, 0, 1920, 1080), Rect::new(1920, 0, 2560, 1440)];
    assert_eq!(pick_monitor((-500, 500), &monitors), Some(0));
    assert_eq!(pick_monitor((9000, 700), &monitors), Some(1));
}

#[test]
fn pick_monitor_without_monitors_is_none() {
    assert_eq!(pick_monitor((0, 0), &[]), None);
}

#[test]
fn height_is_clamped_between_the_minimum_and_the_work_area() {
    let work = Rect::new(0, 0, 1920, 1040);
    assert_eq!(clamp_logical_height(120.0, 1.0, work, MARGIN), MIN_HEIGHT_LOGICAL);
    assert_eq!(clamp_logical_height(480.0, 1.0, work, MARGIN), 480.0);
    assert_eq!(clamp_logical_height(5000.0, 1.0, work, MARGIN), f64::from(1040 - 2 * MARGIN));
}

#[test]
fn height_ceiling_follows_the_scale_factor() {
    // 1560 physical px of work height at 150 % is 1040 logical px.
    let work = Rect::new(0, 0, 2880, 1560);
    assert_eq!(clamp_logical_height(5000.0, 1.5, work, 12), f64::from(1560 - 24) / 1.5);
}

#[test]
fn height_that_is_not_a_number_falls_back_to_the_minimum() {
    let work = Rect::new(0, 0, 1920, 1040);
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -10.0] {
        assert_eq!(clamp_logical_height(bad, 1.0, work, MARGIN), MIN_HEIGHT_LOGICAL, "{bad}");
    }
}

#[test]
fn a_tiny_work_area_wins_over_the_minimum_height() {
    let work = Rect::new(0, 0, 800, 220);
    assert_eq!(clamp_logical_height(500.0, 1.0, work, MARGIN), 204.0);
    let sliver = Rect::new(0, 0, 800, 10);
    assert!(clamp_logical_height(500.0, 1.0, sliver, MARGIN) >= 1.0);
}

#[test]
fn a_broken_scale_factor_is_treated_as_one() {
    let work = Rect::new(0, 0, 1920, 1040);
    assert_eq!(clamp_logical_height(5000.0, 0.0, work, MARGIN), 1024.0);
    assert_eq!(clamp_logical_height(5000.0, f64::NAN, work, MARGIN), 1024.0);
}

#[test]
fn the_invisible_frame_is_subtracted_so_the_client_lands_on_target() {
    // Measured on Windows 11: the rect starts 8 px left of and 1 px above the client area.
    let corrected = compensate_frame(Point { x: 1866, y: 981 }, (1858, 981), (1866, 982));
    assert_eq!(corrected, Point { x: 1858, y: 980 });
}

#[test]
fn a_window_without_a_frame_is_not_moved() {
    let target = Point { x: -1920, y: 40 };
    assert_eq!(compensate_frame(target, (7, 9), (7, 9)), target);
}

#[test]
fn a_title_bar_offset_is_compensated_too() {
    assert_eq!(
        compensate_frame(Point { x: 100, y: 100 }, (92, 69), (100, 100)),
        Point { x: 92, y: 69 }
    );
}

/// A window whose first resize misses by `miss` px of height, as tao's does before the first
/// frame update (measured: asked 560, got 590).
struct Window {
    size: std::cell::Cell<(u32, u32)>,
    sets: std::cell::Cell<usize>,
    miss: u32,
    stubborn: bool,
}

impl Window {
    fn new(miss: u32, stubborn: bool) -> Self {
        Self { size: (0, 0).into(), sets: 0.into(), miss, stubborn }
    }

    fn settle(&self, want: (u32, u32), attempts: usize) -> Result<bool, &'static str> {
        settle_size(
            want,
            attempts,
            || Ok(self.size.get()),
            || {
                let first = self.sets.get() == 0;
                self.sets.set(self.sets.get() + 1);
                let off = if first || self.stubborn { self.miss } else { 0 };
                self.size.set((want.0, want.1 + off));
                Ok(())
            },
        )
    }
}

#[test]
fn a_first_resize_that_misses_by_a_title_bar_is_repeated_until_it_holds() {
    let window = Window::new(30, false);
    assert_eq!(window.settle((432, 560), 3), Ok(true));
    assert_eq!(window.size.get(), (432, 560));
    assert_eq!(window.sets.get(), 2);
}

#[test]
fn a_size_that_holds_at_once_is_set_only_once() {
    let window = Window::new(0, false);
    assert_eq!(window.settle((432, 560), 3), Ok(true));
    assert_eq!(window.sets.get(), 1);
}

#[test]
fn a_size_that_never_holds_stops_after_the_attempts_and_says_so() {
    let window = Window::new(30, true);
    assert_eq!(window.settle((432, 560), 3), Ok(false));
    assert_eq!(window.sets.get(), 3);
}

#[test]
fn a_failing_resize_is_reported_without_another_try() {
    let mut calls = 0;
    let outcome = settle_size(
        (1, 1),
        3,
        || Ok((0, 0)),
        || {
            calls += 1;
            Err("window is gone")
        },
    );
    assert_eq!(outcome, Err("window is gone"));
    assert_eq!(calls, 1);
}

#[test]
fn extreme_coordinates_saturate_instead_of_wrapping() {
    let floor = Point { x: i32::MIN, y: i32::MIN };
    assert_eq!(compensate_frame(floor, (0, 0), (8, 8)), floor);
    let ceiling = Point { x: i32::MAX, y: i32::MAX };
    assert_eq!(compensate_frame(ceiling, (8, 8), (0, 0)), ceiling);
}
