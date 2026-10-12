import { describe, expect, it } from 'vitest';
import { win } from '../test/helpers';
import { burndownOf } from './burndown';

const RESET = win().resetsAt as number;
const LENGTH = 300 * 60;

const epoch = (resetsAt: number, points: [number, number][], windowId = 'five_hour') => ({
  windowId,
  resetsAt,
  points: points.map(([t, usedPct]) => ({ t, usedPct })),
});

describe('burndownOf', () => {
  it('is null while the window has no stored reading', () => {
    expect(burndownOf(win(), [])).toBeNull();
    expect(burndownOf(win(), [epoch(RESET, [], 'five_hour')])).toBeNull();
    expect(burndownOf(win(), [epoch(RESET, [[RESET - 60, 5]], 'weekly')])).toBeNull();
  });

  it('places a reading by the share of the window that had elapsed', () => {
    const start = RESET - LENGTH;
    const result = burndownOf(win(), [
      epoch(RESET, [
        [start, 0],
        [start + LENGTH / 2, 50],
      ]),
    ]);
    expect(result?.current?.plot).toEqual([
      { x: 0, y: 0 },
      { x: 0.5, y: 0.5 },
    ]);
    expect(result?.past).toEqual([]);
  });

  it('separates the current pass from earlier ones and orders those oldest first', () => {
    const older = epoch(RESET - 2 * LENGTH, [[RESET - 2 * LENGTH - 60, 30]]);
    const earlier = epoch(RESET - LENGTH, [[RESET - LENGTH - 60, 90]]);
    const now = epoch(RESET + 2, [[RESET - 60, 40]]);
    const result = burndownOf(win(), [now, earlier, older]);
    expect(result?.current?.peakPct).toBe(40);
    expect(result?.past.map((curve) => curve.peakPct)).toEqual([30, 90]);
  });

  it('reports the peak and when it happened, and the first of equal peaks', () => {
    const result = burndownOf(win(), [
      epoch(RESET, [
        [RESET - 900, 20],
        [RESET - 600, 78],
        [RESET - 300, 78],
      ]),
    ]);
    expect(result?.current).toMatchObject({ peakPct: 78, peakAt: RESET - 600, readings: 3 });
  });

  it('keeps every point inside the plot even when the window length was underestimated', () => {
    const odd = win({ windowMinutes: 10 });
    const result = burndownOf(odd, [
      epoch(RESET, [
        [RESET - 99_999, 120],
        [RESET + 500, -4],
      ]),
    ]);
    const xs = result?.current?.plot.flatMap((p) => [p.x, p.y]) ?? [];
    expect(Math.min(...xs)).toBeGreaterThanOrEqual(0);
    expect(Math.max(...xs)).toBeLessThanOrEqual(1);
  });

  it('has no current pass for a window without a reset time', () => {
    const open = win({ resetsInMin: null });
    const result = burndownOf(open, [epoch(RESET, [[RESET - 60, 5]])]);
    expect(result?.current).toBeNull();
    expect(result?.past).toHaveLength(1);
  });
});
