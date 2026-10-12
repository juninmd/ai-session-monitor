import { describe, expect, it } from 'vitest';
import { LIMITS } from '../lib/contract';
import {
  INTERVAL_PRESETS,
  intervalLabel,
  LITELLM_POLL_PRESETS,
  numericOptions,
  percentLabel,
  secondsLabel,
  TIMEOUT_PRESETS,
  WARN_PRESETS,
} from './options';

const values = (options: { value: number }[]) => options.map((o) => o.value);

describe('numericOptions', () => {
  it('keeps presets inside the range, both ends and the current value, sorted', () => {
    const options = numericOptions([10, 20, 90, 200], { min: 15, max: 100 }, 60, String);
    expect(values(options)).toEqual([15, 20, 60, 90, 100]);
    expect(options.map((o) => o.label)).toEqual(['15', '20', '60', '90', '100']);
  });

  it('never hides the saved value, even when it is not a preset', () => {
    expect(values(numericOptions([30, 60], { min: 30, max: 3600 }, 45, String))).toContain(45);
  });

  it('does not repeat a value', () => {
    const options = numericOptions([30, 60, 60], { min: 30, max: 60 }, 60, String);
    expect(values(options)).toEqual([30, 60]);
  });

  it.each([
    ['interval', INTERVAL_PRESETS, LIMITS.intervalSecs],
    ['litellm poll', LITELLM_POLL_PRESETS, LIMITS.litellmPollSecs],
    ['timeout', TIMEOUT_PRESETS, LIMITS.litellmTimeoutSecs],
    ['warn', WARN_PRESETS, LIMITS.warnAtPct],
  ])(
    'offers the whole %s range the backend accepts and nothing it would clamp away',
    (_n, presets, limits) => {
      const options = values(numericOptions(presets, limits, limits.min, String));
      expect(options[0]).toBe(limits.min);
      expect(options.at(-1)).toBe(limits.max);
      expect(options.every((v) => v >= limits.min && v <= limits.max)).toBe(true);
    },
  );
});

describe('labels', () => {
  it('words intervals, seconds and percentages', () => {
    expect(intervalLabel(300)).toBe('5 min');
    expect(secondsLabel(10)).toBe('10 s');
    expect(percentLabel(25)).toBe('25%');
  });
});
