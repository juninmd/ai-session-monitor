// The trend chart's arithmetic, apart from its drawing: reading a curve at a moment, the band the
// closed passes cover, and where a label fits. Pure, so the picture and its text agree.
import type { Curve } from '../app/model/burndown';
import type { Segment } from './historyModel';

const clamp = (value: number, low: number, high: number) => Math.min(high, Math.max(low, value));

/** The used share a curve reads at `x`, joined in straight lines; null outside what it measured. */
export function valueAt(plot: readonly Segment[], x: number): number | null {
  const first = plot[0];
  const last = plot.at(-1);
  if (first === undefined || last === undefined || x < first.x || x > last.x) return null;
  for (let index = 1; index < plot.length; index += 1) {
    const from = plot[index - 1];
    const to = plot[index];
    if (from === undefined || to === undefined || x > to.x) continue;
    const span = to.x - from.x;
    return span <= 0 ? to.y : from.y + ((x - from.x) / span) * (to.y - from.y);
  }
  return last.y;
}

export interface BandPoint {
  x: number;
  low: number;
  high: number;
}

/**
 * The range the closed passes covered at each moment of the window. Seven thin lines say "it
 * varies"; the band says by how much. Needs two passes to be a range at all.
 */
export function bandOf(past: readonly Curve[], samples = 32): BandPoint[] {
  if (past.length < 2) return [];
  const band: BandPoint[] = [];
  for (let step = 0; step <= samples; step += 1) {
    const x = step / samples;
    const values = past.flatMap((curve) => valueAt(curve.plot, x) ?? []);
    if (values.length < 2) continue;
    band.push({ x, low: Math.min(...values), high: Math.max(...values) });
  }
  return band.length < 2 ? [] : band;
}

/** The stretch of the window the running pass has readings for. */
export function domainOf(plot: readonly Segment[]): { from: number; to: number } | null {
  const first = plot[0];
  const last = plot.at(-1);
  if (first === undefined || last === undefined || last.x <= first.x) return null;
  return { from: first.x, to: last.x };
}

/** A moment the pointer or the scrubber asked for, held inside the measured stretch. */
export const clampMoment = (domain: { from: number; to: number }, x: number): number =>
  clamp(x, domain.from, domain.to);

export interface LabelSpot {
  dx: number;
  dy: number;
  anchor: 'start' | 'end';
}

/**
 * Where the live reading's label goes around its dot: after it while there is room, before it near
 * the trailing edge; under it when a projection climbs away from the dot, so the two never cross.
 */
export function labelSpot(now: Segment, climbing: boolean): LabelSpot {
  const before = now.x > 0.82;
  const under = climbing && now.y > 0.14;
  return {
    dx: before ? -8 : 8,
    dy: under ? 14 : now.y > 0.9 ? 12 : 4,
    anchor: before ? 'end' : 'start',
  };
}
