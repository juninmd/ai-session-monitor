//! Flyout placement from the tray icon and the monitor, in physical pixels. Pure: the shell feeds
//! it rectangles and applies the result, so every taskbar edge and DPI case is unit-testable.

#[cfg(test)]
mod tests;

/// Smallest flyout height the UI may ask for, in logical pixels.
pub const MIN_HEIGHT_LOGICAL: f64 = 200.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }

    pub const fn right(&self) -> i32 {
        self.x.saturating_add(self.w)
    }

    pub const fn bottom(&self) -> i32 {
        self.y.saturating_add(self.h)
    }

    pub const fn center(&self) -> (i32, i32) {
        (self.x.saturating_add(self.w / 2), self.y.saturating_add(self.h / 2))
    }

    pub const fn contains(&self, px: i32, py: i32) -> bool {
        px >= self.x && px < self.right() && py >= self.y && py < self.bottom()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    Top,
    Bottom,
    Left,
    Right,
}

/// What the flyout is placed against. `tray` is `None` when the icon rectangle is unknown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Anchor {
    pub tray: Option<Rect>,
    pub monitor: Rect,
    pub work: Rect,
}

/// The icon rectangle, unless the shell reported an empty one: there is nothing to anchor to, so
/// the flyout takes the notification corner like it does for an unknown icon.
pub fn usable_tray(tray: Option<Rect>) -> Option<Rect> {
    tray.filter(|rect| rect.w > 0 && rect.h > 0)
}

/// Band of the monitor the taskbar reserves on `edge`, 0 when nothing is reserved there.
fn reserved_band(edge: Edge, monitor: Rect, work: Rect) -> i32 {
    match edge {
        Edge::Bottom => monitor.bottom() - work.bottom(),
        Edge::Top => work.y - monitor.y,
        Edge::Right => monitor.right() - work.right(),
        Edge::Left => work.x - monitor.x,
    }
}

/// Which edge the taskbar sits on. The reserved band of the work area decides, because the icon
/// of the overflow popup floats above the taskbar and would mislead. With an auto-hide taskbar
/// nothing is reserved, so the monitor edge nearest to the icon wins.
pub fn taskbar_edge(monitor: Rect, work: Rect, tray: Option<Rect>) -> Edge {
    // Order doubles as the tie-break: bottom is the Windows default.
    let order = [Edge::Bottom, Edge::Top, Edge::Right, Edge::Left];
    let widest = order.iter().copied().fold((0, None), |best, edge| {
        let band = reserved_band(edge, monitor, work);
        if band > best.0 { (band, Some(edge)) } else { best }
    });
    if let (_, Some(edge)) = widest {
        return edge;
    }
    let Some(tray) = tray else { return Edge::Bottom };
    let (cx, cy) = tray.center();
    let distances = [
        (monitor.bottom() - cy, Edge::Bottom),
        (cy - monitor.y, Edge::Top),
        (monitor.right() - cx, Edge::Right),
        (cx - monitor.x, Edge::Left),
    ];
    distances.iter().copied().fold(distances[0], |best, d| if d.0 < best.0 { d } else { best }).1
}

/// `v` limited to `lo..=hi`; when the range is inverted (window larger than the area) `lo` wins
/// so the window keeps its top-left corner visible.
fn clamp_into(v: i32, lo: i32, hi: i32) -> i32 {
    if hi < lo { lo } else { v.clamp(lo, hi) }
}

/// Top-left corner for a window of `size` (physical px) opened from the tray icon.
pub fn place(anchor: &Anchor, size: (i32, i32), margin: i32) -> Point {
    let (w, h) = size;
    let Anchor { tray, monitor, work } = *anchor;
    let edge = taskbar_edge(monitor, work, tray);
    let reserved = reserved_band(edge, monitor, work) > 0;
    let (min_x, max_x) = (work.x + margin, work.right() - w - margin);
    let (min_y, max_y) = (work.y + margin, work.bottom() - h - margin);
    // Along the taskbar the window centres on the icon; without one it takes the far corner,
    // where the notification area lives.
    let along_x = tray.map_or(max_x, |t| t.center().0 - w / 2);
    let along_y = tray.map_or(max_y, |t| t.center().1 - h / 2);
    let (x, y) = match edge {
        Edge::Bottom => {
            let flush = if reserved { work.bottom() } else { tray.map_or(work.bottom(), |t| t.y) };
            (along_x, flush - h - margin)
        }
        Edge::Top => {
            let flush = if reserved { work.y } else { tray.map_or(work.y, |t| t.bottom()) };
            (along_x, flush + margin)
        }
        Edge::Left => {
            let flush = if reserved { work.x } else { tray.map_or(work.x, |t| t.right()) };
            (flush + margin, along_y)
        }
        Edge::Right => {
            let flush = if reserved { work.right() } else { tray.map_or(work.right(), |t| t.x) };
            (flush - w - margin, along_y)
        }
    };
    Point { x: clamp_into(x, min_x, max_x), y: clamp_into(y, min_y, max_y) }
}

/// Window position that puts the client area at `target`, given where the window rect and the
/// client area currently start. On Windows the rect of an undecorated window still carries
/// invisible borders, so placing the rect at `target` would shift what the user sees.
pub fn compensate_frame(target: Point, outer: (i32, i32), inner: (i32, i32)) -> Point {
    Point {
        x: target.x.saturating_sub(inner.0.saturating_sub(outer.0)),
        y: target.y.saturating_sub(inner.1.saturating_sub(outer.1)),
    }
}

/// Resizes with `set` until `read` reports `want`, at most `attempts` times; false when the size
/// never held. tao measures the window's invisible borders on every resize, and before the first
/// frame update they are stale, so the first resize of a never-shown window misses by a title bar.
pub fn settle_size<E>(
    want: (u32, u32),
    attempts: usize,
    mut read: impl FnMut() -> Result<(u32, u32), E>,
    mut set: impl FnMut() -> Result<(), E>,
) -> Result<bool, E> {
    for _ in 0..attempts {
        set()?;
        if read()? == want {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Index of the monitor holding `point`, else the nearest one by centre distance.
pub fn pick_monitor(point: (i32, i32), monitors: &[Rect]) -> Option<usize> {
    if let Some(index) = monitors.iter().position(|m| m.contains(point.0, point.1)) {
        return Some(index);
    }
    let distance = |m: &Rect| {
        let (cx, cy) = m.center();
        let (dx, dy) = (i64::from(cx) - i64::from(point.0), i64::from(cy) - i64::from(point.1));
        dx * dx + dy * dy
    };
    monitors.iter().enumerate().min_by_key(|(_, m)| distance(m)).map(|(index, _)| index)
}

/// Tallest flyout the work area can hold, in physical pixels.
pub fn max_height(work: Rect, margin: i32) -> i32 {
    (work.h - 2 * margin).max(1)
}

/// The UI's requested content height, limited to what the work area can hold. A request that is
/// not a finite number falls back to the minimum.
pub fn clamp_logical_height(requested: f64, scale: f64, work: Rect, margin: i32) -> f64 {
    let scale = if scale.is_finite() && scale > 0.0 { scale } else { 1.0 };
    let ceiling = f64::from(max_height(work, margin)) / scale;
    let wanted = if requested.is_finite() { requested } else { MIN_HEIGHT_LOGICAL };
    // On a tiny work area the ceiling wins over the minimum.
    wanted.max(MIN_HEIGHT_LOGICAL).min(ceiling)
}
