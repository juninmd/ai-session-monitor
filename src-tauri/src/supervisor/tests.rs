use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::task::JoinHandle;
use tokio::time::Instant;

use super::*;

/// What one scripted run does as soon as it is spawned.
enum Run {
    Panic,
    Finish,
    Cancel,
}

type Starts = Arc<Mutex<Vec<Instant>>>;

/// A spawn function that plays `runs` in order and notes when each one started (virtual time).
fn scripted(runs: Vec<Run>) -> (impl FnMut() -> JoinHandle<()>, Starts) {
    let starts: Starts = Arc::default();
    let seen = Arc::clone(&starts);
    let mut runs: VecDeque<Run> = runs.into();
    let spawn = move || {
        seen.lock().expect("lock").push(Instant::now());
        match runs.pop_front().expect("the supervisor started more runs than the script has") {
            Run::Finish => tokio::spawn(async {}),
            Run::Panic => tokio::spawn(async { panic!("scripted panic") }),
            Run::Cancel => {
                let task = tokio::spawn(std::future::pending());
                task.abort();
                task
            }
        }
    };
    (spawn, starts)
}

/// Seconds between consecutive starts, rounded to whole seconds (the timer ticks in milliseconds).
fn gaps(starts: &Starts) -> Vec<u64> {
    let starts = starts.lock().expect("lock");
    starts.windows(2).map(|pair| (pair[1] - pair[0]).as_secs_f64().round() as u64).collect()
}

fn count(starts: &Starts) -> usize {
    starts.lock().expect("lock").len()
}

#[tokio::test(start_paused = true)]
async fn a_panic_is_followed_by_a_new_run_after_two_seconds() {
    let (spawn, starts) = scripted(vec![Run::Panic, Run::Finish]);
    supervise(spawn).await;
    assert_eq!(count(&starts), 2);
    assert_eq!(gaps(&starts), [2]);
}

#[tokio::test(start_paused = true)]
async fn the_wait_doubles_and_stops_at_thirty_seconds() {
    let mut runs: Vec<Run> = (0..7).map(|_| Run::Panic).collect();
    runs.push(Run::Finish);
    let (spawn, starts) = scripted(runs);
    supervise(spawn).await;
    assert_eq!(gaps(&starts), [2, 4, 8, 16, 30, 30, 30]);
}

#[tokio::test(start_paused = true)]
async fn a_normal_end_is_a_shutdown_and_is_not_restarted() {
    let (spawn, starts) = scripted(vec![Run::Finish]);
    let began = Instant::now();
    supervise(spawn).await;
    assert_eq!(count(&starts), 1);
    assert_eq!(began.elapsed(), Duration::ZERO, "no wait before giving up on a clean end");
}

#[tokio::test(start_paused = true)]
async fn a_cancelled_run_is_restarted_like_a_panicked_one() {
    let (spawn, starts) = scripted(vec![Run::Cancel, Run::Finish]);
    supervise(spawn).await;
    assert_eq!(gaps(&starts), [2]);
}

#[tokio::test(start_paused = true)]
async fn mixed_failures_share_one_growing_wait() {
    let (spawn, starts) = scripted(vec![Run::Panic, Run::Cancel, Run::Panic, Run::Finish]);
    supervise(spawn).await;
    assert_eq!(gaps(&starts), [2, 4, 8]);
}

#[tokio::test(start_paused = true)]
async fn a_task_that_runs_for_a_while_before_panicking_is_still_restarted() {
    let starts: Starts = Arc::default();
    let seen = Arc::clone(&starts);
    let mut attempt = 0;
    let spawn = move || {
        attempt += 1;
        seen.lock().expect("lock").push(Instant::now());
        let panic_at_end = attempt == 1;
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(3600)).await;
            assert!(!panic_at_end, "scripted panic after an hour");
        })
    };
    supervise(spawn).await;
    assert_eq!(count(&starts), 2);
    assert_eq!(gaps(&starts), [3602], "an hour of work, then the two second wait");
}

#[tokio::test]
async fn only_a_join_error_from_a_panic_or_a_cancel_is_abnormal() {
    let panicked = tokio::spawn(async { panic!("scripted panic") }).await;
    assert_eq!(abnormal(&panicked), Some("it panicked"));

    let task = tokio::spawn(std::future::pending::<()>());
    task.abort();
    assert_eq!(abnormal(&task.await), Some("it was cancelled"));

    assert_eq!(abnormal(&Ok(())), None);
}
