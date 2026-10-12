//! Which loopback TCP ports a process listens on, from `netstat -ano -p tcp`.
//!
//! The headers and the state word are localized (`LISTENING` is `OUVINDO` in pt-BR), so rows are
//! recognized by shape instead: `TCP <local> <remote> <state...> <pid>`, where a listening socket
//! has remote port 0. Only IPv4 rows count: the endpoint is called on `127.0.0.1`.

use std::collections::{BTreeMap, BTreeSet};

/// Rows read at most; a busy machine has a few thousand.
const MAX_LINES: usize = 200_000;

/// Listening ports of every process in the table that a client on `127.0.0.1` can reach.
pub(crate) fn parse_listening(text: &str) -> BTreeMap<u32, BTreeSet<u16>> {
    let mut found: BTreeMap<u32, BTreeSet<u16>> = BTreeMap::new();
    for line in text.lines().take(MAX_LINES) {
        let columns: Vec<&str> = line.split_whitespace().collect();
        let [proto, local, remote, .., pid] = columns.as_slice() else { continue };
        if !proto.eq_ignore_ascii_case("tcp") {
            continue;
        }
        let (Some((host, port)), Some((_, remote_port)), Ok(pid)) =
            (split_address(local), split_address(remote), pid.parse::<u32>())
        else {
            continue;
        };
        let reachable = matches!(host, "127.0.0.1" | "0.0.0.0");
        if reachable && remote_port == 0 && port != 0 {
            found.entry(pid).or_default().insert(port);
        }
    }
    found
}

fn split_address(address: &str) -> Option<(&str, u16)> {
    let (host, port) = address.rsplit_once(':')?;
    Some((host, port.parse().ok()?))
}

/// Runs the real tool. `netstat.exe` is started by absolute path: one found through PATH could be
/// planted by whoever can write there.
#[cfg(windows)]
pub(crate) async fn listening_ports()
-> Result<BTreeMap<u32, BTreeSet<u16>>, usage_model::HarnessError> {
    use std::time::Duration;

    use usage_model::{ErrorCode, HarnessError};
    use usage_support::{CommandSpec, run_capped};

    let root = std::env::var_os("SystemRoot").unwrap_or_else(|| r"C:\Windows".into());
    let mut spec =
        CommandSpec::new(std::path::Path::new(&root).join("System32").join("netstat.exe"));
    spec.args = ["-ano", "-p", "tcp"].map(Into::into).to_vec();
    spec.timeout = Duration::from_secs(5);
    spec.max_stdout = 4 << 20;
    spec.max_stderr = 8 << 10;
    let output = run_capped(spec).await?;
    if output.status_code != Some(0) {
        return Err(HarnessError::new(ErrorCode::SpawnFailed, false));
    }
    Ok(parse_listening(&String::from_utf8_lossy(&output.stdout)))
}

/// Other platforms have no `netstat` with this output; the declared ports are all there is.
#[cfg(not(windows))]
pub(crate) async fn listening_ports()
-> Result<BTreeMap<u32, BTreeSet<u16>>, usage_model::HarnessError> {
    Ok(BTreeMap::new())
}

#[cfg(test)]
mod tests;
