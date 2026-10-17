// The quota sheet and the trend sentences. Pure, so the wording and the grouping are pinned
// without rendering anything.
import { describe, expect, it } from 'vitest';
import type { ViewContext } from '../../app/windowModel';
import { defaultSettings, type HarnessView } from '../../lib/contract';
import { viewOf, type WindowSpec, windowOf } from '../../mock/fixtures';
import { startFor } from '../../mock/scenarios';
import { NOW } from '../../test/helpers';
import { hasReading, readableWindows, sheetOf } from './quotas';
import { quotaName } from './wording';

const settings = structuredClone(defaultSettings);
const ctx: ViewContext = { nowSecs: NOW, settings };
const env = { now: NOW, settings, secrets: startFor('demo').secrets };

const window_ = (spec: Partial<WindowSpec> & { id: string }) =>
  windowOf(env, {
    kind: 'weekly',
    usedPct: 40,
    windowMinutes: 10080,
    resetsInMin: 4000,
    ...spec,
  });

describe('the quota sheet', () => {
  it('keeps every window, grouped, with the pool named', () => {
    const view = viewOf('antigravity', {
      windows: [
        window_({ id: 'gemini:five_hour', kind: 'five_hour', group: 'gemini' }),
        window_({ id: 'gemini:weekly', group: 'gemini' }),
        window_({ id: 'claude_gpt:five_hour', kind: 'five_hour', group: 'claude_gpt' }),
        window_({ id: 'plain:weekly' }),
      ],
    });
    const sheet = sheetOf(view, ctx);
    expect(sheet.total).toBe(4);
    // Pools come first: a grouped quota only means something next to its pool.
    expect(sheet.groups.map((group) => group.title)).toEqual(['Gemini', 'Claude e GPT-OSS', null]);
    expect(sheet.groups[0]?.windows.map((model) => model.window.id)).toEqual([
      'gemini:five_hour',
      'gemini:weekly',
    ]);
    expect(sheet.groups[2]?.windows[0]?.window.id).toBe('plain:weekly');
  });

  it('names a pooled quota the way a card must, instead of leaving it bare', () => {
    expect(quotaName(window_({ id: 'a', group: 'gemini' }))).toBe('Gemini · Semana');
    expect(quotaName(window_({ id: 'b' }))).toBe('Semana');
  });

  it('counts what a chart could use and never invents a reading', () => {
    const view = viewOf('codex', {
      windows: [
        window_({ id: 'weekly' }),
        window_({ id: 'unknown', usedPct: null }),
        window_({ id: 'no_reset', resetsInMin: null }),
      ],
    });
    const sheet = sheetOf(view, ctx);
    expect(sheet.total).toBe(3);
    expect(sheet.readable).toBe(1);
    expect(readableWindows(view).map((window) => window.id)).toEqual(['weekly']);
    expect(hasReading(window_({ id: 'x' }))).toBe(true);
    expect(hasReading(window_({ id: 'x', usedPct: null }))).toBe(false);
    expect(hasReading(window_({ id: 'x', resetsInMin: null }))).toBe(false);
  });

  it('reports an empty sheet when the source has no windows at all', () => {
    const sheet = sheetOf(viewOf('deepseek'), ctx);
    expect(sheet.total).toBe(0);
    expect(sheet.groups).toEqual([]);
  });

  it('names a numbered group the way the layout does, instead of leaving it raw', () => {
    const view: HarnessView = viewOf('antigravity', {
      windows: [window_({ id: 'a', group: 'group_0' })],
    });
    expect(sheetOf(view, ctx).groups[0]?.title).toBe('Grupo 1');
  });
});
