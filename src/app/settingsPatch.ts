import type { HarnessId, Settings } from '../lib/contract';

/** Typed per-harness patches: a computed key would widen the result and lose the contract. */
export function withEnabled(settings: Settings, id: HarnessId, enabled: boolean): Settings {
  switch (id) {
    case 'claude':
      return { ...settings, claude: { ...settings.claude, enabled } };
    case 'codex':
      return { ...settings, codex: { ...settings.codex, enabled } };
    case 'cursor':
      return { ...settings, cursor: { ...settings.cursor, enabled } };
    case 'copilot':
      return { ...settings, copilot: { ...settings.copilot, enabled } };
    case 'antigravity':
      return { ...settings, antigravity: { ...settings.antigravity, enabled } };
    case 'opencode':
      return { ...settings, opencode: { ...settings.opencode, enabled } };
    case 'kimi':
      return { ...settings, kimi: { ...settings.kimi, enabled } };
    case 'zai':
      return { ...settings, zai: { ...settings.zai, enabled } };
    case 'openrouter':
      return { ...settings, openrouter: { ...settings.openrouter, enabled } };
    case 'deepseek':
      return { ...settings, deepseek: { ...settings.deepseek, enabled } };
    case 'deepinfra':
      return { ...settings, deepinfra: { ...settings.deepinfra, enabled } };
    case 'moonshot':
      return { ...settings, moonshot: { ...settings.moonshot, enabled } };
    case 'novita':
      return { ...settings, novita: { ...settings.novita, enabled } };
    case 'ollama':
      return { ...settings, ollama: { ...settings.ollama, enabled } };
    case 'minimax':
      return { ...settings, minimax: { ...settings.minimax, enabled } };
    case 'litellm':
      return { ...settings, litellm: { ...settings.litellm, enabled } };
  }
}

export function isEnabled(settings: Settings, id: HarnessId): boolean {
  return settings[id].enabled;
}
