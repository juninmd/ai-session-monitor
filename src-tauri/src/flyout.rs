//! The flyout window: placement against the tray icon, show and hide, blur and close handling.
//! The window is created once from tauri.conf.json and never destroyed; it is only hidden and
//! shown again. Locks here are never held across a window call, because those calls can wait on
//! the main thread, which may in turn be waiting for the same lock.

use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Instant;

use tauri::window::Monitor;
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewWindow, WindowEvent,
};

use crate::gate::{ClickAction, Gate};
use crate::position::{
    Anchor, Rect, clamp_logical_height, compensate_frame, pick_monitor, place, settle_size,
    usable_tray,
};
use crate::{diag, events};

#[cfg(test)]
mod tests;

pub const LABEL: &str = "flyout";
const WIDTH_LOGICAL: f64 = 432.0;
const MARGIN_LOGICAL: f64 = 8.0;
/// The height in tauri.conf.json, kept until the UI reports its content height.
const INITIAL_HEIGHT_LOGICAL: f64 = 720.0;
/// Resizes tried before giving up on the requested size; the second one lands in practice.
const SIZE_ATTEMPTS: usize = 3;

/// Set once the page finished loading; before that the window would show an empty webview. A
/// static because the page can finish loading before the flyout is registered with the app.
static PAGE_READY: AtomicBool = AtomicBool::new(false);

/// The page finished loading.
pub fn mark_page_ready() {
    PAGE_READY.store(true, Ordering::SeqCst);
}

type Failure = Box<dyn Error + Send + Sync>;

/// Where the flyout was last opened, so a resize can keep the anchored edge in place.
#[derive(Clone, Copy)]
struct Placement {
    anchor: Anchor,
    scale: f64,
}

struct State {
    gate: Gate,
    placement: Option<Placement>,
    /// The UI's last requested content height; clamped again on every placement.
    height: f64,
    /// An opening asked for before the page was ready, with the tray rectangle it came with.
    deferred: Option<Option<tauri::Rect>>,
}

pub struct Flyout {
    app: AppHandle,
    window: WebviewWindow,
    state: Mutex<State>,
}

fn lock(state: &Mutex<State>) -> MutexGuard<'_, State> {
    state.lock().unwrap_or_else(PoisonError::into_inner)
}

fn px(value: u32) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

/// A rectangle the tray reported, in physical pixels. Windows reports physical ones already; a
/// logical one is converted with `scale`.
fn tray_rect(rect: &tauri::Rect, scale: f64) -> Rect {
    let position = rect.position.to_physical::<i32>(scale);
    let size = rect.size.to_physical::<u32>(scale);
    Rect::new(position.x, position.y, px(size.width), px(size.height))
}

/// Full bounds and work area of a monitor.
fn monitor_rects(monitor: &Monitor) -> (Rect, Rect) {
    let (position, size) = (monitor.position(), monitor.size());
    let work = monitor.work_area();
    (
        Rect::new(position.x, position.y, px(size.width), px(size.height)),
        Rect::new(work.position.x, work.position.y, px(work.size.width), px(work.size.height)),
    )
}

impl Flyout {
    pub fn new(app: &AppHandle) -> tauri::Result<Self> {
        let window = app.get_webview_window(LABEL).ok_or(tauri::Error::WindowNotFound)?;
        let state = State {
            gate: Gate::default(),
            placement: None,
            height: INITIAL_HEIGHT_LOGICAL,
            deferred: None,
        };
        Ok(Self { app: app.clone(), window, state: Mutex::new(state) })
    }

    /// Hides on blur and turns a close request into a hide.
    pub fn attach(self: &Arc<Self>) {
        let flyout = Arc::clone(self);
        self.window.on_window_event(move |event| match event {
            WindowEvent::Focused(focused) => flyout.focus_changed(*focused),
            WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                flyout.hide();
            }
            _ => {}
        });
    }

    pub fn is_visible(&self) -> bool {
        self.window.is_visible().unwrap_or(false)
    }

    /// The page finished loading: from now on the window may be shown, and an opening that was
    /// asked for earlier happens now.
    pub fn page_loaded(&self) {
        mark_page_ready();
        let deferred = lock(&self.state).deferred.take();
        if let Some(tray) = deferred {
            self.show(tray.as_ref());
        }
    }

    pub fn show(&self, tray: Option<&tauri::Rect>) {
        {
            let mut state = lock(&self.state);
            if !PAGE_READY.load(Ordering::SeqCst) {
                state.deferred = Some(tray.cloned());
                return;
            }
        }
        if let Err(err) = self.try_show(tray) {
            diag::warn("cannot open the flyout", &err);
        }
    }

    /// Startup: the window stays hidden and frameless whatever the OS did with it at creation.
    pub fn settle_hidden(&self) {
        if let Err(err) = self.window.set_decorations(false) {
            diag::warn("cannot remove the flyout frame", &err);
        }
        self.hide();
    }

    pub fn hide(&self) {
        if let Err(err) = self.window.hide() {
            diag::warn("cannot hide the flyout", &err);
        }
    }

    /// The mouse went down on the tray icon.
    pub fn tray_pressed(&self) {
        let visible = self.is_visible();
        lock(&self.state).gate.press(Instant::now(), visible);
    }

    /// The mouse came up on the tray icon: toggle, unless this click just closed the flyout.
    pub fn tray_released(&self, tray: &tauri::Rect) {
        let visible = self.is_visible();
        let action = lock(&self.state).gate.release(Instant::now(), visible);
        match action {
            ClickAction::Show => self.show(Some(tray)),
            ClickAction::Hide => self.hide(),
            ClickAction::Ignore => {}
        }
    }

    /// The content height the UI wants, in logical pixels. Applied now when the flyout is open,
    /// otherwise at the next opening.
    pub fn set_height(&self, requested: f64) {
        let placement = {
            let mut state = lock(&self.state);
            state.height = requested;
            state.placement
        };
        if self.is_visible()
            && let Some(placement) = placement
            && let Err(err) = self.apply(placement, requested, false)
        {
            diag::warn("cannot resize the flyout", &err);
        }
    }

    fn focus_changed(&self, focused: bool) {
        if focused {
            return;
        }
        let visible = self.is_visible();
        if lock(&self.state).gate.blurred(Instant::now(), visible) {
            self.hide();
        }
    }

    fn try_show(&self, tray: Option<&tauri::Rect>) -> Result<(), Failure> {
        let placement = self.locate(tray)?;
        let height = {
            let mut state = lock(&self.state);
            state.placement = Some(placement);
            // Recorded before the window appears: the blur guard must already be running when
            // the shell settles focus.
            state.gate.shown(Instant::now());
            state.height
        };
        self.apply(placement, height, true)?;
        self.window.show()?;
        if let Err(err) = self.window.set_focus() {
            diag::warn("cannot focus the flyout", &err);
        }
        self.app.emit_to(LABEL, events::FLYOUT_SHOWN, ())?;
        Ok(())
    }

    /// The monitor holding the tray icon and the anchor on it. Without the icon's position the
    /// primary monitor is used: that is where the notification area lives.
    fn locate(&self, tray: Option<&tauri::Rect>) -> Result<Placement, Failure> {
        let monitors = self.app.available_monitors()?;
        if monitors.is_empty() {
            return Err("no monitor is available".into());
        }
        let primary = self.app.primary_monitor()?;
        let scale = primary.as_ref().map_or(1.0, Monitor::scale_factor);
        let tray = usable_tray(tray.map(|rect| tray_rect(rect, scale)));
        let rects: Vec<(Rect, Rect)> = monitors.iter().map(monitor_rects).collect();
        let bounds: Vec<Rect> = rects.iter().map(|(bounds, _)| *bounds).collect();
        let probe = tray
            .map(|rect| rect.center())
            .or_else(|| primary.as_ref().map(|monitor| monitor_rects(monitor).0.center()))
            .unwrap_or_else(|| bounds[0].center());
        let index = pick_monitor(probe, &bounds).unwrap_or(0);
        let (monitor, work) = rects[index];
        Ok(Placement {
            anchor: Anchor { tray, monitor, work },
            scale: monitors[index].scale_factor(),
        })
    }

    /// Positions and sizes the window. `relocate` first moves it onto the target monitor, whose
    /// DPI may resize it; the size is then set until it holds (see `settle_size`), and the position
    /// repeated with the window's invisible borders measured out, so the visible panel lands where
    /// `place` says.
    fn apply(
        &self,
        placement: Placement,
        requested_height: f64,
        relocate: bool,
    ) -> Result<(), Failure> {
        let Placement { anchor, scale } = placement;
        let margin = (MARGIN_LOGICAL * scale).round() as i32;
        let height = clamp_logical_height(requested_height, scale, anchor.work, margin);
        let size = ((WIDTH_LOGICAL * scale).round() as i32, (height * scale).round() as i32);
        let at = place(&anchor, size, margin);
        if relocate {
            self.window.set_position(PhysicalPosition::new(at.x, at.y))?;
        }
        let want = (size.0.unsigned_abs(), size.1.unsigned_abs());
        let held = settle_size(
            want,
            SIZE_ATTEMPTS,
            || self.window.inner_size().map(|now| (now.width, now.height)),
            || self.window.set_size(PhysicalSize::new(want.0, want.1)),
        )?;
        if !held {
            diag::warn("cannot size the flyout", &"the window did not keep the requested size");
        }
        let (outer, inner) = (self.window.outer_position()?, self.window.inner_position()?);
        let to = compensate_frame(at, (outer.x, outer.y), (inner.x, inner.y));
        self.window.set_position(PhysicalPosition::new(to.x, to.y))?;
        Ok(())
    }
}
