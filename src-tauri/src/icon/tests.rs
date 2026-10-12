use super::*;

fn gauge(pct: f64, status: Status) -> Gauge {
    Gauge { available_pct: Some(pct), status: Some(status) }
}

fn unknown() -> Gauge {
    Gauge { available_pct: None, status: None }
}

fn pixel(img: &Rgba, x: u32, y: u32) -> [u8; 4] {
    let at = ((y * img.size + x) * 4) as usize;
    [img.pixels[at], img.pixels[at + 1], img.pixels[at + 2], img.pixels[at + 3]]
}

/// The pixel on the ring's centre line at `turn` (fraction of a clockwise turn from 12 o'clock).
fn ring_pixel(img: &Rgba, turn: f64) -> [u8; 4] {
    let (outer, inner) = radii(img.size);
    let mid = (outer + inner) / 2.0;
    let centre = f64::from(img.size) / 2.0;
    let angle = turn * TAU;
    let (x, y) = (centre + mid * angle.sin(), centre - mid * angle.cos());
    pixel(img, x.floor() as u32, y.floor() as u32)
}

fn opaque(rgb: [u8; 3]) -> [u8; 4] {
    [rgb[0], rgb[1], rgb[2], 255]
}

fn visible(img: &Rgba) -> usize {
    img.pixels.chunks_exact(4).filter(|p| p[3] > 0).count()
}

fn arc_pixels(img: &Rgba, rgb: [u8; 3]) -> usize {
    img.pixels.chunks_exact(4).filter(|p| p[3] == 255 && p[..3] == rgb).count()
}

#[test]
fn icon_size_follows_the_display_scale() {
    let cases = [(1.0, 16), (1.25, 24), (1.5, 24), (1.75, 32), (2.0, 32), (3.0, 32)];
    for (scale, expected) in cases {
        assert_eq!(icon_size(scale), expected, "scale {scale}");
    }
}

#[test]
fn icon_size_ignores_a_broken_scale() {
    for scale in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert_eq!(icon_size(scale), 16, "scale {scale}");
    }
}

#[test]
fn pixel_buffer_matches_the_requested_size() {
    for size in SIZES {
        let img = render(gauge(50.0, Status::Ok), size);
        assert_eq!(img.size, size);
        assert_eq!(img.pixels.len(), (size * size * 4) as usize);
    }
}

#[test]
fn corners_and_centre_stay_transparent() {
    for size in SIZES {
        let img = render(gauge(100.0, Status::Ok), size);
        let mid = size / 2;
        for (x, y) in [(0, 0), (size - 1, 0), (0, size - 1), (size - 1, size - 1), (mid, mid)] {
            assert_eq!(pixel(&img, x, y)[3], 0, "size {size} pixel {x},{y}");
        }
    }
}

#[test]
fn full_ring_is_the_accent_colour_all_around() {
    let img = render(gauge(100.0, Status::Ok), 32);
    for turn in [0.1, 0.3, 0.6, 0.9] {
        assert_eq!(ring_pixel(&img, turn), opaque(OK), "turn {turn}");
    }
}

#[test]
fn arc_sweeps_clockwise_from_twelve_o_clock() {
    let img = render(gauge(50.0, Status::Ok), 32);
    for turn in [0.125, 0.25, 0.375] {
        assert_eq!(ring_pixel(&img, turn), opaque(OK), "arc at {turn}");
    }
    for turn in [0.625, 0.75, 0.875] {
        let [r, g, b, a] = ring_pixel(&img, turn);
        assert_eq!([r, g, b], TRACK, "track at {turn}");
        assert!(a.abs_diff(140) <= 2, "track alpha {a} at {turn}");
    }
}

#[test]
fn quarter_arc_leaves_the_left_side_to_the_track() {
    let img = render(gauge(25.0, Status::Ok), 24);
    assert_eq!(ring_pixel(&img, 0.125), opaque(OK));
    assert_eq!(ring_pixel(&img, 0.75)[..3], TRACK);
    assert_eq!(ring_pixel(&img, 0.5)[..3], TRACK);
}

#[test]
fn status_picks_the_arc_colour() {
    let cases = [(Status::Ok, OK), (Status::Warn, WARN), (Status::Crit, CRIT)];
    for (status, rgb) in cases {
        let img = render(gauge(60.0, status), 32);
        assert_eq!(ring_pixel(&img, 0.25), opaque(rgb), "{status:?}");
    }
}

#[test]
fn exhausted_ring_is_critical_everywhere_and_has_no_arc() {
    let img = render(gauge(0.0, Status::Out), 32);
    assert!(visible(&img) > 0);
    for p in img.pixels.chunks_exact(4).filter(|p| p[3] > 0) {
        assert_eq!(p[..3], CRIT);
    }
    let a = ring_pixel(&img, 0.25)[3];
    assert!((200..255).contains(&a), "ring alpha {a}");
}

#[test]
fn unknown_reading_is_a_dashed_neutral_ring() {
    let dashed = render(unknown(), 32);
    let solid = render(gauge(0.0, Status::Ok), 32);
    assert!(visible(&dashed) > 0);
    assert!(visible(&dashed) < visible(&solid));
    for p in dashed.pixels.chunks_exact(4).filter(|p| p[3] > 0) {
        assert_eq!(p[..3], TRACK);
    }
    assert_eq!(arc_pixels(&dashed, OK), 0);
}

#[test]
fn arc_length_grows_with_the_percentage() {
    let counts: Vec<usize> = [10.0, 50.0, 90.0]
        .iter()
        .map(|pct| arc_pixels(&render(gauge(*pct, Status::Ok), 32), OK))
        .collect();
    assert!(counts[0] < counts[1] && counts[1] < counts[2], "{counts:?}");
}

#[test]
fn out_of_range_percentages_are_clamped() {
    assert_eq!(render(gauge(150.0, Status::Ok), 24), render(gauge(100.0, Status::Ok), 24));
    assert_eq!(render(gauge(-5.0, Status::Ok), 24), render(gauge(0.0, Status::Ok), 24));
}

#[test]
fn a_non_finite_percentage_reads_as_unknown() {
    let nan = Gauge { available_pct: Some(f64::NAN), status: Some(Status::Ok) };
    assert_eq!(render(nan, 24), render(unknown(), 24));
}

#[test]
fn rendering_is_deterministic() {
    let g = gauge(37.0, Status::Warn);
    assert_eq!(render(g, 16), render(g, 16));
}

#[test]
fn every_size_draws_a_ring_with_a_hole() {
    for size in SIZES {
        let img = render(gauge(70.0, Status::Ok), size);
        let (outer, inner) = radii(size);
        let centre = f64::from(size) / 2.0;
        for y in 0..size {
            for x in 0..size {
                let distance = (f64::from(x) + 0.5 - centre).hypot(f64::from(y) + 0.5 - centre);
                let alpha = img.pixels[((y * size + x) * 4 + 3) as usize];
                // One pixel of slack on each side for the anti-aliased edge.
                if distance < inner - 1.0 || distance > outer + 1.0 {
                    assert_eq!(alpha, 0, "size {size}: ({x},{y}) is outside the ring");
                }
            }
        }
        assert!(visible(&img) > size as usize, "size {size} drew almost nothing");
    }
}
