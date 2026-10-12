//! "Start with Windows": a pure reconcile plan plus an adapter over the per-user Run registry key.
//! The adapter talks to `auto-launch` directly instead of through the Tauri plugin because the
//! plugin picks the machine-wide key whenever the process is elevated, which would start the app
//! for every user of the PC.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use auto_launch::{AutoLaunch, AutoLaunchBuilder, WindowsEnableMode};

#[cfg(test)]
mod tests;

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct AutostartError(String);

/// What the OS registration has to do to match the setting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Plan {
    Enable,
    Disable,
    Keep,
}

/// Why the registration is being reconciled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    /// The app just started and adopts the stored setting. It leaves alone what the user decided
    /// in Task Manager.
    Startup,
    /// The user changed the setting. Turning it on also clears a Task Manager disable.
    UserChange,
}

pub fn plan(wanted: bool, enabled: bool) -> Plan {
    match (wanted, enabled) {
        (true, false) => Plan::Enable,
        (false, true) => Plan::Disable,
        _ => Plan::Keep,
    }
}

pub trait Autostart: Send + Sync {
    /// The Run value exists and Windows is not holding it back (Task Manager can).
    fn is_enabled(&self) -> Result<bool, AutostartError>;
    /// The Run value exists, whatever Task Manager says about it. Reads nothing else.
    fn is_registered(&self) -> Result<bool, AutostartError>;
    /// Writes the Run value and clears a Task Manager disable.
    fn enable(&self) -> Result<(), AutostartError>;
    fn disable(&self) -> Result<(), AutostartError>;
}

/// Brings the registration in line with `wanted`, writing only when it differs. A start-up pass
/// compares the Run value alone, because `enable` would overwrite a Task Manager disable that the
/// user chose; after a change of the setting it compares what Windows would start.
pub fn reconcile(
    port: &dyn Autostart,
    wanted: bool,
    origin: Origin,
) -> Result<Plan, AutostartError> {
    let present = match origin {
        Origin::Startup => port.is_registered()?,
        Origin::UserChange => port.is_enabled()?,
    };
    let step = plan(wanted, present);
    match step {
        Plan::Enable => port.enable()?,
        Plan::Disable => port.disable()?,
        Plan::Keep => {}
    }
    Ok(step)
}

#[derive(Clone, Copy)]
struct Request {
    wanted: bool,
    origin: Origin,
}

/// Serialises registry writes and always applies the most recent request, so two quick toggles
/// racing on worker threads cannot leave the older one in place.
pub struct Applier {
    port: Arc<dyn Autostart>,
    request: Mutex<Request>,
    failed: AtomicBool,
    serial: Mutex<()>,
}

impl Applier {
    pub fn new(port: Arc<dyn Autostart>) -> Self {
        let idle = Request { wanted: false, origin: Origin::Startup };
        Self {
            port,
            request: Mutex::new(idle),
            failed: AtomicBool::new(false),
            serial: Mutex::new(()),
        }
    }

    pub fn request(&self, wanted: bool, origin: Origin) {
        *self.request.lock().unwrap_or_else(PoisonError::into_inner) = Request { wanted, origin };
    }

    /// Whether the last `apply` failed.
    pub fn failed(&self) -> bool {
        self.failed.load(Ordering::SeqCst)
    }

    /// Blocking: reads and writes the registry.
    pub fn apply(&self) -> Result<Plan, AutostartError> {
        let _serial = self.serial.lock().unwrap_or_else(PoisonError::into_inner);
        let Request { wanted, origin } =
            *self.request.lock().unwrap_or_else(PoisonError::into_inner);
        let outcome = reconcile(self.port.as_ref(), wanted, origin);
        self.failed.store(outcome.is_err(), Ordering::SeqCst);
        outcome
    }
}

pub struct RunKey(AutoLaunch);

impl RunKey {
    /// Registers the running executable under `app_name`, for the current user only.
    pub fn for_current_exe(app_name: &str) -> Result<Self, AutostartError> {
        let exe = std::env::current_exe().map_err(|e| AutostartError(e.to_string()))?;
        // Quoted because the Run value is parsed as a command line and installs live under
        // folders with spaces in their names.
        let command = format!("\"{}\"", exe.display());
        AutoLaunchBuilder::new()
            .set_app_name(app_name)
            .set_app_path(&command)
            .set_windows_enable_mode(WindowsEnableMode::CurrentUser)
            .build()
            .map(Self)
            .map_err(|e| AutostartError(e.to_string()))
    }
}

impl Autostart for RunKey {
    fn is_enabled(&self) -> Result<bool, AutostartError> {
        self.0.is_enabled().map_err(|e| AutostartError(e.to_string()))
    }

    fn is_registered(&self) -> Result<bool, AutostartError> {
        #[cfg(windows)]
        {
            run_value_exists(windows_registry::CURRENT_USER, RUN_KEY, self.0.get_app_name())
        }
        #[cfg(not(windows))]
        {
            self.is_enabled()
        }
    }

    fn enable(&self) -> Result<(), AutostartError> {
        self.0.enable().map_err(|e| AutostartError(e.to_string()))
    }

    fn disable(&self) -> Result<(), AutostartError> {
        self.0.disable().map_err(|e| AutostartError(e.to_string()))
    }
}

#[cfg(windows)]
const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";

/// Win32 `ERROR_FILE_NOT_FOUND` as an HRESULT: the key or the value is not there.
#[cfg(windows)]
const FILE_NOT_FOUND: i32 = 0x8007_0002_u32 as i32;

/// Whether `key_path` under `root` holds a value called `name`. Opens the key for reading only.
#[cfg(windows)]
fn run_value_exists(
    root: &windows_registry::Key,
    key_path: &str,
    name: &str,
) -> Result<bool, AutostartError> {
    match root.open(key_path).and_then(|key| key.get_type(name)) {
        Ok(_) => Ok(true),
        Err(err) if err.code().0 == FILE_NOT_FOUND => Ok(false),
        Err(err) => Err(AutostartError(err.to_string())),
    }
}
