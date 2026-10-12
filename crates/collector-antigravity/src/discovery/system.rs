//! The real discovery: same-user processes through `sysinfo`, listening ports through `netstat`.
//!
//! Why `sysinfo` and not `Get-CimInstance Win32_Process` through PowerShell: it runs in-process, so
//! a poll never starts a script interpreter (slow, heavy and a favourite alert of endpoint
//! security), there is no output encoding to get wrong, and no command text exists to tamper
//! with. The command line of a process is only read after its *name* matched, so unrelated
//! processes are never inspected, and only processes of the user running this app are accepted
//! (Windows hands the command line of other users' processes only to administrators).

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;

use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};
use usage_model::{ErrorCode, HarnessError};

use super::{Discovery, DiscoveryError, ServerFlags, Target, netstat, plan_targets};

/// `language_server_windows_x64.exe`, `language_server.exe`, `language_server_linux_x64`, ...
const PROCESS_PREFIX: &str = "language_server";
/// Name matches inspected at most; an IDE with several windows runs a handful.
const MAX_PROCESSES: usize = 16;
/// Servers called at most, newest first.
const MAX_SERVERS: usize = 4;

#[derive(Debug)]
pub(crate) struct SystemDiscovery {
    prefix: String,
}

impl SystemDiscovery {
    pub(crate) fn new() -> Self {
        Self { prefix: PROCESS_PREFIX.to_owned() }
    }

    /// Tests look for their own fabricated process name, never for a real server.
    #[cfg(test)]
    pub(crate) fn with_prefix(prefix: &str) -> Self {
        Self { prefix: prefix.to_ascii_lowercase() }
    }
}

impl Discovery for SystemDiscovery {
    async fn find(&self) -> Result<Vec<Target>, DiscoveryError> {
        let prefix = self.prefix.clone();
        let mut servers = tokio::task::spawn_blocking(move || scan(&prefix))
            .await
            .map_err(|_| DiscoveryError::Failed(HarnessError::new(ErrorCode::Internal, false)))?;
        if servers.is_empty() {
            return Err(DiscoveryError::NotRunning);
        }
        servers.sort_by(|a, b| b.started.cmp(&a.started).then(b.pid.cmp(&a.pid)));
        servers.truncate(MAX_SERVERS);
        plan(&servers, netstat::listening_ports().await)
    }
}

/// One Antigravity language server of this user.
#[derive(Debug)]
struct Server {
    pid: u32,
    /// Epoch seconds; newer servers are tried first.
    started: u64,
    flags: ServerFlags,
}

fn name_matches(name: &OsStr, prefix: &str) -> bool {
    name.to_string_lossy().to_ascii_lowercase().replace('-', "_").starts_with(prefix)
}

fn scan(prefix: &str) -> Vec<Server> {
    let mut system = System::new();
    system.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing());
    let pids: Vec<Pid> = system
        .processes()
        .iter()
        .filter(|(_, process)| name_matches(process.name(), prefix))
        .map(|(pid, _)| *pid)
        .take(MAX_PROCESSES)
        .collect();
    let Ok(me) = sysinfo::get_current_pid() else { return Vec::new() };
    if pids.is_empty() {
        return Vec::new();
    }

    let owner = ProcessRefreshKind::nothing().with_user(UpdateKind::Always);
    system.refresh_processes_specifics(ProcessesToUpdate::Some(&[me]), true, owner);
    let inspect = owner.with_cmd(UpdateKind::Always).with_exe(UpdateKind::Always);
    system.refresh_processes_specifics(ProcessesToUpdate::Some(&pids), true, inspect);
    // An unknown owner on either side is a refusal, not a match.
    let Some(mine) = system.process(me).and_then(|process| process.user_id()).cloned() else {
        return Vec::new();
    };
    pids.iter()
        .filter_map(|pid| system.process(*pid))
        .filter(|process| process.user_id() == Some(&mine))
        .filter_map(|process| {
            let args: Vec<String> =
                process.cmd().iter().map(|arg| arg.to_string_lossy().into_owned()).collect();
            let flags = ServerFlags::parse(&args);
            flags.is_antigravity(process.exe(), &args).then(|| Server {
                pid: process.pid().as_u32(),
                started: process.start_time(),
                flags,
            })
        })
        .collect()
}

fn plan(
    servers: &[Server],
    table: Result<BTreeMap<u32, BTreeSet<u16>>, HarnessError>,
) -> Result<Vec<Target>, DiscoveryError> {
    let (ports, failure) = match table {
        Ok(ports) => (ports, None),
        // The ports the server was told to use still work without the table.
        Err(error) => (BTreeMap::new(), Some(error)),
    };
    let none = BTreeSet::new();
    let targets: Vec<Target> = servers
        .iter()
        .flat_map(|server| plan_targets(&server.flags, ports.get(&server.pid).unwrap_or(&none)))
        .collect();
    if !targets.is_empty() {
        return Ok(targets);
    }
    if servers.iter().any(|server| server.flags.has_usable_token()) {
        Err(DiscoveryError::NoPort(failure))
    } else {
        Err(DiscoveryError::NoToken)
    }
}

#[cfg(test)]
mod tests;
