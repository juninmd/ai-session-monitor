// What a tab chip and the header ring show. Percentages come from summarize() in src/lib/usage.ts.
import type { IconName } from '../components/icons';
import { t } from '../i18n';
import { formatPercent } from '../i18n/format';
import type { HarnessId, HarnessView } from '../lib/contract';
import { isExpired, type Status, type Summary, summarize } from '../lib/usage';
import { sharePct, showsUsed, tq } from './quota';
import type { ViewContext } from './windowModel';

export type TabChip =
  | { kind: 'pct'; status: Status; text: string; pct: number; sr: string }
  | { kind: 'text'; text: string; sr: string; glyph: IconName | null; warn: boolean }
  | { kind: 'loading'; sr: string };

/** Glyph per status: colour is never the only signal. */
export const STATUS_GLYPH: Record<Status, IconName> = {
  ok: 'check',
  warn: 'warn',
  crit: 'alert',
  out: 'ban',
};

export function summaryOf(view: HarnessView, ctx: ViewContext): Summary | null {
  return summarize(view, ctx.nowSecs, ctx.settings.general.warnAtPct);
}

function text(label: string, sr: string, glyph: IconName | null = null, warn = false): TabChip {
  return { kind: 'text', text: label, sr, glyph, warn };
}

function readyChip(view: HarnessView, summary: Summary | null, ctx: ViewContext): TabChip {
  if (view.stale) {
    return text(
      t('tab.stale'),
      t('tab.sr.stale'),
      view.error ? 'warn' : 'clock',
      view.error !== null,
    );
  }
  if (summary) {
    const showUsed = showsUsed(ctx.settings);
    const shown = sharePct(summary.availablePct, showUsed);
    const label = formatPercent(shown);
    return {
      kind: 'pct',
      status: summary.status,
      text: label,
      pct: shown,
      sr: tq('tab.sr.pct', showUsed, { pct: label, status: t(`status.${summary.status}`) }),
    };
  }
  if (view.windows.length === 0) return text(t('tab.no_windows'), t('tab.sr.no_windows'));
  if (view.windows.every((w) => isExpired(w, ctx.nowSecs))) {
    return text(t('tab.expired'), t('tab.sr.expired'), 'refresh');
  }
  return text(t('tab.unknown'), t('tab.sr.unknown'));
}

export function tabChip(view: HarnessView, summary: Summary | null, ctx: ViewContext): TabChip {
  switch (view.state) {
    case 'loading':
      return { kind: 'loading', sr: t('tab.sr.loading') };
    case 'disabled':
      return text(t('tab.disabled'), t('tab.sr.disabled'));
    case 'needs_consent':
    case 'needs_setup':
      return text(t('tab.connect'), t('tab.sr.connect'));
    case 'no_data':
      return text(t('tab.no_data'), t('tab.sr.no_data'));
    case 'error':
      return text(t('tab.error'), t('tab.sr.error'), 'warn', true);
    case 'ready':
      return readyChip(view, summary, ctx);
  }
}

/** The tray ring and the header follow one harness, or the lowest available of all. */
export function traySummary(
  views: readonly HarnessView[],
  ctx: ViewContext,
): { id: HarnessId; summary: Summary } | null {
  const source = ctx.settings.general.traySource;
  let best: { id: HarnessId; summary: Summary } | null = null;
  for (const view of views) {
    if (source !== null && view.id !== source) continue;
    const summary = summaryOf(view, ctx);
    if (summary && (best === null || summary.availablePct < best.summary.availablePct)) {
      best = { id: view.id, summary };
    }
  }
  return best;
}
