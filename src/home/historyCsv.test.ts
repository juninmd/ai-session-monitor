import { describe, expect, it } from 'vitest';
import { historyCsv } from './historyCsv';

describe('historyCsv', () => {
  it('writes a header and one row per stored reading, oldest epoch first', () => {
    const csv = historyCsv('codex', [
      {
        windowId: 'weekly',
        resetsAt: 1_700_086_400,
        points: [
          { t: 1_700_000_000, usedPct: 12.5 },
          { t: 1_700_003_600, usedPct: 20 },
        ],
      },
    ]);
    expect(csv.split('\n')).toEqual([
      'harness,window,resets_at,read_at,used_pct',
      'codex,weekly,2023-11-15T22:13:20.000Z,2023-11-14T22:13:20.000Z,12.5',
      'codex,weekly,2023-11-15T22:13:20.000Z,2023-11-14T23:13:20.000Z,20',
    ]);
  });

  it('is only the header when nothing is stored', () => {
    expect(historyCsv('codex', [])).toBe('harness,window,resets_at,read_at,used_pct');
  });
});
