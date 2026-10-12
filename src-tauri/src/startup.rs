//! Startup failures. A release build has no console, so a failure that stops the app has to reach
//! the user another way: a toast that names the step that failed, then a non-zero exit.

use std::fmt::Display;
use std::time::Duration;

use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;
use usage_model::ErrorCode;

use crate::diag;

#[cfg(test)]
mod tests;

/// Process exit code of an app that could not start.
pub const EXIT_CODE: i32 = 1;
/// The notification plugin posts the toast from a background task; exiting at once could lose it.
const TOAST_GRACE: Duration = Duration::from_secs(2);
const TITLE: &str = "AI Session Monitor não iniciou";

/// The step that stopped startup. The underlying error can hold paths and OS text, so only the
/// step reaches the toast; the details go to stderr.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    /// The per-user folders could not be resolved.
    Paths,
    /// The data folder could not be created.
    DataDir,
    /// The flyout window does not exist.
    Window,
    /// The tray icon could not be created.
    Tray,
    /// `AppService::new` refused to start.
    Service(ErrorCode),
}

pub struct Notice {
    pub title: &'static str,
    pub body: String,
}

impl Reason {
    /// Stable snake_case name, in the vocabulary `ErrorCode` uses on the wire.
    pub fn code(self) -> String {
        match self {
            Self::Paths => "paths".to_owned(),
            Self::DataDir => "data_dir".to_owned(),
            Self::Window => "window".to_owned(),
            Self::Tray => "tray".to_owned(),
            Self::Service(code) => wire_name(code),
        }
    }

    /// The toast, in pt-BR. Built from the reason alone, so nothing the OS reported can leak in.
    pub fn notice(self) -> Notice {
        let what = match self {
            Self::Paths => "Não foi possível localizar as pastas do seu perfil",
            Self::DataDir => "Não foi possível criar a pasta de dados do aplicativo",
            Self::Window => "A janela do painel não foi criada",
            Self::Tray => "Não foi possível criar o ícone na bandeja",
            Self::Service(ErrorCode::SettingsUnreadable) => {
                "O arquivo de configurações está ilegível e não pôde ser movido para recomeçar"
            }
            Self::Service(_) => "O serviço de monitoramento não pôde ser iniciado",
        };
        Notice {
            title: TITLE,
            body: format!("{what} (código: {}). O aplicativo vai fechar.", self.code()),
        }
    }

    /// Writes the cause to stderr, for runs from a terminal, and hands the reason back for `?`.
    pub fn because(self, cause: &dyn Display) -> Self {
        diag::warn("cannot start", &format_args!("{}: {cause}", self.code()));
        self
    }
}

fn wire_name(code: ErrorCode) -> String {
    match serde_json::to_value(code) {
        Ok(serde_json::Value::String(name)) => name,
        _ => "internal".to_owned(),
    }
}

/// Tells the user, then ends the process with `EXIT_CODE`. Returning an error from the setup hook
/// instead would make Tauri panic, which nobody sees without a console.
pub fn abort(app: &AppHandle, reason: Reason) {
    let notice = reason.notice();
    if let Err(err) = app.notification().builder().title(notice.title).body(&notice.body).show() {
        diag::warn("cannot show the startup failure", &err);
    }
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(TOAST_GRACE).await;
        app.exit(EXIT_CODE);
    });
}
