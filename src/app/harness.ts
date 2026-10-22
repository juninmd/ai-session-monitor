import { t } from '../i18n';
import { type ExternalLink, HARNESS_IDS, type HarnessId } from '../lib/contract';

/** Tab order is the order of `HarnessId::ALL` in Rust. */
export const TAB_ORDER: readonly HarnessId[] = HARNESS_IDS;

export function harnessName(id: HarnessId): string {
  return t(`harness.${id}`);
}

/** Where a settings group lives: a harness, or one of the app's own sections. */
export type SettingsSection = HarnessId | 'general' | 'history' | 'sources' | 'update';

/** True for the ids that name a provider, so a section can be narrowed back to a harness. */
export function isHarnessId(value: string): value is HarnessId {
  return (HARNESS_IDS as readonly string[]).includes(value);
}

/** The vendor page an "open the console" action leads to; OpenCode is the default. */
export function consoleLink(id: HarnessId): ExternalLink {
  switch (id) {
    case 'kimi':
      return 'kimi_console';
    case 'zai':
      return 'zai_console';
    case 'openrouter':
      return 'openrouter_console';
    case 'deepseek':
      return 'deepseek_console';
    case 'deepinfra':
      return 'deepinfra_console';
    case 'moonshot':
      return 'moonshot_console';
    case 'novita':
      return 'novita_console';
    case 'ollama':
      return 'ollama_console';
    case 'minimax':
      return 'minimax_console';
    case 'cursor':
      return 'cursor_dashboard';
    case 'copilot':
      return 'copilot_settings';
    default:
      return 'opencode_go_console';
  }
}
