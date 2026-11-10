// The triage's state filter: every urgency tier falls in one of four buckets a person can ask for
// ("what do I have to act on?"). Pure; the strip and the grid read the same counts.
import type { Tier, TriageRow } from './triageModel';

export type HealthBucket = 'act' | 'watch' | 'fine' | 'idle';
export type TriageFilter = 'all' | HealthBucket;

export const HEALTH_BUCKETS: readonly HealthBucket[] = ['act', 'watch', 'fine', 'idle'];

const BUCKET_OF: Record<Tier, HealthBucket> = {
  blocked: 'act',
  risk: 'act',
  broken: 'act',
  watch: 'watch',
  fine: 'fine',
  loading: 'idle',
  unknown: 'idle',
};

export const bucketOf = (tier: Tier): HealthBucket => BUCKET_OF[tier];

export interface HealthShare {
  bucket: HealthBucket;
  count: number;
}

/** How many providers sit in each bucket, in the order a person acts; empty buckets are left out. */
export function healthOf(rows: readonly TriageRow[]): HealthShare[] {
  const counts: Record<HealthBucket, number> = { act: 0, watch: 0, fine: 0, idle: 0 };
  for (const row of rows) counts[bucketOf(row.tier)] += 1;
  return HEALTH_BUCKETS.map((bucket) => ({ bucket, count: counts[bucket] })).filter(
    (share) => share.count > 0,
  );
}

/**
 * The filter that can actually be applied: a bucket that emptied out (the provider recovered while
 * it was selected) falls back to everything, so the grid never shows nothing with no way out.
 */
export function effectiveFilter(
  filter: TriageFilter,
  health: readonly HealthShare[],
): TriageFilter {
  return filter !== 'all' && health.some((share) => share.bucket === filter) ? filter : 'all';
}

export function filterRows(rows: readonly TriageRow[], filter: TriageFilter): readonly TriageRow[] {
  return filter === 'all' ? rows : rows.filter((row) => bucketOf(row.tier) === filter);
}

/** The strip earns its row only when there is something to tell apart. */
export const showsHealth = (health: readonly HealthShare[]): boolean => health.length > 1;
