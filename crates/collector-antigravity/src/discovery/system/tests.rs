use std::ffi::OsStr;

use super::*;

fn server(pid: u32, started: u64, args: &[&str]) -> Server {
    let args: Vec<String> = args.iter().map(|arg| (*arg).to_owned()).collect();
    Server { pid, started, flags: ServerFlags::parse(&args) }
}

fn table(rows: &[(u32, &[u16])]) -> Result<BTreeMap<u32, BTreeSet<u16>>, HarnessError> {
    Ok(rows.iter().map(|(pid, ports)| (*pid, ports.iter().copied().collect())).collect())
}

fn ports_of(targets: &[Target]) -> Vec<(u16, &str)> {
    targets.iter().map(|target| (target.port, target.token.expose())).collect()
}

#[test]
fn process_names_match_by_prefix_ignoring_case_and_dashes() {
    for name in [
        "language_server_windows_x64.exe",
        "Language_Server_Windows_X64.EXE",
        "language_server.exe",
        "language-server",
        "language_server_linux_arm",
    ] {
        assert!(name_matches(OsStr::new(name), PROCESS_PREFIX), "{name}");
    }
    for name in ["", "server.exe", "my_language_server.exe", "languageserver.exe", "lang_server"] {
        assert!(!name_matches(OsStr::new(name), PROCESS_PREFIX), "{name}");
    }
}

#[test]
fn targets_follow_the_servers_in_the_order_given_with_the_ports_of_their_own_pid() {
    let servers = [
        server(10, 200, &["--csrf_token", "newer"]),
        server(20, 100, &["--csrf_token", "older", "--https_server_port", "7000"]),
    ];
    let found = plan(&servers, table(&[(10, &[5001, 5002]), (20, &[6001]), (99, &[1])])).unwrap();
    assert_eq!(
        ports_of(&found),
        [(5001, "newer"), (5002, "newer"), (7000, "older"), (6001, "older"),]
    );
}

#[test]
fn a_server_without_a_token_adds_nothing_while_another_has_one() {
    let servers =
        [server(1, 2, &["--app_data_dir", "antigravity"]), server(3, 1, &["--csrf_token", "t"])];
    let found = plan(&servers, table(&[(1, &[4000]), (3, &[4100])])).unwrap();
    assert_eq!(ports_of(&found), [(4100, "t")]);
}

#[test]
fn a_failed_port_table_still_leaves_the_ports_the_server_was_told_to_use() {
    let servers = [server(1, 1, &["--csrf_token", "t", "--https_server_port", "7443"])];
    let broken = Err(HarnessError::new(ErrorCode::Timeout, true));
    assert_eq!(ports_of(&plan(&servers, broken).unwrap()), [(7443, "t")]);
}

#[test]
fn nothing_to_call_is_explained() {
    let tokenless = [server(1, 1, &["--https_server_port", "7443"])];
    assert!(matches!(plan(&tokenless, table(&[(1, &[1])])), Err(DiscoveryError::NoToken)));

    let starting = [server(1, 1, &["--csrf_token", "t"])];
    assert!(matches!(plan(&starting, table(&[(2, &[9])])), Err(DiscoveryError::NoPort(None))));
    let cause = HarnessError::new(ErrorCode::SpawnFailed, false);
    let unreadable = plan(&starting, Err(cause.clone()));
    assert!(matches!(&unreadable, Err(DiscoveryError::NoPort(Some(found))) if *found == cause));
}

#[tokio::test]
async fn no_such_process_is_not_running() {
    // A name no real program has: the scan reads the process list and finds nothing.
    let discovery = SystemDiscovery::with_prefix("zz_ai_session_monitor_no_such_process");
    assert!(matches!(discovery.find().await, Err(DiscoveryError::NotRunning)));
}

/// Against the real operating system: a child process of this test binary stands in for the
/// language server (nothing of Antigravity is touched), carrying fabricated flags on its command
/// line. It proves the command line of a same-user process is readable and that the scan picks
/// it by name and by `--app_data_dir`.
#[cfg(windows)]
mod real_os {
    use std::process::{Child, Command, Stdio};
    use std::time::{Duration, Instant};

    use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

    use super::*;
    use crate::discovery::Scheme;

    const PARK_VAR: &str = "AI_SESSION_MONITOR_TEST_PARK";
    const TOKEN: &str = "fabricated-token-for-the-self-exec-test";

    /// Keeps the child alive for the parent to inspect; does nothing in a normal run.
    #[test]
    fn child_parks_when_asked_to() {
        if std::env::var_os(PARK_VAR).is_some() {
            std::thread::sleep(Duration::from_secs(60));
        }
    }

    /// Ends the child however the test ends.
    struct Parked(Child);

    impl Parked {
        fn pid(&self) -> u32 {
            self.0.id()
        }
    }

    impl Drop for Parked {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    /// This test binary is `collector_antigravity-<hash>.exe`: its own name is the prefix.
    fn own_prefix() -> String {
        let exe = std::env::current_exe().unwrap();
        let stem = exe.file_stem().unwrap().to_string_lossy().to_ascii_lowercase();
        stem.split('-').next().unwrap().to_owned()
    }

    fn spawn_parked(app_data_dir: &str, port: &str) -> Parked {
        let child = Command::new(std::env::current_exe().unwrap())
            // Everything after `--` is a free argument to the test harness: ignored by it, still
            // on the command line.
            .args(["--exact", "discovery::system::tests::real_os::child_parks_when_asked_to", "--"])
            .args([
                "--csrf_token",
                TOKEN,
                "--https_server_port",
                port,
                "--app_data_dir",
                app_data_dir,
            ])
            .env(PARK_VAR, "1")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        Parked(child)
    }

    fn wait_for(prefix: &str, pid: u32) -> Option<Server> {
        let deadline = Instant::now() + Duration::from_secs(15);
        while Instant::now() < deadline {
            if let Some(found) = scan(prefix).into_iter().find(|server| server.pid == pid) {
                return Some(found);
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        None
    }

    /// The command line of `pid` as the operating system reports it, bypassing the scan's filters.
    fn raw_command_line(pid: u32) -> Vec<String> {
        let pid = Pid::from_u32(pid);
        let mut system = System::new();
        let kind = ProcessRefreshKind::nothing().with_cmd(UpdateKind::Always);
        system.refresh_processes_specifics(ProcessesToUpdate::Some(&[pid]), true, kind);
        let args = system.process(pid).map(|process| process.cmd().to_vec()).unwrap_or_default();
        args.iter().map(|arg| arg.to_string_lossy().into_owned()).collect()
    }

    #[test]
    fn the_command_line_of_a_same_user_process_is_read_and_the_app_is_told_apart() {
        let prefix = own_prefix();
        let antigravity = spawn_parked("antigravity", "45678");
        let other = spawn_parked("windsurf", "45679");

        let found = wait_for(&prefix, antigravity.pid()).expect("the scan sees the child");
        assert_eq!(found.flags.csrf_token.as_ref().map(|token| token.expose()), Some(TOKEN));
        assert_eq!(found.flags.https_port, Some(45678));
        assert_eq!(found.flags.app_data_dir.as_deref(), Some("antigravity"));
        assert!(found.started > 0);

        // The other child is alive and carries its own app name; the scan leaves it out.
        let seen = Instant::now() + Duration::from_secs(15);
        let mut line = raw_command_line(other.pid());
        while !line.iter().any(|arg| arg == "windsurf") && Instant::now() < seen {
            std::thread::sleep(Duration::from_millis(100));
            line = raw_command_line(other.pid());
        }
        assert!(line.iter().any(|arg| arg == "windsurf"), "the OS reports the other child's flags");
        assert!(scan(&prefix).iter().all(|server| server.pid != other.pid()));
    }

    #[tokio::test]
    async fn discovery_turns_that_command_line_into_a_target() {
        let child = spawn_parked("antigravity-ide", "45680");
        let discovery = SystemDiscovery::with_prefix(&own_prefix());
        let deadline = Instant::now() + Duration::from_secs(15);
        let mut found = Vec::new();
        while Instant::now() < deadline && !found.iter().any(|target: &Target| target.port == 45680)
        {
            found = discovery.find().await.unwrap_or_default();
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        let target =
            found.iter().find(|target| target.port == 45680).expect("target for the child");
        assert_eq!((target.scheme, target.token.expose()), (Scheme::Https, TOKEN));
        drop(child);
    }
}
