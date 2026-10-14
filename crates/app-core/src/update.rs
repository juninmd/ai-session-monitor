//! Serial updater operations with safe events. No installer runs without verified bytes.

use std::future::Future;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tokio::sync::Mutex as AsyncMutex;
use usage_model::{ErrorCode, HarnessError, UpdateFailure, UpdatePhase, UpdateStatus};
use usage_support::http::MAX_RELEASE_BYTES;
use usage_support::update::{UpdateCandidate, parse_manifest, verify_installer};
use usage_support::{ClientOptions, SafeClient};

const MANIFEST: &str =
    "https://github.com/juninmd/ai-session-monitor/releases/latest/download/latest.json";
type Sink = Arc<dyn Fn(&UpdateStatus) + Send + Sync>;
type Progress = Arc<dyn Fn(u64, Option<u64>) + Send + Sync>;

struct State {
    status: UpdateStatus,
    candidate: Option<UpdateCandidate>,
    bytes: Option<Vec<u8>>,
}

pub struct UpdateManager {
    state: Mutex<State>,
    operation: AsyncMutex<()>,
    public_key: String,
    sink: Sink,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

fn client(seconds: u64) -> Result<SafeClient, HarnessError> {
    SafeClient::new(ClientOptions {
        timeout: Duration::from_secs(seconds),
        ..ClientOptions::default()
    })
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |t| t.as_secs().min(i64::MAX as u64) as i64)
}

impl UpdateManager {
    /// The sink must be quick, and must not synchronously re-enter this manager.
    pub fn new(
        version: &str,
        public_key: String,
        installable: bool,
        enabled: bool,
        sink: Sink,
    ) -> Arc<Self> {
        let mut status = UpdateStatus::new(version, installable);
        if !enabled {
            status.phase = UpdatePhase::Unavailable;
        }
        Arc::new(Self {
            state: Mutex::new(State { status, candidate: None, bytes: None }),
            operation: AsyncMutex::new(()),
            public_key,
            sink,
        })
    }

    pub fn status(&self) -> UpdateStatus {
        lock(&self.state).status.clone()
    }

    fn change(&self, work: impl FnOnce(&mut State)) -> UpdateStatus {
        let mut state = lock(&self.state);
        work(&mut state);
        let status = state.status.clone();
        (self.sink)(&status);
        status
    }

    pub async fn check(&self) -> UpdateStatus {
        self.check_with(async { client(30)?.get_release_bytes(MANIFEST, 1 << 20, |_, _| {}).await })
            .await
    }

    async fn check_with(
        &self,
        fetch: impl Future<Output = Result<Vec<u8>, HarnessError>>,
    ) -> UpdateStatus {
        let Ok(_operation) = self.operation.try_lock() else { return self.status() };
        if matches!(
            self.status().phase,
            UpdatePhase::Ready | UpdatePhase::Installing | UpdatePhase::Unavailable
        ) {
            return self.status();
        }
        let current = self
            .change(|state| {
                state.status.phase = UpdatePhase::Checking;
                state.status.failure = None;
                state.status.checked_at = Some(now());
            })
            .current_version;
        let result = fetch.await.and_then(|bytes| parse_manifest(&bytes, &current));
        self.change(|state| {
            state.bytes = None;
            match result {
                Ok(candidate) => {
                    state.status.available_version = candidate.as_ref().map(|c| c.version.clone());
                    state.status.notes = candidate.as_ref().and_then(|c| c.notes.clone());
                    state.status.phase = if candidate.is_some() {
                        UpdatePhase::Available
                    } else {
                        UpdatePhase::Current
                    };
                    state.candidate = candidate;
                    state.status.downloaded_bytes = 0;
                    state.status.total_bytes = None;
                }
                Err(_) => {
                    state.status.phase = UpdatePhase::Error;
                    state.status.failure = Some(UpdateFailure::Check);
                    // A failed check cannot leave an older candidate eligible for download.
                    state.candidate = None;
                    state.status.available_version = None;
                    state.status.notes = None;
                }
            }
        })
    }

    pub async fn download(self: &Arc<Self>) -> UpdateStatus {
        self.download_with(|candidate, progress| async move {
            client(180)?
                .get_release_bytes(&candidate.url, MAX_RELEASE_BYTES, move |read, total| {
                    progress(read, total)
                })
                .await
        })
        .await
    }

    async fn download_with<F, Fut>(self: &Arc<Self>, fetch: F) -> UpdateStatus
    where
        F: FnOnce(UpdateCandidate, Progress) -> Fut,
        Fut: Future<Output = Result<Vec<u8>, HarnessError>>,
    {
        let Ok(_operation) = self.operation.try_lock() else { return self.status() };
        let candidate = {
            let state = lock(&self.state);
            if !matches!(state.status.phase, UpdatePhase::Available | UpdatePhase::Error) {
                return state.status.clone();
            }
            state.candidate.clone()
        };
        let Some(candidate) = candidate else { return self.status() };
        self.change(|state| {
            state.bytes = None;
            state.status.phase = UpdatePhase::Downloading;
            state.status.failure = None;
            state.status.downloaded_bytes = 0;
            state.status.total_bytes = None;
        });
        let manager = Arc::clone(self);
        let last_progress = Mutex::new(Instant::now());
        let bytes = fetch(
            candidate.clone(),
            Arc::new(move |read, total| {
                let mut last = lock(&last_progress);
                if read != 0 && Some(read) != total && last.elapsed() < Duration::from_millis(100) {
                    return;
                }
                *last = Instant::now();
                manager.change(|state| {
                    state.status.downloaded_bytes = read;
                    state.status.total_bytes = total;
                });
            }),
        )
        .await;
        let result = bytes.map_err(|_| UpdateFailure::Download).and_then(|bytes| {
            verify_installer(&bytes, &candidate, &self.public_key)
                .map_err(|_| UpdateFailure::Signature)?;
            Ok(bytes)
        });
        self.change(|state| match result {
            Ok(bytes) => {
                state.status.downloaded_bytes = bytes.len() as u64;
                state.bytes = Some(bytes);
                state.status.phase = UpdatePhase::Ready;
            }
            Err(failure) => {
                state.status.phase = UpdatePhase::Error;
                state.status.failure = Some(failure);
            }
        })
    }

    /// Shell supplies the installation adapter. It receives only already authenticated bytes.
    pub async fn install<F, Fut>(&self, install: F) -> Result<UpdateStatus, HarnessError>
    where
        F: FnOnce(Vec<u8>) -> Fut,
        Fut: Future<Output = Result<(), HarnessError>>,
    {
        let Ok(_operation) = self.operation.try_lock() else { return Ok(self.status()) };
        let bytes = {
            let mut state = lock(&self.state);
            if !state.status.installable || state.status.phase != UpdatePhase::Ready {
                return Err(HarnessError::new(ErrorCode::InvalidInput, false));
            }
            state.bytes.take().ok_or_else(|| HarnessError::new(ErrorCode::Internal, false))?
        };
        self.change(|state| {
            state.status.phase = UpdatePhase::Installing;
        });
        let result = install(bytes).await;
        Ok(self.change(|state| {
            if result.is_err() {
                state.status.phase = UpdatePhase::Error;
                state.status.failure = Some(UpdateFailure::Install);
            }
        }))
    }
}

#[cfg(test)]
mod tests;
