//! The real collector crates behind `app-core`'s `Collector` and `ClaudeShim` seams.

use std::path::Path;
use std::sync::Arc;

use app_core::{Adapters, ClaudeShim, CollectFuture, Collector, SecretStore};
use usage_model::settings::ClaudeSettings;
use usage_model::{Ctx, HarnessError, HarnessId, HarnessView, SecretKind, Settings, ShimStatus};

#[cfg(test)]
mod tests;

type Run = for<'a> fn(&'a Settings, &'a dyn SecretStore, &'a Ctx) -> CollectFuture<'a>;

/// One harness: its id and the function that reads it.
struct Wired {
    id: HarnessId,
    run: Run,
}

impl Collector for Wired {
    fn id(&self) -> HarnessId {
        self.id
    }

    fn collect<'a>(
        &'a self,
        settings: &'a Settings,
        secrets: &'a dyn SecretStore,
        ctx: &'a Ctx,
    ) -> CollectFuture<'a> {
        (self.run)(settings, secrets, ctx)
    }
}

fn codex<'a>(s: &'a Settings, _: &'a dyn SecretStore, ctx: &'a Ctx) -> CollectFuture<'a> {
    Box::pin(collector_codex::collect(&s.codex, ctx))
}

fn antigravity<'a>(s: &'a Settings, _: &'a dyn SecretStore, ctx: &'a Ctx) -> CollectFuture<'a> {
    Box::pin(collector_antigravity::collect(&s.antigravity, ctx))
}

/// The keyed collectors read their key here; a store failure becomes that tab's error.
fn opencode<'a>(s: &'a Settings, secrets: &'a dyn SecretStore, ctx: &'a Ctx) -> CollectFuture<'a> {
    Box::pin(async move {
        match secrets.get(SecretKind::Opencode) {
            Ok(key) => collector_opencode::collect(&s.opencode, key.as_ref(), ctx).await,
            Err(err) => HarnessView::error(HarnessId::Opencode, err),
        }
    })
}

fn kimi<'a>(s: &'a Settings, secrets: &'a dyn SecretStore, ctx: &'a Ctx) -> CollectFuture<'a> {
    Box::pin(async move {
        match secrets.get(SecretKind::Kimi) {
            Ok(key) => collector_kimi::collect(&s.kimi, key.as_ref(), ctx).await,
            Err(err) => HarnessView::error(HarnessId::Kimi, err),
        }
    })
}

fn zai<'a>(s: &'a Settings, secrets: &'a dyn SecretStore, ctx: &'a Ctx) -> CollectFuture<'a> {
    Box::pin(async move {
        match secrets.get(SecretKind::Zai) {
            Ok(key) => collector_zai::collect(&s.zai, key.as_ref(), ctx).await,
            Err(err) => HarnessView::error(HarnessId::Zai, err),
        }
    })
}

fn openrouter<'a>(
    s: &'a Settings,
    secrets: &'a dyn SecretStore,
    ctx: &'a Ctx,
) -> CollectFuture<'a> {
    Box::pin(async move {
        match secrets.get(SecretKind::Openrouter) {
            Ok(key) => collector_openrouter::collect(&s.openrouter, key.as_ref(), ctx).await,
            Err(err) => HarnessView::error(HarnessId::Openrouter, err),
        }
    })
}

fn deepseek<'a>(s: &'a Settings, secrets: &'a dyn SecretStore, ctx: &'a Ctx) -> CollectFuture<'a> {
    Box::pin(async move {
        match secrets.get(SecretKind::Deepseek) {
            Ok(key) => collector_deepseek::collect(&s.deepseek, key.as_ref(), ctx).await,
            Err(err) => HarnessView::error(HarnessId::Deepseek, err),
        }
    })
}

fn deepinfra<'a>(s: &'a Settings, secrets: &'a dyn SecretStore, ctx: &'a Ctx) -> CollectFuture<'a> {
    Box::pin(async move {
        match secrets.get(SecretKind::Deepinfra) {
            Ok(key) => collector_deepinfra::collect(&s.deepinfra, key.as_ref(), ctx).await,
            Err(err) => HarnessView::error(HarnessId::Deepinfra, err),
        }
    })
}

fn moonshot<'a>(s: &'a Settings, secrets: &'a dyn SecretStore, ctx: &'a Ctx) -> CollectFuture<'a> {
    Box::pin(async move {
        match secrets.get(SecretKind::Moonshot) {
            Ok(key) => collector_moonshot::collect(&s.moonshot, key.as_ref(), ctx).await,
            Err(err) => HarnessView::error(HarnessId::Moonshot, err),
        }
    })
}

fn novita<'a>(s: &'a Settings, secrets: &'a dyn SecretStore, ctx: &'a Ctx) -> CollectFuture<'a> {
    Box::pin(async move {
        match secrets.get(SecretKind::Novita) {
            Ok(key) => collector_novita::collect(&s.novita, key.as_ref(), ctx).await,
            Err(err) => HarnessView::error(HarnessId::Novita, err),
        }
    })
}

fn ollama<'a>(s: &'a Settings, secrets: &'a dyn SecretStore, ctx: &'a Ctx) -> CollectFuture<'a> {
    Box::pin(async move {
        match secrets.get(SecretKind::Ollama) {
            Ok(key) => collector_ollama::collect(&s.ollama, key.as_ref(), ctx).await,
            Err(err) => HarnessView::error(HarnessId::Ollama, err),
        }
    })
}

fn minimax<'a>(s: &'a Settings, secrets: &'a dyn SecretStore, ctx: &'a Ctx) -> CollectFuture<'a> {
    Box::pin(async move {
        match secrets.get(SecretKind::Minimax) {
            Ok(key) => collector_minimax::collect(&s.minimax, key.as_ref(), ctx).await,
            Err(err) => HarnessView::error(HarnessId::Minimax, err),
        }
    })
}

fn litellm<'a>(s: &'a Settings, secrets: &'a dyn SecretStore, ctx: &'a Ctx) -> CollectFuture<'a> {
    Box::pin(async move {
        match secrets.get(SecretKind::Litellm) {
            Ok(key) => collector_litellm::collect(&s.litellm, key.as_ref(), ctx).await,
            Err(err) => HarnessView::error(HarnessId::Litellm, err),
        }
    })
}

/// `settings` with the folder `CLAUDE_CONFIG_DIR` named (see `claude_dir`) when the user left
/// theirs blank; a folder the user typed always wins. The Claude collector reads no environment.
fn with_env_dir(settings: &ClaudeSettings, env_dir: Option<&str>) -> ClaudeSettings {
    let mut resolved = settings.clone();
    let blank = settings.config_dir.as_deref().is_none_or(|dir| dir.trim().is_empty());
    if blank && let Some(dir) = env_dir {
        resolved.config_dir = Some(dir.to_owned());
    }
    resolved
}

struct Claude {
    env_dir: Option<String>,
}

impl Collector for Claude {
    fn id(&self) -> HarnessId {
        HarnessId::Claude
    }

    fn collect<'a>(
        &'a self,
        settings: &'a Settings,
        _: &'a dyn SecretStore,
        ctx: &'a Ctx,
    ) -> CollectFuture<'a> {
        let claude = with_env_dir(&settings.claude, self.env_dir.as_deref());
        Box::pin(async move { collector_claude::collect(&claude, ctx).await })
    }
}

struct ClaudeCodeShim {
    env_dir: Option<String>,
}

impl ClaudeCodeShim {
    fn resolved(&self, settings: &ClaudeSettings) -> ClaudeSettings {
        with_env_dir(settings, self.env_dir.as_deref())
    }
}

impl ClaudeShim for ClaudeCodeShim {
    fn status(&self, settings: &ClaudeSettings, ctx: &Ctx) -> ShimStatus {
        collector_claude::shim_status(&self.resolved(settings), ctx)
    }

    fn install(
        &self,
        settings: &ClaudeSettings,
        ctx: &Ctx,
        shim_exe: &Path,
    ) -> Result<ShimStatus, HarnessError> {
        collector_claude::install_shim(&self.resolved(settings), ctx, shim_exe)
    }

    fn uninstall(&self, settings: &ClaudeSettings, ctx: &Ctx) -> Result<ShimStatus, HarnessError> {
        collector_claude::uninstall_shim(&self.resolved(settings), ctx)
    }

    fn refresh_copy(
        &self,
        settings: &ClaudeSettings,
        ctx: &Ctx,
        shim_exe: &Path,
    ) -> Result<(), HarnessError> {
        collector_claude::refresh_shim_copy(&self.resolved(settings), ctx, shim_exe)
    }
}

/// `claude_dir` is the folder `CLAUDE_CONFIG_DIR` named at startup, if any.
pub fn adapters(claude_dir: Option<String>) -> Adapters {
    let wired = |id, run: Run| -> Arc<dyn Collector> { Arc::new(Wired { id, run }) };
    let claude: Arc<dyn Collector> = Arc::new(Claude { env_dir: claude_dir.clone() });
    Adapters {
        // Tab order, same as `HarnessId::ALL`.
        collectors: vec![
            claude,
            wired(HarnessId::Codex, codex),
            wired(HarnessId::Antigravity, antigravity),
            wired(HarnessId::Opencode, opencode),
            wired(HarnessId::Kimi, kimi),
            wired(HarnessId::Zai, zai),
            wired(HarnessId::Openrouter, openrouter),
            wired(HarnessId::Deepseek, deepseek),
            wired(HarnessId::Deepinfra, deepinfra),
            wired(HarnessId::Moonshot, moonshot),
            wired(HarnessId::Novita, novita),
            wired(HarnessId::Ollama, ollama),
            wired(HarnessId::Minimax, minimax),
            wired(HarnessId::Litellm, litellm),
        ],
        claude_shim: Arc::new(ClaudeCodeShim { env_dir: claude_dir }),
    }
}
