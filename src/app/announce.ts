// Decides what the polite live region says when a harness changes status between renders.
import { t } from '../i18n';
import { formatPercent } from '../i18n/format';
import type { HarnessId, HarnessView } from '../lib/contract';
import type { Summary } from '../lib/usage';
import { harnessName } from './harness';
import { sharePct, tq } from './quota';

export type StatusKey = 'ok' | 'warn' | 'crit' | 'out' | 'error' | 'stale' | 'other';

const PROBLEMS: ReadonlySet<StatusKey> = new Set(['warn', 'crit', 'out', 'error', 'stale']);

export function statusKeyOf(view: HarnessView, summary: Summary | null): StatusKey {
  if (view.state === 'error') return 'error';
  if (view.state === 'ready' && view.stale) return 'stale';
  return summary ? summary.status : 'other';
}

/** One sentence per transition worth interrupting for; null for quiet changes. */
export function announcementFor(
  id: HarnessId,
  previous: StatusKey | undefined,
  next: StatusKey,
  summary: Summary | null,
  showUsed = false,
): string | null {
  if (previous === undefined || previous === next) return null;
  const name = harnessName(id);
  const pct = summary ? formatPercent(sharePct(summary.availablePct, showUsed)) : '';
  switch (next) {
    case 'warn':
      return tq('live.warn', showUsed, { name, pct });
    case 'crit':
      return tq('live.crit', showUsed, { name, pct });
    case 'out':
      return t('live.out', { name });
    case 'error':
      return t('live.error', { name });
    case 'stale':
      return t('live.stale', { name });
    case 'ok':
      return PROBLEMS.has(previous) ? tq('live.ok', showUsed, { name, pct }) : null;
    case 'other':
      return null;
  }
}
