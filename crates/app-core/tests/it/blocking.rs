//! A collector that blocks its thread inside `poll` cannot be cancelled. `run` must give up on it
//! after a short grace instead of hanging with it. Real threads and real time: no paused clock.

use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use app_core::{
    Adapters, AppPaths, AppService, CollectFuture, Collector, EventSink, MemorySecretStore,
    SecretStore,
};
use usage_model::{Ctx, HarnessId, HarnessView, Settings};

use crate::collector::lock;
use crate::fake_shim::{FakeShim, installed};
use crate::sink::RecordingSink;

/// Longest the stuck thread waits to be released, so a broken test cannot hang the suite.
const STUCK_LIMIT: Duration = Duration::from_secs(20);
/// Well above the grace `run` allows, well below `STUCK_LIMIT`.
const RUN_MUST_END_WITHIN: Duration = Duration::from_secs(8);

struct Stuck {
    entered: mpsc::Sender<()>,
    release: Mutex<mpsc::Receiver<()>>,
}

impl Collector for Stuck {
    fn id(&self) -> HarnessId {
        HarnessId::Claude
    }

    fn collect<'a>(
        &'a self,
        _settings: &'a Settings,
        _secrets: &'a dyn SecretStore,
        _ctx: &'a Ctx,
    ) -> CollectFuture<'a> {
        Box::pin(async move {
            let _ = self.entered.send(());
            // No await point: the worker thread is held, so aborting the task cannot reach it.
            let _ = lock(&self.release).recv_timeout(STUCK_LIMIT);
            HarnessView::loading(HarnessId::Claude)
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn run_ends_after_a_short_grace_even_when_a_collector_blocks_its_thread() {
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let stuck = Stuck { entered: entered_tx, release: Mutex::new(release_rx) };
    let dir = tempfile::tempdir().unwrap_or_else(|e| panic!("tempdir: {e}"));
    let paths = AppPaths { data_dir: dir.path().join("data"), home_dir: dir.path().join("home") };
    let adapters = Adapters {
        collectors: vec![Arc::new(stuck)],
        claude_shim: FakeShim::answering(Ok(installed())),
    };
    let sink: Arc<dyn EventSink> = RecordingSink::new();
    let secrets: Arc<dyn SecretStore> = Arc::new(MemorySecretStore::new());
    let service =
        AppService::new(paths, secrets, sink, adapters).unwrap_or_else(|e| panic!("{e:?}"));

    let runner = tokio::spawn(Arc::clone(&service).run());
    let parked =
        tokio::task::spawn_blocking(move || entered_rx.recv_timeout(Duration::from_secs(10))).await;
    assert!(matches!(parked, Ok(Ok(()))), "the collector never started");

    let asked = Instant::now();
    service.shutdown();
    let ended = tokio::time::timeout(RUN_MUST_END_WITHIN, runner).await;
    let waited = asked.elapsed();
    // Frees the worker thread first, so a failure below cannot leave the runtime stuck at drop.
    let _ = release_tx.send(());

    assert!(ended.is_ok(), "run() hung behind a collector it could not cancel ({waited:?})");
    assert!(waited < STUCK_LIMIT, "it ended only because the collector was released");
}
