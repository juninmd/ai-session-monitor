// The panes of the settings screen: the menu opens one, and every source group lands on the same
// one. The screen's own heading is the menu's; a pane prints its title in the drawer bar.
import { isHarnessId, type SettingsSection } from '../../app/harness';
import type { MessageKey } from '../../i18n';
import type { HarnessId } from '../../lib/contract';

/** One pane of the settings screen: what a menu row opens, and what a drawer prints. */
export type SettingsPane = 'general' | 'history' | 'sources' | 'update';

/** The pane a section opens: a harness id is a group inside Fontes de dados. */
export function paneOf(section: SettingsSection): SettingsPane {
  if (section === 'general' || section === 'history' || section === 'update') return section;
  return 'sources';
}

/** The harness whose group a deep link opens, or null when the link is not about one. */
export function sourceOf(section: SettingsSection | null): HarnessId | null {
  return section !== null && isHarnessId(section) ? section : null;
}

/** The heading of each pane, and the name of its row in the menu. */
export const LEVEL_TITLE: Record<SettingsPane, MessageKey> = {
  general: 'settings.monitoring',
  history: 'settings.history',
  update: 'update.title',
  sources: 'settings.sources',
};
