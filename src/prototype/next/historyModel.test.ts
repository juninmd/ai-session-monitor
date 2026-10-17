// The comparison sentence and the projection the chart draws. The projection must never disagree
// with paceOf: it exists to draw what that rule already decided, anchored at the live reading.
import { describe, expect, it } from 'vitest';
import { type Burndown, burndownOf } from '../../app/burndown';
import { modelWindow, type ViewContext } from '../../app/windowModel';
import { defaultSettings, type HistoryEpoch } from '../../lib/contract';
import { windowOf } from '../../mock/fixtures';
import { startFor } from '../../mock/scenarios';
import { NOW } from '../../test/helpers';
import { nowPoint, projectionLabel, projectionOf, trendOf } from './historyModel';

const settings = structuredClone(defaultSettings);
const ctx: ViewContext = { nowSecs: NOW, settings };
const env = { now: NOW, settings, secrets: startFor('demo').secrets };

const WINDOW_MIN = 300;

/** A pass that already closed, `age` windows back, peaking at `peak`. */
function pastEpoch(age: number, peak: number, resetsInMin = 113): HistoryEpoch {
  const end = NOW + resetsInMin * 60 - age * WINDOW_MIN * 60;
  return {
    windowId: 'five_hour',
    resetsAt: end,
    points: [
      { t: end - WINDOW_MIN * 60, usedPct: 0 },
      { t: end - (WINDOW_MIN * 60) / 2, usedPct: peak / 2 },
      { t: end, usedPct: peak },
    ],
  };
}

/** The pass still running: its readings start at the reset that opened it and stop at now. */
function currentEpoch(peak: number, resetsInMin = 113): HistoryEpoch {
  const end = NOW + resetsInMin * 60;
  const start = end - WINDOW_MIN * 60;
  const elapsed = NOW - start;
  return {
    windowId: 'five_hour',
    resetsAt: end,
    points: [0, 0.5, 1].map((share) => ({
      t: Math.round(start + elapsed * share),
      usedPct: peak * share,
    })),
  };
}

const fiveHour = (usedPct: number | null, resetsInMin = 113) =>
  windowOf(env, {
    id: 'five_hour',
    kind: 'five_hour',
    usedPct,
    windowMinutes: WINDOW_MIN,
    resetsInMin,
  });

function burndownOfPeaks(nowPeak: number, pastPeaks: number[]): Burndown {
  const epochs = [
    ...pastPeaks.map((peak, index) => pastEpoch(pastPeaks.length - index, peak)),
    currentEpoch(nowPeak),
  ];
  const burndown = burndownOf(fiveHour(nowPeak), epochs);
  if (burndown === null) throw new Error('no burndown');
  return burndown;
}

describe('the comparison with the earlier passes', () => {
  it('states the two measured facts instead of judging a pass still running', () => {
    const trend = trendOf(burndownOfPeaks(90, [40, 50]));
    expect(trend.peakNow).toBeCloseTo(90, 0);
    expect(trend.pastAverage).toBeCloseTo(45, 0);
    expect(trend.pastCount).toBe(2);
    expect(trend.label).toBe(
      'Pico de 90% até agora · as 2 renovações anteriores fecharam em média em 45%.',
    );
  });

  it('carries the direction as data without colouring it as a status', () => {
    expect(trendOf(burndownOfPeaks(90, [40, 50])).kind).toBe('more');
    expect(trendOf(burndownOfPeaks(20, [80, 90])).kind).toBe('less');
    expect(trendOf(burndownOfPeaks(50, [48, 52])).kind).toBe('similar');
  });

  it('writes the singular correctly for one earlier pass', () => {
    expect(trendOf(burndownOfPeaks(90, [40])).label).toContain('a renovação anterior fechou');
  });

  it('says so when there is nothing to compare with', () => {
    const trend = trendOf(burndownOfPeaks(50, []));
    expect(trend.kind).toBe('first');
    expect(trend.pastAverage).toBeNull();
    expect(trend.label).toContain('sem renovação anterior para comparar');
  });
});

describe('the projection the chart draws', () => {
  /** 90 % used with 60 of 300 minutes to go: paceOf already says it runs out first. */
  function atRisk() {
    const window = fiveHour(90, 60);
    const model = modelWindow('codex', window, ctx);
    const burndown = burndownOf(window, [currentEpoch(90, 60)]);
    if (burndown === null) throw new Error('no burndown');
    const now = nowPoint(model);
    if (now === null) throw new Error('no now point');
    return { model, burndown, now };
  }

  it('anchors at the live reading, not at the last stored one', () => {
    const { now } = atRisk();
    expect(now.x).toBeCloseTo((WINDOW_MIN - 60) / WINDOW_MIN, 5);
    expect(now.y).toBeCloseTo(0.9, 5);
  });

  it('draws a line to 100 % for a quota paceOf says will run out', () => {
    const { model, now } = atRisk();
    if (model.pace.kind !== 'risk') throw new Error('expected a risky window');
    const projection = projectionOf(model, now);
    expect(projection).not.toBeNull();
    expect(projection?.to.y).toBe(1);
    expect(projection?.to.x).toBeCloseTo(now.x + model.pace.runwayMin / WINDOW_MIN, 5);
    expect(projection?.to.x).toBeLessThan(1);
    expect(projection?.minutes).toBeCloseTo(model.pace.runwayMin, 5);
  });

  it('draws nothing for a quota that reaches its renewal', () => {
    const window = fiveHour(20, 60);
    const model = modelWindow('codex', window, ctx);
    const now = nowPoint(model);
    if (now === null) throw new Error('no now point');
    expect(model.pace.kind).toBe('fine');
    expect(projectionOf(model, now)).toBeNull();
  });

  it('draws nothing once the projection would land past the renewal', () => {
    const { model, now } = atRisk();
    const runaway = { ...model, pace: { kind: 'risk' as const, runwayMin: 10_000, earlyMin: 0 } };
    expect(projectionOf(runaway, now)).toBeNull();
  });

  it('draws nothing without a length to project over', () => {
    const { model, now } = atRisk();
    const broken = { ...model, window: { ...model.window, windowMinutes: 0 } };
    expect(projectionOf(broken, now)).toBeNull();
  });

  it('has no anchor at all for a window with no reading or no length', () => {
    expect(nowPoint(modelWindow('codex', fiveHour(null), ctx))).toBeNull();
    const timeless = windowOf(env, {
      id: 'x',
      kind: 'other',
      usedPct: 40,
      windowMinutes: null,
      resetsInMin: 60,
    });
    expect(nowPoint(modelWindow('codex', timeless, ctx))).toBeNull();
  });

  it('prints the runway it drew', () => {
    const { model } = atRisk();
    expect(projectionLabel(72, model, ctx)).toBe('1h 12m');
  });
});
