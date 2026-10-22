// The Ctrl+K list, built from what the home monitors: one jump per provider, the fleet actions,
// the settings entries and — in the browser dev loop only — the fabricated scenarios.
import { useMemo } from 'react';
import type { ViewContext } from '../app/windowModel';
import { useLocale } from '../i18n/locale';
import { isTauriRuntime } from '../lib/backend';
import type { HarnessId, HarnessView } from '../lib/contract';
import { commandsFor, type PaletteCommand } from './paletteModel';

export interface CommandSources {
  views: readonly HarnessView[];
  ctx: ViewContext;
  select: (id: HarnessId) => void;
  refresh: (id?: HarnessId) => void;
  openSettings: (section: HarnessId | 'general' | null) => void;
  copySummary: (text: string) => void;
}

export function useCommands(input: CommandSources): PaletteCommand[] {
  const { views, ctx, select, refresh, openSettings, copySummary } = input;
  const browser = isTauriRuntime() === false;
  // The labels are built in the active language, so a language change has to rebuild them.
  const locale = useLocale();
  // biome-ignore lint/correctness/useExhaustiveDependencies: `locale` only re-runs the labels.
  return useMemo(
    () =>
      commandsFor({
        views,
        ctx,
        select,
        refresh,
        openSettings,
        copySummary,
        // Switching the fabricated scenario only means something outside the packaged app.
        ...(browser
          ? {
              scenario: (name: string) => {
                const url = new URL(window.location.href);
                url.searchParams.set('mock', name);
                window.location.assign(url);
              },
            }
          : {}),
      }),
    [views, ctx, select, refresh, openSettings, copySummary, browser, locale],
  );
}
