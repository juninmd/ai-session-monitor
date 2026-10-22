//! The rig: a service wired to fakes inside a temp dir, plus helpers to move virtual time.
//! Tests run on tokio's paused clock, so minutes of scheduling take milliseconds.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use app_core::MemorySecretStore;
use app_core::{Adapters, AppPaths, AppService, Clock, Collector, SecretStore};
use tempfile::TempDir;
use tokio::task::JoinHandle;
use tokio::time::Instant;
use usage_model::{HarnessError, HarnessId, Settings};

use crate::collector::{FakeCollector, Step};
use crate::doubles::VirtualClock;
use crate::fake_shim::{FakeShim, installed};
use crate::sink::RecordingSink;
use crate::views::ready;

/// Lets everything runnable at the current virtual instant run; time does not move.
pub async fn settle() {
    for _ in 0..8 {
        tokio::task::yield_now().await;
    }
}

/// Whole seconds as the milliseconds `FakeCollector::offsets_ms` reports.
pub fn secs(list: &[u64]) -> Vec<u128> {
    list.iter().map(|s| u128::from(*s) * 1000).collect()
}

pub fn seconds(n: u64) -> Duration {
    Duration::from_secs(n)
}

pub struct Rig {
    pub service: Arc<AppService>,
    pub sink: Arc<RecordingSink>,
    pub secrets: Arc<dyn SecretStore>,
    pub shim: Arc<FakeShim>,
    /// The virtual instant the rig was built at; scheduling offsets count from here.
    pub start: Instant,
    pub data_dir: PathBuf,
    pub home_dir: PathBuf,
    fakes: Vec<Arc<FakeCollector>>,
    runner: Option<JoinHandle<()>>,
    _dir: Option<TempDir>,
}

impl Rig {
    pub fn fake(&self, id: HarnessId) -> &Arc<FakeCollector> {
        let found = self.fakes.iter().find(|fake| fake.id() == id);
        found.unwrap_or_else(|| panic!("the rig has no collector for {id:?}"))
    }

    /// Starts the scheduler loop on the runtime and lets it run its first pass.
    pub async fn run(&mut self) {
        self.runner = Some(tokio::spawn(Arc::clone(&self.service).run()));
        settle().await;
    }

    /// Moves virtual time to `offset` after the start, then lets the consequences play out.
    pub async fn at(&self, offset: Duration) {
        tokio::time::sleep_until(self.start + offset).await;
        settle().await;
    }

    pub async fn after(&self, wait: Duration) {
        tokio::time::sleep(wait).await;
        settle().await;
    }

    pub fn settings_file(&self) -> PathBuf {
        self.data_dir.join("settings.json")
    }

    /// The settings as they are on disk right now.
    pub fn saved_settings(&self) -> Settings {
        let bytes = std::fs::read(self.settings_file()).unwrap_or_default();
        serde_json::from_slice(&bytes).unwrap_or_else(|e| panic!("settings.json: {e}"))
    }

    /// Takes the running scheduler task, to await its end.
    pub fn take_runner(&mut self) -> JoinHandle<()> {
        self.runner.take().unwrap_or_else(|| panic!("the rig is not running"))
    }

    /// True when the scheduler task has ended (or was never started).
    pub fn runner_finished(&self) -> bool {
        self.runner.as_ref().is_none_or(JoinHandle::is_finished)
    }

    /// Saves the current settings changed by `edit`; panics when the service rejects them.
    pub fn edit(&self, edit: impl FnOnce(&mut Settings)) -> Settings {
        let mut settings = self.service.settings();
        edit(&mut settings);
        self.service.save_settings(settings).unwrap_or_else(|e| panic!("save rejected: {e:?}"))
    }

    /// Every file under the data dir as text, for "this never reached the disk" checks.
    pub fn data_text(&self) -> String {
        let mut text = String::new();
        let mut pending = vec![self.data_dir.clone()];
        while let Some(dir) = pending.pop() {
            for entry in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
                let path = entry.path();
                if path.is_dir() {
                    pending.push(path);
                } else {
                    text.push_str(&String::from_utf8_lossy(
                        &std::fs::read(&path).unwrap_or_default(),
                    ));
                }
            }
        }
        text
    }
}

#[derive(Default)]
pub struct RigBuilder {
    dir: Option<PathBuf>,
    settings: Option<Settings>,
    raw_settings: Option<Vec<u8>>,
    files: Vec<(String, Vec<u8>)>,
    overrides: Vec<Arc<FakeCollector>>,
    omitted: Vec<HarnessId>,
    secrets: Option<Arc<dyn SecretStore>>,
    clock: Option<Arc<dyn Clock>>,
    shim: Option<Arc<FakeShim>>,
    sink: Option<Arc<RecordingSink>>,
    pristine: bool,
}

/// The settings every rig starts from: the pacing and alerts that react to the readings are off,
/// so the older scheduling tests keep their fixed cadence; tests of those features switch them on.
pub fn steady() -> Settings {
    let mut settings = Settings::default();
    settings.general.adaptive = false;
    settings.general.language = usage_model::settings::Language::PtBr;
    settings.general.alert_thresholds.clear();
    settings
}

pub fn rig() -> RigBuilder {
    RigBuilder::default()
}

impl RigBuilder {
    /// Works inside a directory the test owns, so it can inspect it after a failed start.
    pub fn in_dir(mut self, dir: &Path) -> Self {
        self.dir = Some(dir.to_path_buf());
        self
    }

    /// Changes the defaults; the result is written as the settings file before the service loads.
    pub fn settings(mut self, edit: impl FnOnce(&mut Settings)) -> Self {
        let mut settings = self.settings.take().unwrap_or_else(steady);
        edit(&mut settings);
        self.settings = Some(settings);
        self
    }

    /// Starts with no settings file at all, so the service runs on the real defaults.
    pub fn pristine(mut self) -> Self {
        self.pristine = true;
        self
    }

    pub fn raw_settings(mut self, bytes: &[u8]) -> Self {
        self.raw_settings = Some(bytes.to_vec());
        self
    }

    /// Another file in the data dir before the service starts.
    pub fn file(mut self, name: &str, bytes: &[u8]) -> Self {
        self.files.push((name.to_owned(), bytes.to_vec()));
        self
    }

    pub fn collector(mut self, fake: Arc<FakeCollector>) -> Self {
        self.overrides.push(fake);
        self
    }

    pub fn without_collector(mut self, id: HarnessId) -> Self {
        self.omitted.push(id);
        self
    }

    pub fn secret_store(mut self, store: Arc<dyn SecretStore>) -> Self {
        self.secrets = Some(store);
        self
    }

    pub fn clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = Some(clock);
        self
    }

    pub fn shim(mut self, shim: Arc<FakeShim>) -> Self {
        self.shim = Some(shim);
        self
    }

    pub fn sink(mut self, sink: Arc<RecordingSink>) -> Self {
        self.sink = Some(sink);
        self
    }

    pub fn build(self) -> Rig {
        self.try_build().unwrap_or_else(|error| panic!("service did not start: {error:?}"))
    }

    pub fn try_build(self) -> Result<Rig, HarnessError> {
        let (owned, base) = match &self.dir {
            Some(path) => (None, path.clone()),
            None => {
                let dir = tempfile::tempdir().unwrap_or_else(|e| panic!("tempdir: {e}"));
                let base = dir.path().to_path_buf();
                (Some(dir), base)
            }
        };
        let (data_dir, home_dir) = (base.join("data"), base.join("home"));
        for path in [&data_dir, &home_dir] {
            std::fs::create_dir_all(path).unwrap_or_else(|e| panic!("mkdir: {e}"));
        }
        let file = data_dir.join("settings.json");
        let content = match (&self.raw_settings, &self.settings) {
            (Some(raw), _) => Some(raw.clone()),
            (None, Some(settings)) => serde_json::to_vec_pretty(settings).ok(),
            (None, None) if self.pristine => None,
            (None, None) => serde_json::to_vec_pretty(&steady()).ok(),
        };
        if let Some(bytes) = content {
            std::fs::write(&file, bytes).unwrap_or_else(|e| panic!("write settings: {e}"));
        }
        for (name, bytes) in &self.files {
            std::fs::write(data_dir.join(name), bytes)
                .unwrap_or_else(|e| panic!("write {name}: {e}"));
        }
        let fakes: Vec<Arc<FakeCollector>> = HarnessId::ALL
            .into_iter()
            .filter(|id| !self.omitted.contains(id))
            .map(|id| {
                let custom = self.overrides.iter().find(|fake| fake.id() == id);
                custom
                    .cloned()
                    .unwrap_or_else(|| FakeCollector::always(id, Step::View(ready(id, 10.0))))
            })
            .collect();
        let shim = self.shim.unwrap_or_else(|| FakeShim::answering(Ok(installed())));
        let sink = self.sink.unwrap_or_default();
        let secrets = self.secrets.unwrap_or_else(|| Arc::new(MemorySecretStore::new()));
        let clock = self.clock.unwrap_or_else(|| Arc::new(VirtualClock::new()));
        let adapters = Adapters {
            collectors: fakes.iter().map(|f| Arc::clone(f) as Arc<dyn Collector>).collect(),
            claude_shim: shim.clone(),
        };
        let paths = AppPaths { data_dir: data_dir.clone(), home_dir: home_dir.clone() };
        let service =
            AppService::with_clock(paths, secrets.clone(), sink.clone(), adapters, clock)?;
        Ok(Rig {
            service,
            sink,
            secrets,
            shim,
            start: Instant::now(),
            data_dir,
            home_dir,
            fakes,
            runner: None,
            _dir: owned,
        })
    }

    /// Builds the rig and starts the scheduler.
    pub async fn start(self) -> Rig {
        let mut rig = self.build();
        rig.run().await;
        rig
    }
}
