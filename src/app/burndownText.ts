// The words that go with the burndown, so the chart never carries a fact on its own.
import { t } from '../i18n';
import { formatClock, formatPercent } from '../i18n/format';
import type { Burndown } from './burndown';

/** "Pico de 78% às 14:02 nesta janela." plus one sentence about the earlier passes. */
export function summarySentences(burndown: Burndown, nowSecs: number): string[] {
  const { current, past } = burndown;
  const sentences = [
    current === null
      ? t('history.no_current')
      : t('history.peak_now', {
          pct: formatPercent(current.peakPct),
          when: formatClock(current.peakAt, nowSecs).label,
        }),
  ];
  if (past.length > 0) {
    const peaks = past.map((curve) => formatPercent(curve.peakPct)).join(t('list.sep'));
    sentences.push(
      past.length === 1
        ? t('history.past_one', { peaks })
        : t('history.past_many', { n: past.length, peaks }),
    );
  }
  return sentences;
}

export interface TableRow {
  key: number;
  reset: string;
  peak: string;
  readings: number;
}

/** Newest first, the current pass marked in words. */
export function tableRows(burndown: Burndown, nowSecs: number): TableRow[] {
  const rows = [...burndown.past, ...(burndown.current ? [burndown.current] : [])]
    .map((curve) => {
      const when = formatClock(curve.resetsAt, nowSecs).text;
      const isCurrent = curve === burndown.current;
      return {
        key: curve.resetsAt,
        reset: isCurrent ? t('history.row_current', { when }) : when,
        peak: formatPercent(curve.peakPct),
        readings: curve.readings,
      };
    })
    .reverse();
  return rows;
}
