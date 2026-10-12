import type { HarnessId, Settings } from '../lib/contract';

/** Typed per-harness patches: a computed key would widen the result and lose the contract. */
export function withEnabled(settings: Settings, id: HarnessId, enabled: boolean): Settings {
  switch (id) {
    case 'claude':
      return { ...settings, claude: { ...settings.claude, enabled } };
    case 'codex':
      return { ...settings, codex: { ...settings.codex, enabled } };
    case 'antigravity':
      return { ...settings, antigravity: { ...settings.antigravity, enabled } };
    case 'opencode':
      return { ...settings, opencode: { ...settings.opencode, enabled } };
    case 'kimi':
      return { ...settings, kimi: { ...settings.kimi, enabled } };
    case 'zai':
      return { ...settings, zai: { ...settings.zai, enabled } };
    case 'litellm':
      return { ...settings, litellm: { ...settings.litellm, enabled } };
  }
}

export function isEnabled(settings: Settings, id: HarnessId): boolean {
  return settings[id].enabled;
}
