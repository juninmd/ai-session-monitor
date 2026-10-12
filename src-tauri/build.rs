//! Build script: derives the ACL command permissions from the shared IPC contract and makes sure
//! the packaged `statusline-shim` sidecar exists, because tauri-build refuses to start without it.

use std::error::Error;
use std::io::Write;
use std::path::Path;
use std::{env, fs, io};

use tauri_build::{AppManifest, Attributes};

type BuildResult<T> = Result<T, Box<dyn Error>>;

const CONTRACT: &str = "../contract/ipc-commands.json";
const SHIM_STEM: &str = "statusline-shim";
/// A real executable is far larger; anything below is the debug placeholder or a truncated copy.
const MIN_SIDECAR_BYTES: u64 = 4096;
const PLACEHOLDER: &[u8] =
    b"placeholder: run `node src-tauri/scripts/prepare-sidecar.mjs` to package the real shim\n";

fn main() -> BuildResult<()> {
    directive(&format!("cargo:rerun-if-changed={CONTRACT}"))?;
    directive("cargo:rerun-if-changed=build.rs")?;

    let names = command_names(Path::new(CONTRACT))?;
    ensure_sidecar()?;

    // `AppManifest` wants `'static` slices; the process ends right after this call.
    let leaked: Vec<&'static str> =
        names.into_iter().map(|name| &*Box::leak(name.into_boxed_str())).collect();
    let leaked: &'static [&'static str] = Box::leak(leaked.into_boxed_slice());
    tauri_build::try_build(Attributes::new().app_manifest(AppManifest::new().commands(leaked)))?;
    Ok(())
}

/// Cargo reads build-script directives from stdout.
fn directive(line: &str) -> io::Result<()> {
    writeln!(io::stdout().lock(), "{line}")
}

/// Command names from the contract, each one a plain snake_case identifier because tauri-build
/// turns them into file names.
fn command_names(contract: &Path) -> BuildResult<Vec<String>> {
    let json: serde_json::Value = serde_json::from_str(&fs::read_to_string(contract)?)?;
    let list = json["commands"].as_array().ok_or("ipc-commands.json has no `commands` array")?;
    let mut names = Vec::with_capacity(list.len());
    for entry in list {
        let name = entry.as_str().ok_or("ipc-commands.json holds a non-string command")?;
        let valid = !name.is_empty()
            && name.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
        if !valid {
            return Err(format!("invalid command name in ipc-commands.json: {name:?}").into());
        }
        names.push(name.to_owned());
    }
    if names.is_empty() {
        return Err("ipc-commands.json lists no commands".into());
    }
    Ok(names)
}

fn is_real_binary(path: &Path) -> bool {
    fs::metadata(path).is_ok_and(|meta| meta.is_file() && meta.len() >= MIN_SIDECAR_BYTES)
}

/// Debug and check builds get a placeholder so a clean checkout compiles; a release build must
/// ship the real shim, so it fails loudly instead.
fn ensure_sidecar() -> BuildResult<()> {
    let triple = env::var("TARGET")?;
    let ext = if env::var("CARGO_CFG_TARGET_OS")? == "windows" { ".exe" } else { "" };
    let path = Path::new("binaries").join(format!("{SHIM_STEM}-{triple}{ext}"));
    directive(&format!("cargo:rerun-if-changed={}", path.display()))?;
    if is_real_binary(&path) {
        return Ok(());
    }
    if env::var("PROFILE").is_ok_and(|profile| profile == "release") {
        return Err(format!(
            "sidecar {} is missing or only a placeholder; run \
             `node src-tauri/scripts/prepare-sidecar.mjs` first",
            path.display()
        )
        .into());
    }
    if !path.exists() {
        fs::create_dir_all("binaries")?;
        fs::write(&path, PLACEHOLDER)?;
    }
    Ok(())
}
