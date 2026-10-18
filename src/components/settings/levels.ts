// The two levels of the settings screen: the menu, and the one section the menu opened.
import type { SettingsSection } from '../../app/harness';
import type { MessageKey } from '../../i18n';
import type { HarnessId } from '../../lib/contract';

export type SettingsLevel = 'menu' | 'general' | 'history' | 'update' | 'sources';

/** Where a deep link opens: the menu when nothing asked for a section. */
export function levelOf(section: SettingsSection | null): SettingsLevel {
  if (section === 'general') return 'general';
  if (section === 'update') return 'update';
  if (section === null) return 'menu';
  return 'sources';
}

/** The harness whose group a deep link opens, or null when the link is not about one. */
export function sourceOf(section: SettingsSection | null): HarnessId | null {
  if (section === null || section === 'general' || section === 'update') return null;
  return section;
}

/** The heading of each level, and the name of its row in the menu. */
export const LEVEL_TITLE: Record<SettingsLevel, MessageKey> = {
  menu: 'settings.title',
  general: 'settings.monitoring',
  history: 'settings.history',
  update: 'update.title',
  sources: 'settings.sources',
};
