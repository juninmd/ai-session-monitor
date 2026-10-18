// The one-line state each row of the settings menu shows, read from the stored settings.
import { TAB_ORDER } from '../../app/harness';
import { intervalLabel, percentLabel } from '../../app/options';
import { t } from '../../i18n';
import type { HarnessId, Settings, UpdateStatus } from '../../lib/contract';

/**
 * Sources with no vendor-documented interface. The chip in the group and the count in the menu row
 * read this one list, so a source is never badged in one place and counted in another.
 */
export const UNOFFICIAL: ReadonlySet<HarnessId> = new Set(['antigravity', 'kimi', 'zai']);

/** Mirrors RETENTION_SECS in crates/app-core/src/history/ledger.rs. */
export const HISTORY_RETENTION_DAYS = 35;

const SEP = ' · ';

/** "1 min · adaptativo · avisa em 25%". */
export function monitoringSummary(settings: Settings): string {
  const { general } = settings;
  const parts = [intervalLabel(general.intervalSecs)];
  if (general.adaptive) parts.push(t('settings.adaptive_summary'));
  parts.push(t('settings.warn_summary', { pct: percentLabel(general.warnAtPct) }));
  return parts.join(SEP);
}

/** "Ligado · 35 dias", or just "Desligado" when nothing is kept. */
export function historySummary(settings: Settings): string {
  if (!settings.general.historyEnabled) return t('settings.off');
  return [t('settings.on'), t('settings.history_summary', { days: HISTORY_RETENTION_DAYS })].join(
    SEP,
  );
}

/** "Automáticas · v0.1.0"; the version is left out until the updater has reported one. */
export function updateSummary(auto: boolean, status: UpdateStatus | null): string {
  const parts = [t(auto ? 'settings.update_auto' : 'settings.update_manual')];
  if (status !== null) parts.push(`v${status.currentVersion}`);
  return parts.join(SEP);
}

/** "14 fontes · 3 não oficiais". */
export function sourcesSummary(): string {
  return t('settings.sources_summary', {
    n: TAB_ORDER.length,
    unofficial: UNOFFICIAL.size,
  });
}
