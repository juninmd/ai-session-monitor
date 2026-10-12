//! The rest of the command surface: defaults, persistence across starts, the clock, Debug.

use std::sync::Arc;

use app_core::{Adapters, AppPaths, AppService, MemorySecretStore};
use usage_model::Settings;

use crate::fake_shim::{FakeShim, installed};
use crate::rig::{rig, seconds};
use crate::sink::RecordingSink;
use crate::views::BASE;

fn second_start(paths: AppPaths) -> Arc<AppService> {
    let adapters =
        Adapters { collectors: Vec::new(), claude_shim: FakeShim::answering(Ok(installed())) };
    let started =
        AppService::new(paths, Arc::new(MemorySecretStore::new()), RecordingSink::new(), adapters);
    started.unwrap_or_else(|e| panic!("second start failed: {e:?}"))
}

#[tokio::test(start_paused = true)]
async fn a_fresh_install_has_the_defaults_and_writes_no_file_until_the_first_save() {
    let rig = rig().build();
    let mut defaults = Settings::default();
    defaults.normalize();

    assert_eq!(rig.service.settings(), defaults);
    assert!(!rig.settings_file().exists());
    rig.edit(|s| s.general.notify_on_out = true);
    assert!(rig.settings_file().exists());
}

#[tokio::test(start_paused = true)]
async fn saved_settings_are_what_the_next_start_loads() {
    let rig = rig().build();
    let target = std::env::temp_dir().join("fabricated-codex");
    let saved = rig.edit(|s| {
        s.general.interval_secs = 120;
        s.codex.binary_path = Some(target.to_string_lossy().into_owned());
    });

    let paths = AppPaths { data_dir: rig.data_dir.clone(), home_dir: rig.home_dir.clone() };
    assert_eq!(second_start(paths).settings(), saved);
}

#[tokio::test(start_paused = true)]
async fn the_snapshot_time_follows_the_injected_clock() {
    let rig = rig().start().await;
    assert_eq!(rig.service.snapshot().generated_at, BASE);

    rig.at(seconds(125)).await;
    assert_eq!(rig.service.snapshot().generated_at, BASE + 125);
}

#[tokio::test(start_paused = true)]
async fn debug_output_shows_no_internals() {
    let rig = rig().build();

    assert_eq!(format!("{:?}", rig.service), "AppService { .. }");
}
