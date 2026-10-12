//! Polls are independent: the scheduler may overlap them with other sources' polls, cancel one
//! at its hard timeout, and start the next as if nothing happened.

use std::time::{Duration, Instant};

use collector_litellm::collect;
use serde_json::json;
use tempfile::TempDir;
use usage_model::settings::LitellmScope;
use usage_model::{Ctx, ViewState};
use usage_support::Secret;

use crate::common::{Fake, NOW, Reply, SECRET, assert_ready, run, scoped, settings, window};

/// Answers `user-N` with a record whose spend is N, so a mixed-up answer is visible.
async fn numbered_proxy(delay: Duration) -> Fake {
    Fake::start(move |seen| {
        let id = seen.param("user_id").unwrap_or_default();
        let spend: f64 = id.strip_prefix("user-").and_then(|n| n.parse().ok()).unwrap_or(-1.0);
        let record = json!({ "user_id": id, "user_info": { "user_id": id, "spend": spend, "max_budget": 100.0 } });
        Reply::json(&record.to_string()).after(delay)
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn overlapping_polls_each_get_their_own_answer() {
    let fake = numbered_proxy(Duration::ZERO).await;
    let polls: Vec<_> = (0..24)
        .map(|n| {
            let settings = scoped(&fake.base(), LitellmScope::User, &format!("user-{n}"));
            tokio::spawn(async move { (n, run(&settings).await) })
        })
        .collect();
    for poll in polls {
        let (n, view) = poll.await.unwrap();
        assert_ready(&view);
        let amount = window(&view, "budget").amount.as_ref().unwrap().used;
        assert_eq!(amount, f64::from(n), "poll {n} read another poll's record");
    }
    assert_eq!(fake.requests().len(), 24);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn polls_do_not_wait_for_each_other() {
    let fake = numbered_proxy(Duration::from_millis(700)).await;
    let started = Instant::now();
    let polls: Vec<_> = (0..8)
        .map(|n| {
            let settings = scoped(&fake.base(), LitellmScope::User, &format!("user-{n}"));
            tokio::spawn(async move { run(&settings).await.state })
        })
        .collect();
    for poll in polls {
        assert_eq!(poll.await.unwrap(), ViewState::Ready);
    }
    let elapsed = started.elapsed();
    assert!(elapsed < Duration::from_millis(4_000), "8 polls of 0.7 s ran one by one: {elapsed:?}");
}

#[tokio::test]
async fn a_cancelled_poll_leaves_nothing_behind() {
    let fake = numbered_proxy(Duration::from_secs(30)).await;
    let data = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    let ctx = Ctx::new(NOW, data.path(), home.path());
    let settings = scoped(&fake.base(), LitellmScope::User, "user-1");
    let key = Secret::new(SECRET);

    // What the scheduler does at its hard timeout: drop the future mid-request.
    let cancelled =
        tokio::time::timeout(Duration::from_millis(300), collect(&settings, Some(&key), &ctx))
            .await;
    assert!(cancelled.is_err(), "the poll should still have been waiting for the proxy");
    assert_eq!(fake.requests().len(), 1, "the request had been sent");

    // The next poll is not affected by the abandoned one.
    let fast = numbered_proxy(Duration::ZERO).await;
    let again = scoped(&fast.base(), LitellmScope::User, "user-1");
    assert_ready(&run(&again).await);
    assert_eq!(std::fs::read_dir(data.path()).unwrap().count(), 0);
    assert_eq!(std::fs::read_dir(home.path()).unwrap().count(), 0);
}

#[tokio::test]
async fn repeated_polls_are_idempotent() {
    let fake = Fake::routes([("/key/info", Reply::json(fixture!("key_with_budget.json")))]).await;
    let settings = settings(&fake.base());
    let first = run(&settings).await;
    for _ in 0..5 {
        assert_eq!(run(&settings).await, first, "same input, same view");
    }
}

#[tokio::test]
async fn a_failing_poll_does_not_poison_the_next_one() {
    let healthy =
        Fake::routes([("/key/info", Reply::json(fixture!("key_with_budget.json")))]).await;
    let broken = Fake::routes([("/key/info", Reply::json_status(500, "{}"))]).await;
    for _ in 0..3 {
        assert_eq!(run(&settings(&broken.base())).await.state, ViewState::Error);
        assert_ready(&run(&settings(&healthy.base())).await);
    }
}
