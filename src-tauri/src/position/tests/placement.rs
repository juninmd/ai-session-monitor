use super::*;

const WIN: (i32, i32) = (432, 560);

/// 1920x1080 monitor with a 40 px taskbar at the bottom.
fn bottom_bar() -> (Rect, Rect) {
    (Rect::new(0, 0, 1920, 1080), Rect::new(0, 0, 1920, 1040))
}

fn icon(x: i32, y: i32) -> Option<Rect> {
    Some(Rect::new(x, y, 24, 24))
}

fn place_at(monitor: Rect, work: Rect, tray: Option<Rect>) -> Point {
    place(&Anchor { tray, monitor, work }, WIN, MARGIN)
}

#[test]
fn bottom_taskbar_centres_the_window_on_the_icon_above_the_work_area() {
    let (monitor, work) = bottom_bar();
    let at = place_at(monitor, work, icon(1500, 1048));
    assert_eq!(at, Point { x: 1512 - 216, y: 1040 - 560 - MARGIN });
}

#[test]
fn icon_near_the_right_edge_pins_the_window_to_the_margin() {
    let (monitor, work) = bottom_bar();
    let at = place_at(monitor, work, icon(1890, 1048));
    assert_eq!(at.x, 1920 - 432 - MARGIN);
}

#[test]
fn icon_near_the_left_edge_pins_the_window_to_the_margin() {
    let (monitor, work) = bottom_bar();
    let at = place_at(monitor, work, icon(0, 1048));
    assert_eq!(at.x, MARGIN);
}

#[test]
fn top_taskbar_hangs_the_window_below_it() {
    let monitor = Rect::new(0, 0, 1920, 1080);
    let work = Rect::new(0, 40, 1920, 1040);
    let at = place_at(monitor, work, icon(1500, 8));
    assert_eq!(at, Point { x: 1512 - 216, y: 40 + MARGIN });
}

#[test]
fn left_taskbar_puts_the_window_beside_it_centred_on_the_icon() {
    let monitor = Rect::new(0, 0, 1920, 1080);
    let work = Rect::new(60, 0, 1860, 1080);
    let at = place_at(monitor, work, icon(16, 300));
    assert_eq!(at, Point { x: 60 + MARGIN, y: 312 - 280 });
}

#[test]
fn right_taskbar_puts_the_window_beside_it() {
    let monitor = Rect::new(0, 0, 1920, 1080);
    let work = Rect::new(0, 0, 1860, 1080);
    let at = place_at(monitor, work, icon(1880, 300));
    assert_eq!(at, Point { x: 1860 - 432 - MARGIN, y: 312 - 280 });
}

#[test]
fn vertical_taskbar_clamps_an_icon_near_the_bottom() {
    let monitor = Rect::new(0, 0, 1920, 1080);
    let work = Rect::new(60, 0, 1860, 1080);
    let at = place_at(monitor, work, icon(16, 1040));
    assert_eq!(at.y, 1080 - 560 - MARGIN);
}

#[test]
fn monitor_left_of_the_primary_uses_negative_coordinates() {
    let monitor = Rect::new(-1920, 0, 1920, 1080);
    let work = Rect::new(-1920, 0, 1920, 1040);
    let at = place_at(monitor, work, icon(-200, 1048));
    assert_eq!(at, Point { x: 0 - 432 - MARGIN, y: 1040 - 560 - MARGIN });
}

#[test]
fn monitor_above_the_primary_uses_negative_y() {
    let monitor = Rect::new(0, -1080, 1920, 1080);
    let work = Rect::new(0, -1080, 1920, 1040);
    let at = place_at(monitor, work, icon(1500, -32));
    assert_eq!(at.y, -40 - 560 - MARGIN);
}

#[test]
fn scaled_display_works_in_physical_pixels() {
    // 150 %: 2880x1620 monitor, 60 px taskbar, 648x840 window, 12 px margin.
    let monitor = Rect::new(0, 0, 2880, 1620);
    let work = Rect::new(0, 0, 2880, 1560);
    let anchor = Anchor { tray: icon(2700, 1572), monitor, work };
    let at = place(&anchor, (648, 840), 12);
    assert_eq!(at, Point { x: 2880 - 648 - 12, y: 1560 - 840 - 12 });
}

#[test]
fn window_taller_than_the_work_area_keeps_its_top_margin() {
    let (monitor, work) = bottom_bar();
    let anchor = Anchor { tray: icon(1500, 1048), monitor, work };
    assert_eq!(place(&anchor, (432, 2000), MARGIN).y, MARGIN);
}

#[test]
fn window_wider_than_the_work_area_keeps_its_left_margin() {
    let (monitor, work) = bottom_bar();
    let anchor = Anchor { tray: icon(1500, 1048), monitor, work };
    assert_eq!(place(&anchor, (3000, 560), MARGIN).x, MARGIN);
}

#[test]
fn unknown_icon_falls_back_to_the_notification_corner() {
    let (monitor, work) = bottom_bar();
    let at = place_at(monitor, work, None);
    assert_eq!(at, Point { x: 1920 - 432 - MARGIN, y: 1040 - 560 - MARGIN });
}

#[test]
fn an_empty_icon_rect_places_like_an_unknown_icon() {
    let (monitor, work) = bottom_bar();
    let empty_rects = [
        Rect::new(0, 0, 0, 0),
        Rect::new(1500, 1048, 0, 24),
        Rect::new(1500, 1048, 24, 0),
        Rect::new(1500, 1048, -24, 24),
        Rect::new(-32000, -32000, 0, 0),
    ];
    for empty in empty_rects {
        let docked = place_at(monitor, work, usable_tray(Some(empty)));
        assert_eq!(docked, place_at(monitor, work, None), "{empty:?}");
        // With an auto-hide taskbar nothing is reserved, so the icon alone picks the edge.
        let hidden = place_at(monitor, monitor, usable_tray(Some(empty)));
        assert_eq!(hidden, place_at(monitor, monitor, None), "{empty:?} with an auto-hide bar");
    }
    assert_eq!(usable_tray(icon(1500, 1048)), icon(1500, 1048));
    assert_eq!(usable_tray(None), None);
}

#[test]
fn icon_in_the_overflow_popup_still_anchors_to_the_taskbar_edge() {
    let (monitor, work) = bottom_bar();
    let popup = place_at(monitor, work, icon(1500, 900));
    let docked = place_at(monitor, work, icon(1500, 1048));
    assert_eq!(popup.y, docked.y);
}

#[test]
fn hidden_taskbar_at_the_bottom_anchors_to_the_icon() {
    let monitor = Rect::new(0, 0, 1920, 1080);
    let at = place_at(monitor, monitor, icon(1700, 1056));
    assert_eq!(taskbar_edge(monitor, monitor, icon(1700, 1056)), Edge::Bottom);
    assert_eq!(at.y, 1056 - 560 - MARGIN);
}

#[test]
fn hidden_taskbar_at_the_top_anchors_below_the_icon() {
    let monitor = Rect::new(0, 0, 1920, 1080);
    let tray = icon(1700, 0);
    assert_eq!(taskbar_edge(monitor, monitor, tray), Edge::Top);
    assert_eq!(place_at(monitor, monitor, tray).y, 24 + MARGIN);
}

#[test]
fn hidden_taskbar_on_the_left_anchors_beside_the_icon() {
    let monitor = Rect::new(0, 0, 1920, 1080);
    let tray = icon(0, 500);
    assert_eq!(taskbar_edge(monitor, monitor, tray), Edge::Left);
    assert_eq!(place_at(monitor, monitor, tray).x, 24 + MARGIN);
}

#[test]
fn hidden_taskbar_on_the_right_anchors_beside_the_icon() {
    let monitor = Rect::new(0, 0, 1920, 1080);
    let tray = icon(1896, 500);
    assert_eq!(taskbar_edge(monitor, monitor, tray), Edge::Right);
    assert_eq!(place_at(monitor, monitor, tray).x, 1896 - 432 - MARGIN);
}

#[test]
fn the_widest_reserved_band_decides_the_edge() {
    let monitor = Rect::new(0, 0, 1920, 1080);
    let work = Rect::new(0, 0, 1900, 1040);
    assert_eq!(taskbar_edge(monitor, work, None), Edge::Bottom);
    let work = Rect::new(0, 0, 1860, 1070);
    assert_eq!(taskbar_edge(monitor, work, None), Edge::Right);
}

#[test]
fn nothing_reserved_and_no_icon_defaults_to_the_bottom() {
    let monitor = Rect::new(0, 0, 1920, 1080);
    assert_eq!(taskbar_edge(monitor, monitor, None), Edge::Bottom);
}

#[test]
fn placement_stays_inside_the_work_area_for_every_edge_and_icon_position() {
    let monitor = Rect::new(-1280, 200, 1280, 1024);
    let layouts = [
        Rect::new(-1280, 200, 1280, 984),  // bottom
        Rect::new(-1280, 240, 1280, 984),  // top
        Rect::new(-1220, 200, 1220, 1024), // left
        Rect::new(-1280, 200, 1220, 1024), // right
    ];
    for work in layouts {
        for step in 0..=20 {
            let (x, y) = (monitor.x + step * 64, monitor.y + step * 51);
            for tray in [icon(x, y), icon(monitor.x, y), icon(x, monitor.bottom() - 30), None] {
                let at = place_at(monitor, work, tray);
                let case = format!("work {work:?} tray {tray:?} -> {at:?}");
                assert!(at.x >= work.x + MARGIN && at.x + WIN.0 <= work.right() - MARGIN, "{case}");
                assert!(
                    at.y >= work.y + MARGIN && at.y + WIN.1 <= work.bottom() - MARGIN,
                    "{case}"
                );
            }
        }
    }
}
