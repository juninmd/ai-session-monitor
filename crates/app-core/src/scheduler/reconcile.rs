//! Reacting to a settings change: which harnesses stop, start, restart or are only re-timed.

use std::sync::Arc;
use std::time::Duration;

use tokio::time::Instant;
use usage_model::{HarnessId, Settings};

use super::Scheduler;
use crate::timing;

impl Scheduler {
    /// Applies new settings: switched-off harnesses stop, switched-on ones start, a changed
    /// section of a running harness is read again, and a changed interval re-times the next call.
    pub(super) fn reconcile(&mut self) {
        let new = Arc::clone(&self.settings.borrow_and_update());
        let old = std::mem::replace(&mut self.applied, Arc::clone(&new));
        let switch_ons = self.shared.with_hub(|hub| {
            hub.reconcile_enabled(&new);
            hub.switch_ons()
        });
        let now = Instant::now();
        let switched_on = std::mem::replace(&mut self.seen_switch_ons, switch_ons);
        for (idx, seen) in switched_on.into_iter().enumerate() {
            let id = self.slots[idx].id;
            let enabled = new.is_enabled(id);
            self.slots[idx].enabled = enabled;
            if !enabled {
                self.cancel(idx);
                let slot = &mut self.slots[idx];
                slot.due = None;
                slot.reset_failures();
            } else if seen != switch_ons[idx]
                || !old.is_enabled(id)
                || section_changed(id, &old, &new)
            {
                self.slots[idx].reset_failures();
                self.restart(idx, now);
            } else {
                let (mut before, after) = (timing::cadence(id, &old), timing::cadence(id, &new));
                if old.general.adaptive && !new.general.adaptive {
                    before = Duration::ZERO; // a stretched wait snaps back to the base cadence
                }
                self.slots[idx].retime(before, after, now);
            }
        }
    }
}

fn section_changed(id: HarnessId, old: &Settings, new: &Settings) -> bool {
    match id {
        HarnessId::Claude => old.claude != new.claude,
        HarnessId::Codex => old.codex != new.codex,
        HarnessId::Cursor => old.cursor != new.cursor,
        HarnessId::Copilot => old.copilot != new.copilot,
        HarnessId::Antigravity => old.antigravity != new.antigravity,
        HarnessId::Opencode => old.opencode != new.opencode,
        HarnessId::Kimi => old.kimi != new.kimi,
        HarnessId::Zai => old.zai != new.zai,
        HarnessId::Openrouter => old.openrouter != new.openrouter,
        HarnessId::Deepseek => old.deepseek != new.deepseek,
        HarnessId::Deepinfra => old.deepinfra != new.deepinfra,
        HarnessId::Moonshot => old.moonshot != new.moonshot,
        HarnessId::Novita => old.novita != new.novita,
        HarnessId::Ollama => old.ollama != new.ollama,
        HarnessId::Minimax => old.minimax != new.minimax,
        HarnessId::Litellm => old.litellm != new.litellm,
    }
}
