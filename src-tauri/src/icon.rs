//! The tray ring gauge as RGBA pixels. Pure: the size follows the display scale, the colour the
//! status, the sweep the percentage still available.

use std::f64::consts::TAU;

use usage_model::Status;

#[cfg(test)]
mod tests;

/// Sizes the shell can scale cleanly to its icon slot (16 px at 100 %).
const SIZES: [u32; 3] = [16, 24, 32];
/// Samples per pixel along each axis; enough to soften the ring edges at 16 px.
const SUPERSAMPLE: u32 = 4;
/// Dashes around the ring of an unknown reading.
const DASHES: f64 = 16.0;

// Mid-tones that stay readable on both the light and the dark taskbar.
const TRACK: [u8; 3] = [0x8a, 0x93, 0xa0];
const OK: [u8; 3] = [0x5a, 0x9b, 0xec];
const WARN: [u8; 3] = [0xfa, 0xb2, 0x19];
const CRIT: [u8; 3] = [0xef, 0x6b, 0x61];

/// What the ring shows. `available_pct` is `None` while no reading backs the tray.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gauge {
    pub available_pct: Option<f64>,
    pub status: Option<Status>,
}

/// Straight (non-premultiplied) RGBA, row-major.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rgba {
    pub size: u32,
    pub pixels: Vec<u8>,
}

/// Smallest supported size that is not smaller than the slot the shell draws (16 px per 100 %).
pub fn icon_size(scale: f64) -> u32 {
    let scale = if scale.is_finite() && scale > 0.0 { scale } else { 1.0 };
    let wanted = 16.0 * scale;
    SIZES.iter().copied().find(|size| f64::from(*size) >= wanted - 1e-6).unwrap_or(SIZES[2])
}

/// Outer and inner radius of the ring, in pixels.
fn radii(size: u32) -> (f64, f64) {
    let outer = f64::from(size) / 2.0 - 0.5;
    let width = (f64::from(size) * 0.19).max(2.0);
    (outer, outer - width)
}

struct Paint {
    track: [u8; 3],
    track_alpha: f64,
    arc: [u8; 3],
    /// Fraction of the ring (clockwise from 12 o'clock) drawn in the arc colour.
    sweep: f64,
    dashed: bool,
}

fn paint_for(gauge: Gauge) -> Paint {
    let pct = gauge.available_pct.filter(|pct| pct.is_finite()).map(|pct| pct.clamp(0.0, 100.0));
    let accent = match gauge.status {
        Some(Status::Warn) => WARN,
        Some(Status::Crit | Status::Out) => CRIT,
        Some(Status::Ok) | None => OK,
    };
    match (pct, gauge.status) {
        (None, _) => Paint { track: TRACK, track_alpha: 0.55, arc: OK, sweep: 0.0, dashed: true },
        // Nothing left: the whole ring turns to the critical colour instead of an arc.
        (Some(_), Some(Status::Out)) => {
            Paint { track: CRIT, track_alpha: 0.9, arc: CRIT, sweep: 0.0, dashed: false }
        }
        (Some(pct), _) => Paint {
            track: TRACK,
            track_alpha: 0.55,
            arc: accent,
            sweep: pct / 100.0,
            dashed: false,
        },
    }
}

/// Coverage of one pixel: how many of its samples hit the arc and the track.
fn coverage(size: u32, px: u32, py: u32, paint: &Paint) -> (f64, f64) {
    let (outer, inner) = radii(size);
    let centre = f64::from(size) / 2.0;
    let (mut arc_hits, mut track_hits) = (0_u32, 0_u32);
    for sy in 0..SUPERSAMPLE {
        for sx in 0..SUPERSAMPLE {
            let step =
                |p: u32, s: u32| f64::from(p) + (f64::from(s) + 0.5) / f64::from(SUPERSAMPLE);
            let (dx, dy) = (step(px, sx) - centre, step(py, sy) - centre);
            let radius = dx.hypot(dy);
            if radius < inner || radius > outer {
                continue;
            }
            // Clockwise fraction of the turn, 0 at 12 o'clock.
            let turn = (dx.atan2(-dy) + TAU).rem_euclid(TAU) / TAU;
            if paint.dashed && ((turn * DASHES).floor() as u32) % 2 == 1 {
                continue;
            }
            if turn <= paint.sweep {
                arc_hits += 1;
            } else {
                track_hits += 1;
            }
        }
    }
    let samples = f64::from(SUPERSAMPLE * SUPERSAMPLE);
    (f64::from(arc_hits) / samples, f64::from(track_hits) / samples)
}

pub fn render(gauge: Gauge, size: u32) -> Rgba {
    let paint = paint_for(gauge);
    let mut pixels = Vec::with_capacity((size * size * 4) as usize);
    for py in 0..size {
        for px in 0..size {
            let (arc, track) = coverage(size, px, py, &paint);
            let track_alpha = track * paint.track_alpha;
            // Arc over track, both in straight alpha.
            let alpha = arc + track_alpha * (1.0 - arc);
            if alpha <= 0.0 {
                pixels.extend_from_slice(&[0, 0, 0, 0]);
                continue;
            }
            let rgb: [u8; 3] = std::array::from_fn(|i| {
                let mixed = f64::from(paint.arc[i]) * arc
                    + f64::from(paint.track[i]) * track_alpha * (1.0 - arc);
                (mixed / alpha).round().clamp(0.0, 255.0) as u8
            });
            pixels.extend_from_slice(&rgb);
            pixels.push((alpha * 255.0).round().clamp(0.0, 255.0) as u8);
        }
    }
    Rgba { size, pixels }
}
