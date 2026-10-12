use std::collections::BTreeSet;

use super::*;

const ENGLISH: &str = "
Active Connections

  Proto  Local Address          Foreign Address        State           PID
  TCP    0.0.0.0:135            0.0.0.0:0              LISTENING       1112
  TCP    127.0.0.1:50123        0.0.0.0:0              LISTENING       4242
  TCP    127.0.0.1:50123        127.0.0.1:50510        ESTABLISHED     4242
  TCP    127.0.0.1:50510        127.0.0.1:50123        ESTABLISHED     7777
  TCP    127.0.0.1:50200        0.0.0.0:0              LISTENING       4242
  TCP    192.168.1.20:50300     0.0.0.0:0              LISTENING       4242
  TCP    [::]:135               [::]:0                 LISTENING       1112
  TCP    [::1]:50400            [::]:0                 LISTENING       4242
  TCP    127.0.0.1:50999        127.0.0.1:50123        TIME_WAIT       0
";

const PORTUGUESE: &str = "
Conex\u{f5}es ativas

  Proto  Endere\u{e7}o local         Endere\u{e7}o externo       Estado          PID
  TCP    127.0.0.1:50123        0.0.0.0:0              OUVINDO         4242
  TCP    127.0.0.1:50123        127.0.0.1:50510        ESTABELECIDA    4242
";

fn ports(map: &BTreeMap<u32, BTreeSet<u16>>, pid: u32) -> Vec<u16> {
    map.get(&pid).map(|set| set.iter().copied().collect()).unwrap_or_default()
}

#[test]
fn listening_loopback_sockets_are_found_per_process() {
    let found = parse_listening(ENGLISH);
    assert_eq!(ports(&found, 4242), [50123, 50200]);
    // 0.0.0.0 accepts connections on 127.0.0.1 too.
    assert_eq!(ports(&found, 1112), [135]);
    assert_eq!(found.len(), 2);
}

#[test]
fn only_the_shape_of_a_row_matters_not_the_language() {
    let found = parse_listening(PORTUGUESE);
    assert_eq!(ports(&found, 4242), [50123]);
    assert_eq!(found.len(), 1);
    // The state may even be several words.
    let multi = parse_listening("  TCP  127.0.0.1:4000  0.0.0.0:0  ESCUTANDO ATIVAMENTE  31\n");
    assert_eq!(ports(&multi, 31), [4000]);
}

#[test]
fn connections_other_families_and_other_addresses_are_not_listeners() {
    let found = parse_listening(ENGLISH);
    // 50510 is only ever the client end; 50300 is bound to a LAN address; [::1] is IPv6.
    for (pid, port) in [(7777, 50510), (4242, 50300), (4242, 50400), (4242, 50510), (0, 50999)] {
        assert!(!ports(&found, pid).contains(&port), "{pid}:{port}");
    }
}

#[test]
fn rows_that_are_not_rows_are_skipped_without_failing() {
    let text = "\
TCP 127.0.0.1:1000 0.0.0.0:0 LISTENING
TCP 127.0.0.1:1001 0.0.0.0:0 LISTENING abc
TCP 127.0.0.1:99999 0.0.0.0:0 LISTENING 5
TCP 127.0.0.1:0 0.0.0.0:0 LISTENING 5
TCP 127.0.0.1 0.0.0.0:0 LISTENING 5
TCP 127.0.0.1:1002 0.0.0.0 LISTENING 5
TCP 127.0.0.1:1003 0.0.0.0:x LISTENING 5
UDP 127.0.0.1:1004 0.0.0.0:0 LISTENING 5
 TCP
:::

TCP 127.0.0.1:1005 0.0.0.0:0 LISTENING 99999999999
TCP 127.0.0.1:1006 0.0.0.0:0 LISTENING 5
";
    let found = parse_listening(text);
    assert_eq!(ports(&found, 5), [1006]);
    assert_eq!(found.len(), 1);
    assert!(parse_listening("").is_empty());
    assert!(parse_listening("\0\0\0 \u{1f511}\u{1f511}").is_empty());
}

#[test]
fn protocol_case_line_endings_and_odd_spacing_do_not_matter() {
    let found = parse_listening(
        "tcp\t127.0.0.1:2000\t0.0.0.0:0\tLISTENING\t8\r\n  Tcp   0.0.0.0:2001   0.0.0.0:0   LISTENING   8  \r\n",
    );
    assert_eq!(ports(&found, 8), [2000, 2001]);
}

#[test]
fn a_process_with_several_listeners_gets_them_all_once_in_order() {
    let rows: String = [2500, 2100, 2300, 2100]
        .iter()
        .map(|port| format!("TCP 127.0.0.1:{port} 0.0.0.0:0 LISTENING 12\n"))
        .collect();
    assert_eq!(ports(&parse_listening(&rows), 12), [2100, 2300, 2500]);
}

#[test]
fn only_the_first_rows_are_read() {
    let filler = "noise\n".repeat(MAX_LINES - 1);
    let last = "TCP 127.0.0.1:3000 0.0.0.0:0 LISTENING 77\n";
    assert_eq!(ports(&parse_listening(&format!("{filler}{last}")), 77), [3000]);
    assert!(parse_listening(&format!("{filler}noise\n{last}")).is_empty());
}

#[cfg(windows)]
#[tokio::test]
async fn the_real_tool_reports_a_listener_this_test_opened() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let table = listening_ports().await.expect("netstat runs and parses");
    assert!(ports(&table, std::process::id()).contains(&port), "own port {port} missing");
    drop(listener);
}
