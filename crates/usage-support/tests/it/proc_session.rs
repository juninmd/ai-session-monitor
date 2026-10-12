//! `LineSession`: a line-oriented conversation with a child over stdin and stdout.

use std::time::{Duration, Instant};

use usage_model::ErrorCode;
use usage_support::{CommandSpec, DEFAULT_MAX_LINE, LineSession};

use crate::common::peer::peer;

const WAIT: Duration = Duration::from_secs(10);

fn session(args: &[&str]) -> LineSession {
    LineSession::spawn(peer(args)).unwrap()
}

async fn exited(session: &mut LineSession) {
    let error = session.read_line(WAIT).await.unwrap_err();
    assert_eq!(error.code, ErrorCode::NotRunning);
}

#[tokio::test]
async fn lines_make_a_round_trip_including_unicode() {
    let mut chat = session(&["echo"]);
    for line in ["ping", "olá, mundo 🔑", "", "  spaces  kept  "] {
        chat.send_line(line).await.unwrap();
        assert_eq!(chat.read_line(WAIT).await.unwrap(), format!("got:{line}"));
    }
}

#[tokio::test]
async fn many_lines_queued_before_reading_come_back_in_order() {
    let mut chat = session(&["echo"]);
    for n in 0..200 {
        chat.send_line(&format!("line {n}")).await.unwrap();
    }
    for n in 0..200 {
        assert_eq!(chat.read_line(WAIT).await.unwrap(), format!("got:line {n}"));
    }
}

#[tokio::test]
async fn a_timeout_keeps_the_bytes_that_did_arrive() {
    let mut chat = session(&["partial", "1500"]);
    let error = chat.read_line(Duration::from_millis(300)).await.unwrap_err();
    assert_eq!(error.code, ErrorCode::Timeout);
    assert!(error.retryable && error.params.is_empty());
    // "part" arrived before the timeout and must not be lost when reading resumes.
    assert_eq!(chat.read_line(WAIT).await.unwrap(), "partial");
}

#[tokio::test]
async fn a_silent_child_times_out_and_the_session_stays_usable() {
    let mut chat = session(&["echo"]);
    let started = Instant::now();
    let error = chat.read_line(Duration::from_millis(200)).await.unwrap_err();
    assert_eq!(error.code, ErrorCode::Timeout);
    assert!(started.elapsed() < Duration::from_secs(5));
    chat.send_line("still here").await.unwrap();
    assert_eq!(chat.read_line(WAIT).await.unwrap(), "got:still here");
}

#[tokio::test]
async fn an_enormous_wait_is_clamped_not_a_panic() {
    let mut chat = session(&["echo"]);
    chat.send_line("x").await.unwrap();
    assert_eq!(chat.read_line(Duration::MAX).await.unwrap(), "got:x");
}

#[tokio::test]
async fn end_of_output_is_not_running_and_the_last_partial_line_comes_first() {
    let mut chat = session(&["exit", "0"]);
    exited(&mut chat).await;
    exited(&mut chat).await;

    let mut chat = session(&["print", "5"]);
    assert_eq!(chat.read_line(WAIT).await.unwrap(), "xxxxx");
    exited(&mut chat).await;
}

#[tokio::test]
async fn a_line_over_the_cap_is_a_protocol_error_that_ends_reading() {
    let mut chat = session(&["line", "5000"]).with_max_line(1000);
    let error = chat.read_line(WAIT).await.unwrap_err();
    assert_eq!(error.code, ErrorCode::ProtocolError);
    assert!(!error.retryable);
    let again = chat.read_line(Duration::from_millis(200)).await.unwrap_err();
    assert_eq!(again.code, ErrorCode::ProtocolError, "a broken stream stays broken");
}

#[tokio::test]
async fn the_default_cap_is_one_mebibyte_exactly() {
    assert_eq!(DEFAULT_MAX_LINE, 1 << 20);
    let mut fits = session(&["line", &DEFAULT_MAX_LINE.to_string()]);
    assert_eq!(fits.read_line(WAIT).await.unwrap().len(), DEFAULT_MAX_LINE);

    let mut over = session(&["line", &(DEFAULT_MAX_LINE + 1).to_string()]);
    let error = over.read_line(WAIT).await.unwrap_err();
    assert_eq!(error.code, ErrorCode::ProtocolError);
}

#[tokio::test]
async fn a_line_cap_beyond_the_hard_ceiling_is_clamped_to_it() {
    const CEILING: usize = 16 << 20;
    let mut fits = session(&["line", &CEILING.to_string()]).with_max_line(usize::MAX);
    assert_eq!(fits.read_line(WAIT).await.unwrap().len(), CEILING);

    let mut over = session(&["line", &(CEILING + 1).to_string()]).with_max_line(usize::MAX);
    let error = over.read_line(WAIT).await.unwrap_err();
    assert_eq!(error.code, ErrorCode::ProtocolError, "an absurd cap must not mean no cap");
}

#[tokio::test]
async fn a_line_that_would_break_the_framing_is_refused_and_nothing_is_sent() {
    let mut chat = session(&["echo"]).with_max_line(10);
    for bad in ["a\nb", "a\rb", "\n", "01234567890"] {
        let error = chat.send_line(bad).await.unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidInput, "{bad:?}");
    }
    chat.send_line("good").await.unwrap();
    assert_eq!(chat.read_line(WAIT).await.unwrap(), "got:good", "refused lines reached the child");
}

#[tokio::test]
async fn sending_to_a_child_that_left_is_not_running() {
    let mut chat = session(&["exit", "0"]);
    exited(&mut chat).await;
    let error = chat.send_line("anyone there?").await.unwrap_err();
    assert_eq!(error.code, ErrorCode::NotRunning);
}

#[tokio::test]
async fn kill_ends_the_child_and_is_idempotent() {
    let mut chat = session(&["sleep", "60000"]);
    chat.kill().await;
    chat.kill().await;
    exited(&mut chat).await;
    let error = chat.send_line("x").await.unwrap_err();
    assert_eq!(error.code, ErrorCode::NotRunning);
}

#[tokio::test]
async fn an_unread_stderr_flood_cannot_stall_the_child() {
    // 6 MB on stderr first, then four bytes on stdout: only a drained stderr lets them through.
    let mut chat = session(&["print-both", "4", "6000000"]);
    assert_eq!(chat.read_line(WAIT).await.unwrap(), "xxxx");
}

#[tokio::test]
async fn start_failures_are_reported_before_any_io() {
    let dir = tempfile::tempdir().unwrap();
    let missing = LineSession::spawn(CommandSpec::new(dir.path().join("missing.exe")));
    assert_eq!(missing.unwrap_err().code, ErrorCode::BinaryNotFound);

    let mut spec = peer(&["echo"]);
    spec.cwd = Some(dir.path().join("no-such-dir"));
    assert_eq!(LineSession::spawn(spec).unwrap_err().code, ErrorCode::SpawnFailed);
}

#[tokio::test]
async fn sessions_are_independent() {
    let mut sessions: Vec<_> = (0..4).map(|_| session(&["echo"])).collect();
    for round in 0..3 {
        for (n, chat) in sessions.iter_mut().enumerate() {
            chat.send_line(&format!("s{n} r{round}")).await.unwrap();
        }
        for (n, chat) in sessions.iter_mut().enumerate() {
            assert_eq!(chat.read_line(WAIT).await.unwrap(), format!("got:s{n} r{round}"));
        }
    }
}

#[tokio::test]
async fn debug_output_shows_no_conversation() {
    let mut chat = session(&["echo"]);
    chat.send_line("password=hunter2").await.unwrap();
    let text = format!("{chat:?}");
    assert!(text.contains("LineSession") && !text.contains("hunter2"), "{text}");
}

#[cfg(windows)]
mod windows {
    use super::*;
    use crate::common::peer::{is_alive, wait_for_liveness};

    async fn grandchild_pid(chat: &mut LineSession) -> u32 {
        let line = chat.read_line(WAIT).await.unwrap();
        let pid = line.strip_prefix("child:").unwrap().parse().unwrap();
        assert!(is_alive(pid).await, "the grandchild must be running");
        pid
    }

    #[tokio::test]
    async fn kill_ends_the_whole_process_tree() {
        let mut chat = session(&["grandchild"]);
        let pid = grandchild_pid(&mut chat).await;
        chat.kill().await;
        assert!(wait_for_liveness(pid, false).await, "the grandchild survived kill()");
    }

    #[tokio::test]
    async fn kill_reaches_the_grandchild_of_a_child_that_leaves_when_stdin_closes() {
        let mut chat = session(&["grandchild-until-eof"]);
        let pid = grandchild_pid(&mut chat).await;
        chat.kill().await;
        let message =
            "closing stdin before the tree kill let the root exit and orphan the grandchild";
        assert!(wait_for_liveness(pid, false).await, "{message}");
    }

    #[tokio::test]
    async fn dropping_the_session_ends_the_whole_process_tree() {
        let mut chat = session(&["grandchild"]);
        let pid = grandchild_pid(&mut chat).await;
        drop(chat);
        assert!(wait_for_liveness(pid, false).await, "the grandchild outlived its session");
    }
}
