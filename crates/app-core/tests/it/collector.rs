//! A scriptable `Collector`: what it returns per call, and what each call saw.

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use app_core::{CollectFuture, Collector, SecretStore};
use tokio::time::Instant;
use usage_model::{Ctx, HarnessId, HarnessView, SecretKind, Settings};

pub fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[derive(Clone)]
pub enum Step {
    View(HarnessView),
    /// Answers after the delay (virtual time).
    After(Duration, HarnessView),
    /// Never answers; only the scheduler's deadline or a cancellation ends it.
    Hang,
    Panic,
}

/// What one collector call saw.
#[derive(Clone, Debug)]
pub struct Call {
    pub at: Instant,
    pub settings: Settings,
    pub litellm_key: Option<String>,
    pub opencode_key: Option<String>,
    pub kimi_key: Option<String>,
    pub zai_key: Option<String>,
    pub openrouter_key: Option<String>,
    pub deepseek_key: Option<String>,
    pub now_secs: i64,
    pub data_dir: PathBuf,
    pub home_dir: PathBuf,
}

pub struct FakeCollector {
    id: HarnessId,
    plan: Box<dyn Fn(usize) -> Step + Send + Sync>,
    calls: Mutex<Vec<Call>>,
    in_flight: Arc<AtomicUsize>,
}

impl FakeCollector {
    /// `plan` gets the zero-based index of the call.
    pub fn new(id: HarnessId, plan: impl Fn(usize) -> Step + Send + Sync + 'static) -> Arc<Self> {
        let (calls, in_flight) = (Mutex::new(Vec::new()), Arc::new(AtomicUsize::new(0)));
        Arc::new(Self { id, plan: Box::new(plan), calls, in_flight })
    }

    pub fn always(id: HarnessId, step: Step) -> Arc<Self> {
        Self::new(id, move |_| step.clone())
    }

    /// The steps in order, then `then` for every later call.
    pub fn scripted(id: HarnessId, steps: Vec<Step>, then: Step) -> Arc<Self> {
        Self::new(id, move |n| steps.get(n).unwrap_or(&then).clone())
    }

    pub fn calls(&self) -> Vec<Call> {
        lock(&self.calls).clone()
    }

    pub fn count(&self) -> usize {
        lock(&self.calls).len()
    }

    /// Calls started that neither finished nor were cancelled.
    pub fn in_flight(&self) -> usize {
        self.in_flight.load(Ordering::SeqCst)
    }

    /// Start of every call, in milliseconds since `start`.
    pub fn offsets_ms(&self, start: Instant) -> Vec<u128> {
        self.calls().iter().map(|call| call.at.duration_since(start).as_millis()).collect()
    }
}

/// Counts a call as in flight until its future is dropped, however that happens.
struct InFlight(Arc<AtomicUsize>);

impl Drop for InFlight {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

impl Collector for FakeCollector {
    fn id(&self) -> HarnessId {
        self.id
    }

    fn collect<'a>(
        &'a self,
        settings: &'a Settings,
        secrets: &'a dyn SecretStore,
        ctx: &'a Ctx,
    ) -> CollectFuture<'a> {
        let key = |kind| secrets.get(kind).ok().flatten().map(|s| s.expose().to_owned());
        let index = {
            let mut calls = lock(&self.calls);
            calls.push(Call {
                at: Instant::now(),
                settings: settings.clone(),
                litellm_key: key(SecretKind::Litellm),
                opencode_key: key(SecretKind::Opencode),
                kimi_key: key(SecretKind::Kimi),
                zai_key: key(SecretKind::Zai),
                openrouter_key: key(SecretKind::Openrouter),
                deepseek_key: key(SecretKind::Deepseek),
                now_secs: ctx.now_secs,
                data_dir: ctx.data_dir.clone(),
                home_dir: ctx.home_dir.clone(),
            });
            calls.len() - 1
        };
        let step = (self.plan)(index);
        self.in_flight.fetch_add(1, Ordering::SeqCst);
        let guard = InFlight(Arc::clone(&self.in_flight));
        Box::pin(async move {
            let _guard = guard;
            match step {
                Step::View(view) => view,
                Step::After(delay, view) => {
                    tokio::time::sleep(delay).await;
                    view
                }
                Step::Hang => std::future::pending().await,
                Step::Panic => panic!("fake collector failure"),
            }
        })
    }
}
