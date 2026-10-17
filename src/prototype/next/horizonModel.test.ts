// The horizon answers one question per provider: does the remaining quota cover the time until the
// renewal? These tests pin the shape of that answer, including what it refuses to project.
import { describe, expect, it } from 'vitest';
import type { ViewContext } from '../../app/windowModel';
import { defaultSettings, type HarnessId, type HarnessView } from '../../lib/contract';
import { asStale, viewOf, type WindowSpec, windowOf } from '../../mock/fixtures';
import { startFor } from '../../mock/scenarios';
import { NOW } from '../../test/helpers';
import { exhaustionLabel, type HorizonRow, horizonOf } from './horizonModel';

const settings = structuredClone(defaultSettings);
const ctx: ViewContext = { nowSecs: NOW, settings };
const env = { now: NOW, settings, secrets: startFor('demo').secrets };

function reading(id: HarnessId, spec: Partial<WindowSpec> = {}): HarnessView {
  return viewOf(id, {
    windows: [
      windowOf(env, {
        id: 'five_hour',
        kind: 'five_hour',
        usedPct: 48,
        windowMinutes: 300,
        resetsInMin: 60,
        ...spec,
      }),
    ],
  });
}

function rowFor(views: readonly HarnessView[], id: string): HorizonRow {
  const row = horizonOf(views, ctx).rows.find((item) => item.view.id === id);
  if (row === undefined) throw new Error(`no horizon row for ${id}`);
  return row;
}

const lasts = reading('claude', { usedPct: 48 });
const short = reading('codex', { usedPct: 90 });
const gone = reading('zai', { usedPct: 100 });
const far = reading('kimi', { usedPct: 10, resetsInMin: 240 });

describe('horizon rows', () => {
  it('measures the covered share of the time left and the stretch with no quota', () => {
    const rows = horizonOf([lasts, short, gone, far], ctx).rows;
    const byId = new Map(rows.map((row) => [row.view.id, row]));
    expect(byId.get('claude')?.kind).toBe('lasts');
    expect(byId.get('claude')?.cover).toBe(1);
    expect(byId.get('claude')?.darkMin).toBe(0);
    expect(byId.get('codex')?.kind).toBe('short');
    expect(byId.get('codex')?.cover).toBeCloseTo(26.667 / 60, 3);
    expect(byId.get('codex')?.darkMin).toBeCloseTo(33.333, 3);
    expect(byId.get('zai')?.kind).toBe('out');
    expect(byId.get('zai')?.cover).toBe(0);
    expect(byId.get('zai')?.darkMin).toBe(60);
  });

  it('sorts the long uncovered stretches first, the worst one leading', () => {
    const rows = horizonOf([lasts, far, short, gone], ctx).rows;
    expect(rows.map((row) => row.view.id)).toEqual(['zai', 'codex', 'claude', 'kimi']);
  });

  it('keeps the status the rest of the interface prints, so it cannot contradict the triage', () => {
    // 8 % left with 500 of 10 080 minutes to go: statusOf says crit, paceOf says it covers.
    const critButCovered = reading('codex', {
      id: 'weekly',
      kind: 'weekly',
      usedPct: 92,
      windowMinutes: 10080,
      resetsInMin: 500,
    });
    const row = rowFor([critButCovered], 'codex');
    expect(row.kind).toBe('lasts');
    expect(row.model?.status).toBe('crit');
  });

  it('reports the window that renews first, whatever its shortfall', () => {
    expect(horizonOf([lasts, far], ctx).soonest?.view.id).toBe('claude');
    expect(horizonOf([short], ctx).soonest?.view.id).toBe('codex');
  });

  it('never projects from a stale, missing or barely started reading', () => {
    const rows = horizonOf(
      [
        asStale(reading('kimi'), env, 12),
        reading('deepseek', { usedPct: null }),
        reading('opencode', { usedPct: 2, resetsInMin: 295 }),
      ],
      ctx,
    ).rows;
    expect(rows.map((row) => row.kind)).toEqual(['broken', 'unknown', 'unknown']);
    expect(rows.map((row) => row.cover)).toEqual([null, null, null]);
    expect(rows.map((row) => row.label)).toEqual([
      'Leitura antiga',
      'Sem leitura',
      'Cedo para prever',
    ]);
  });

  it('says it is reading instead of calling a source unreadable', () => {
    const row = rowFor([viewOf('codex', { state: 'loading' })], 'codex');
    expect(row.kind).toBe('unknown');
    expect(row.label).toBe('Lendo…');
  });

  it('names the pool a grouped quota belongs to, so the row is not just "5 horas"', () => {
    const pooled = viewOf('antigravity', {
      windows: [
        windowOf(env, {
          id: 'gemini:five_hour',
          kind: 'five_hour',
          group: 'gemini',
          usedPct: 20,
          windowMinutes: 300,
          resetsInMin: 60,
        }),
        windowOf(env, {
          id: 'claude_gpt:five_hour',
          kind: 'five_hour',
          group: 'claude_gpt',
          usedPct: 90,
          windowMinutes: 300,
          resetsInMin: 60,
        }),
      ],
    });
    expect(rowFor([pooled], 'antigravity').windowName).toBe('Claude e GPT-OSS · 5 horas');
    expect(rowFor([pooled], 'antigravity').kind).toBe('short');
  });

  it('prints the moment the quota runs out only when there is a shortfall', () => {
    expect(exhaustionLabel(rowFor([lasts], 'claude'), ctx)).toBeNull();
    expect(exhaustionLabel(rowFor([short], 'codex'), ctx)).toContain('esgota');
    expect(exhaustionLabel(rowFor([gone], 'zai'), ctx)).toBe('já esgotado');
  });
});
