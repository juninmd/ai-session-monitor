import { describe, expect, it } from 'vitest';
import type { Curve } from '../app/model/burndown';
import { bandOf, clampMoment, domainOf, labelSpot, valueAt } from './trendChartModel';

const curve = (plot: [number, number][]): Curve => ({
  resetsAt: 0,
  plot: plot.map(([x, y]) => ({ x, y })),
  peakPct: 0,
  peakAt: 0,
  readings: plot.length,
});

describe('valueAt', () => {
  const plot = curve([
    [0.1, 0.1],
    [0.3, 0.5],
    [0.5, 0.5],
  ]).plot;

  it('joins two readings in a straight line', () => {
    expect(valueAt(plot, 0.2)).toBeCloseTo(0.3);
    expect(valueAt(plot, 0.4)).toBeCloseTo(0.5);
  });

  it('reads the ends exactly and nothing outside what was measured', () => {
    expect(valueAt(plot, 0.1)).toBeCloseTo(0.1);
    expect(valueAt(plot, 0.5)).toBeCloseTo(0.5);
    expect(valueAt(plot, 0.05)).toBeNull();
    expect(valueAt(plot, 0.6)).toBeNull();
    expect(valueAt([], 0.2)).toBeNull();
  });

  it('survives two readings at the same moment', () => {
    const doubled = curve([
      [0.2, 0.1],
      [0.2, 0.4],
      [0.4, 0.6],
    ]).plot;
    expect(valueAt(doubled, 0.2)).toBeCloseTo(0.4);
  });
});

describe('bandOf', () => {
  it('needs two closed passes to be a range', () => {
    expect(bandOf([])).toEqual([]);
    expect(
      bandOf([
        curve([
          [0, 0],
          [1, 1],
        ]),
      ]),
    ).toEqual([]);
  });

  it('spans the lowest and the highest pass at each moment both measured', () => {
    const band = bandOf(
      [
        curve([
          [0, 0],
          [1, 1],
        ]),
        curve([
          [0, 0],
          [0.5, 0.2],
        ]),
      ],
      4,
    );
    expect(band.map((point) => point.x)).toEqual([0, 0.25, 0.5]);
    expect(band[2]?.low).toBeCloseTo(0.2);
    expect(band[2]?.high).toBeCloseTo(0.5);
  });
});

describe('the scrubber', () => {
  const domain = { from: 0.2, to: 0.6 };

  it('holds a requested moment inside the measured stretch', () => {
    expect(clampMoment(domain, 0)).toBeCloseTo(0.2);
    expect(clampMoment(domain, 0.4)).toBeCloseTo(0.4);
    expect(clampMoment(domain, 2.5)).toBeCloseTo(0.6);
  });

  it('has no stretch to scrub with a single reading', () => {
    expect(domainOf([{ x: 0.3, y: 0.1 }])).toBeNull();
    expect(domainOf([])).toBeNull();
    expect(
      domainOf([
        { x: 0.1, y: 0 },
        { x: 0.4, y: 0.2 },
      ]),
    ).toEqual({ from: 0.1, to: 0.4 });
  });
});

describe('labelSpot', () => {
  it('sits after the dot, and before it near the trailing edge', () => {
    expect(labelSpot({ x: 0.4, y: 0.5 }, false)).toEqual({ dx: 8, dy: 4, anchor: 'start' });
    expect(labelSpot({ x: 0.9, y: 0.5 }, false)).toEqual({ dx: -8, dy: 4, anchor: 'end' });
  });

  it('drops under the dot when a projection climbs away from it', () => {
    expect(labelSpot({ x: 0.4, y: 0.5 }, true).dy).toBe(14);
    // Too close to the axis to fit underneath: it stays beside the dot.
    expect(labelSpot({ x: 0.4, y: 0.05 }, true).dy).toBe(4);
  });
});
