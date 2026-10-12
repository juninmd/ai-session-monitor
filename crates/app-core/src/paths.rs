use std::path::PathBuf;

/// Where the app keeps its files. The shell fills it from Tauri's path resolver; tests use temp dirs.
#[derive(Clone, Debug)]
pub struct AppPaths {
    /// `%LOCALAPPDATA%\dev.aisessionmonitor.tray`: settings, the Claude shim and its data.
    pub data_dir: PathBuf,
    /// `%USERPROFILE%`: where the harnesses keep their own state.
    pub home_dir: PathBuf,
}

impl AppPaths {
    pub fn settings_file(&self) -> PathBuf {
        self.data_dir.join("settings.json")
    }

    pub fn history_file(&self) -> PathBuf {
        self.data_dir.join("history.json")
    }
}
