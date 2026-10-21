// The stored readings of one window as CSV, for a spreadsheet. Pure: the board copies the result.
import type { HistoryEpoch } from '../lib/contract';

const HEADER = 'harness,window,resets_at,read_at,used_pct';

const iso = (secs: number) => new Date(secs * 1000).toISOString();

export function historyCsv(harness: string, epochs: readonly HistoryEpoch[]): string {
  const rows = epochs.flatMap((epoch) =>
    epoch.points.map((point) =>
      [harness, epoch.windowId, iso(epoch.resetsAt), iso(point.t), point.usedPct].join(','),
    ),
  );
  return [HEADER, ...rows].join('\n');
}
