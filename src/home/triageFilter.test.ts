// The state filter's rules: which bucket a tier falls in, what the strip counts, and that a filter
// can never leave the grid empty with no way back.
import { describe, expect, it } from 'vitest';
import { bucketOf, effectiveFilter, filterRows, healthOf, showsHealth } from './triageFilter';
import type { Tier, TriageRow } from './triageModel';

const row = (tier: Tier): TriageRow => ({ tier }) as TriageRow;
const rows = (...tiers: Tier[]) => tiers.map(row);

describe('bucketOf', () => {
  it('asks for action on what ran out, is running out or failed', () => {
    expect((['blocked', 'risk', 'broken'] as const).map(bucketOf)).toEqual(['act', 'act', 'act']);
  });

  it('keeps a threshold reading apart from a calm one, and no reading apart from both', () => {
    expect(bucketOf('watch')).toBe('watch');
    expect(bucketOf('fine')).toBe('fine');
    expect((['loading', 'unknown'] as const).map(bucketOf)).toEqual(['idle', 'idle']);
  });
});

describe('healthOf', () => {
  it('counts each bucket in the order a person acts and leaves the empty ones out', () => {
    expect(healthOf(rows('fine', 'risk', 'fine', 'broken', 'unknown'))).toEqual([
      { bucket: 'act', count: 2 },
      { bucket: 'fine', count: 2 },
      { bucket: 'idle', count: 1 },
    ]);
    expect(healthOf([])).toEqual([]);
  });

  it('is worth a row only when there are two states to tell apart', () => {
    expect(showsHealth(healthOf(rows('fine', 'fine')))).toBe(false);
    expect(showsHealth(healthOf(rows('fine', 'watch')))).toBe(true);
  });
});

describe('the filter', () => {
  const list = rows('risk', 'watch', 'fine', 'fine');

  it('keeps the rows of one bucket, in their order', () => {
    expect(filterRows(list, 'fine')).toEqual([list[2], list[3]]);
    expect(filterRows(list, 'all')).toBe(list);
  });

  it('falls back to everything when its bucket emptied out', () => {
    const health = healthOf(rows('fine', 'watch'));
    expect(effectiveFilter('watch', health)).toBe('watch');
    expect(effectiveFilter('act', health)).toBe('all');
    expect(effectiveFilter('all', health)).toBe('all');
  });
});
