import { describe, expect, it } from 'vitest';
import { defaultSettings } from '../lib/contract';
import { normalizeSettings, normalizeThresholds } from './settingsRules';

describe('normalizeThresholds (mirrors Settings::normalize)', () => {
  it.each([
    [
      [95, 80, 80],
      [80, 95],
    ],
    [[0, 100, 50, 1.5], [50]],
    [[], []],
    [
      [99, 1],
      [1, 99],
    ],
    [
      [90, 10, 70, 30, 50, 20],
      [10, 20, 30, 50],
    ],
  ])('%j becomes %j', (given, expected) => {
    expect(normalizeThresholds(given)).toEqual(expected);
  });

  it('is applied when settings are saved', () => {
    const draft = structuredClone(defaultSettings);
    draft.general.alertThresholds = [96, 0, 40, 40];
    expect(normalizeSettings(draft).general.alertThresholds).toEqual([40, 96]);
  });
});
