import { describe, expect, it } from 'vitest';
import { NOW } from '../test/helpers';
import type { Burndown, Curve } from './burndown';
import { summarySentences, tableRows } from './burndownText';

const curve = (resetsAt: number, peakPct: number, peakAt = NOW - 600): Curve => ({
  resetsAt,
  plot: [],
  peakPct,
  peakAt,
  readings: 4,
});

const burndown = (current: Curve | null, past: Curve[]): Burndown => ({
  lengthSecs: 18_000,
  current,
  past,
});

describe('summarySentences', () => {
  it('quotes the peak and when it happened', () => {
    const [first] = summarySentences(burndown(curve(NOW + 60, 78), []), NOW);
    expect(first).toMatch(/^Pico de 78% às \d{2}:\d{2} nesta janela\.$/);
  });

  it('says there is nothing yet instead of inventing a peak', () => {
    expect(summarySentences(burndown(null, []), NOW)).toEqual(['Ainda sem leituras nesta janela.']);
  });

  it('adds the earlier passes in the singular and the plural', () => {
    const one = summarySentences(burndown(curve(NOW, 10), [curve(NOW - 1, 62)]), NOW);
    expect(one[1]).toBe('Janela anterior: pico de 62%.');
    const many = summarySentences(burndown(null, [curve(NOW - 2, 62), curve(NOW - 1, 91)]), NOW);
    expect(many[1]).toBe('2 janelas anteriores: picos de 62%, 91%.');
  });
});

describe('tableRows', () => {
  it('lists the newest pass first and marks the current one in words', () => {
    const rows = tableRows(burndown(curve(NOW + 60, 40), [curve(NOW - 100, 30)]), NOW);
    expect(rows.map((row) => row.peak)).toEqual(['40%', '30%']);
    expect(rows[0]?.reset).toMatch(/\(atual\)$/);
    expect(rows[1]?.reset).not.toMatch(/atual/);
  });
});
