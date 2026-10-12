import type { HarnessId, Settings } from '../lib/contract';
import { TAB_ORDER } from './harness';

/** Patches apply against the latest stored selection, preserving other settings. */
export function withVisibleTab(settings: Settings, id: HarnessId, visible: boolean): Settings {
  const visibleTabs = TAB_ORDER.filter((candidate) =>
    candidate === id ? visible : settings.general.visibleTabs.includes(candidate),
  );
  return withVisibleTabs(settings, visibleTabs);
}

export function withVisibleTabs(settings: Settings, visibleTabs: readonly HarnessId[]): Settings {
  return { ...settings, general: { ...settings.general, visibleTabs: [...visibleTabs] } };
}
