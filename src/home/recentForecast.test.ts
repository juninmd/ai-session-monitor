import { describe, expect, it } from 'vitest';
import { defaultSettings, type HistoryEpoch } from '../lib/contract';
import { windowOf } from '../mock/fixtures';
import { startFor } from '../mock/scenarios';
import { NOW } from '../test/helpers';
import { recentForecast } from './recentForecast';

const window = windowOf(
  { now: NOW, settings: defaultSettings, secrets: startFor('demo').secrets },
  { id: 'five_hour', kind: 'five_hour', usedPct: 90, windowMinutes: 300, resetsInMin: 60 },
);

function epoch(consumption: readonly number[], ageMin = 0): HistoryEpoch {
  return {
    windowId: window.id,
    resetsAt: NOW + 3600,
    points: consumption.map((usedPct, index) => ({
      t: NOW - (ageMin + (consumption.length - 1 - index) * 20) * 60,
      usedPct,
    })),
  };
}

describe('recent quota forecast', () => {
  it('projects the faster and slower observed rates from the same reading', () => {
    expect(recentForecast(window, [epoch([60, 70, 90])], NOW)).toEqual({
      earliestMin: 10,
      latestMin: 20,
    });
  });

  it('reports renewal as a possibility when one interval was flat', () => {
    expect(recentForecast(window, [epoch([70, 70, 90])], NOW)).toEqual({
      earliestMin: 10,
      latestMin: null,
    });
  });

  it('abstains on stale, sparse, contradictory and decreasing series', () => {
    for (const readings of [
      epoch([60, 70, 90], 16),
      epoch([70, 90]),
      epoch([60, 70, 85]),
      epoch([80, 70, 90]),
      epoch([90, 90, 90]),
    ]) {
      expect(recentForecast(window, [readings], NOW)).toBeNull();
    }
  });

  it('ignores the previous cycle and forecasts that miss the reset', () => {
    expect(recentForecast(window, [{ ...epoch([60, 70, 90]), resetsAt: NOW }], NOW)).toBeNull();
    expect(recentForecast(window, [epoch([88, 89, 90])], NOW)).toBeNull();
  });
});
