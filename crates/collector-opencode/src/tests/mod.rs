//! Test support and suites. All fixtures are fabricated: no real profile, store or key is read.

mod collect_edges;
mod collect_flows;
mod fake_db;
mod fake_server;
mod flow_support;
mod gates;
mod local_files;
mod local_guard;
mod local_path;
mod local_sums;
mod local_support;
mod remote_http;
mod remote_parse;
mod remote_parse_edges;

use std::collections::BTreeMap;
use std::path::Path;

use usage_model::settings::OpencodeSettings;
use usage_model::{Ctx, HarnessView};
use usage_support::Secret;

use fake_db::{CANARY_CREDENTIAL, CANARY_PROMPT, NOW};

/// A fabricated key: if it shows up anywhere but the Authorization header, a test fails.
const KEY: &str = "fab-go-key-5c1e8a";

fn fixture(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join(name);
    std::fs::read_to_string(path).unwrap()
}

fn secret() -> Secret {
    Secret::new(KEY)
}

fn ctx(home: &Path) -> Ctx {
    Ctx::new(NOW, home.join("app-data"), home)
}

fn settings(local_consent: bool, data_dir: Option<&Path>) -> OpencodeSettings {
    OpencodeSettings {
        enabled: true,
        local_consent,
        data_dir: data_dir.map(|dir| dir.to_string_lossy().into_owned()),
    }
}

/// RFC 3339 with milliseconds, the way the service writes `resetsAt`.
fn iso(secs: i64) -> String {
    let (year, month, day) = crate::remote::month::civil_from_days(secs.div_euclid(86_400));
    let rest = secs.rem_euclid(86_400);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.000Z",
        rest / 3_600,
        rest % 3_600 / 60,
        rest % 60
    )
}

/// Neither the key nor the database canaries may appear anywhere in a view.
fn assert_clean(view: &HarnessView) {
    let json = serde_json::to_string(view).unwrap();
    let debug = format!("{view:?}");
    for needle in [KEY, CANARY_CREDENTIAL, CANARY_PROMPT] {
        assert!(!json.contains(needle), "{needle} leaked into the serialized view");
        assert!(!debug.contains(needle), "{needle} leaked into the debug output");
    }
}

/// File names, sizes and modification times of a directory: equal before and after means
/// nothing was created, deleted or written.
fn snapshot(dir: &Path) -> BTreeMap<String, (u64, std::time::SystemTime)> {
    std::fs::read_dir(dir)
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            let meta = entry.metadata().unwrap();
            (
                entry.file_name().to_string_lossy().into_owned(),
                (meta.len(), meta.modified().unwrap()),
            )
        })
        .collect()
}
