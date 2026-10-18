// The triage rules are the whole point of the proposed home: which provider a person should look
// at first, and what the headline says. Pure functions, so they are tested without a DOM.
import { describe, expect, it } from 'vitest';
import { describeError } from '../app/errorText';
import type { ViewContext } from '../app/windowModel';
import { defaultSettings, type HarnessId, type HarnessView } from '../lib/contract';
import { asStale, problem, viewOf, type WindowSpec, windowOf } from '../mock/fixtures';
import { startFor } from '../mock/scenarios';
import { NOW } from '../test/helpers';
import { headlineOf, triageOf, triageRow } from './triageModel';
import { quotaName } from './wording';

const settings = structuredClone(defaultSettings);
const ctx: ViewContext = { nowSecs: NOW, settings };
const env = { now: NOW, settings, secrets: startFor('demo').secrets };

function reading(
  id: HarnessId,
  spec: Partial<WindowSpec> = {},
  patch: Partial<HarnessView> = {},
): HarnessView {
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
    ...patch,
  });
}

/** Available falls to 20 % with 1 580 minutes to go: alarming by threshold, calm by pace. */
const watchful = (id: HarnessId) =>
  reading(id, {
    id: 'weekly',
    kind: 'weekly',
    usedPct: 80,
    windowMinutes: 10080,
    resetsInMin: 1580,
  });

const blocked = reading('codex', { usedPct: 100 });
const risking = reading('zai', { usedPct: 90 });
const fine = reading('claude', { usedPct: 48 });
const unknown = reading('deepseek', { usedPct: null });

describe('triage ordering', () => {
  it('puts what blocks you first and unknown readings last', () => {
    const rows = triageOf([fine, unknown, watchful('kimi'), blocked, risking], ctx).rows;
    expect(rows.map((row) => row.tier)).toEqual(['blocked', 'risk', 'watch', 'fine', 'unknown']);
    expect(rows.map((row) => row.view.id)).toEqual(['codex', 'zai', 'kimi', 'claude', 'deepseek']);
  });

  it('ranks two at-risk providers by how soon they run out', () => {
    const soon = reading('codex', { usedPct: 95, resetsInMin: 60 });
    const later = reading('zai', { usedPct: 70, resetsInMin: 60 });
    const rows = triageOf([later, soon], ctx).rows;
    expect(rows.map((row) => row.view.id)).toEqual(['codex', 'zai']);
  });

  it('sends a disabled source to no group at all and a failure to attention', () => {
    const triage = triageOf(
      [reading('codex', {}, { state: 'error', error: problem('binary_not_found') })],
      ctx,
    );
    expect(triage.attention).toHaveLength(1);
    expect(triage.unknown).toHaveLength(0);
    const expected = describeError(
      { code: 'binary_not_found', retryable: false, params: {} },
      'codex',
    );
    expect(triage.attention[0]?.verdict).toBe(expected.title);
  });

  it('keeps a stale reading out of the numbers and says how old it is', () => {
    const row = triageRow(asStale(fine, env, 23), ctx);
    expect(row.tier).toBe('broken');
    expect(row.verdict).toBe('Leitura antiga · há 23 min');
    expect(row.model).toBeNull();
    expect(row.runsOutAt).toBeNull();
  });

  it('never reports a source still being read as a source with no reading', () => {
    const loading = viewOf('codex', { state: 'loading' });
    const triage = triageOf([loading], ctx);
    expect(triage.loading).toHaveLength(1);
    expect(triage.unknown).toHaveLength(0);
    expect(triage.attention).toHaveLength(0);
    expect(triage.rows[0]?.tier).toBe('loading');
    expect(triage.rows[0]?.verdict).toBe('Lendo…');
  });
});

describe('triage verdicts', () => {
  it('names the moment a quota runs out and the stretch with no quota', () => {
    const row = triageRow(risking, ctx);
    expect(row.tier).toBe('risk');
    expect(row.verdict).toContain('Esgota às');
    expect(row.verdict).toContain('sem cota');
    expect(row.runsOutAt).not.toBeNull();
  });

  it('treats an exhausted window as blocked with no projection left', () => {
    const row = triageRow(blocked, ctx);
    expect(row.tier).toBe('blocked');
    expect(row.verdict).toContain('Esgotado · volta às');
    expect(row.runsOutAt).toBe(ctx.nowSecs);
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
    const first = pooled.windows[0];
    if (first === undefined) throw new Error('no window');
    expect(quotaName(first)).toBe('Gemini · 5 horas');
  });

  it('says a comfortable reading reaches its renewal and never invents a pace', () => {
    expect(triageRow(fine, ctx).verdict).toBe('Dura até renovar');
    expect(triageRow(fine, ctx).short).toBe('');
    expect(triageRow(unknown, ctx).verdict).toBe('Sem leitura');
    expect(triageRow(unknown, ctx).tier).toBe('unknown');
  });

  it('will not project from a window with no burn rate yet', () => {
    const early = reading('codex', {
      id: 'weekly',
      kind: 'weekly',
      usedPct: 2,
      windowMinutes: 10080,
      resetsInMin: 10000,
    });
    expect(triageRow(early, ctx).verdict).toBe('Cedo para prever');
  });
});

describe('triage headline', () => {
  it('leads with the first provider and counts the rest', () => {
    const headline = headlineOf(triageOf([fine, blocked, risking], ctx));
    expect(headline.tone).toBe('attention');
    expect(headline.text).toContain('mais 1 provider');
    expect(headline.ids).toEqual(['codex', 'zai']);
  });

  it('calls a threshold alarm when nothing is projected to run out', () => {
    const headline = headlineOf(triageOf([fine, watchful('kimi')], ctx));
    expect(headline.tone).toBe('attention');
    expect(headline.text).toBe('1 provider precisa de atenção');
    expect(headline.ids).toEqual(['kimi']);
  });

  it('says everything is fine, and mentions the sources it could not read yet', () => {
    expect(headlineOf(triageOf([fine], ctx)).text).toBe('Nada esgota antes de renovar');
    expect(headlineOf(triageOf([fine, unknown], ctx)).text).toContain('1 sem leitura');
  });

  it('says it is reading instead of calling the first snapshot empty', () => {
    const loading = [viewOf('claude', { state: 'loading' }), viewOf('codex', { state: 'loading' })];
    expect(headlineOf(triageOf(loading, ctx)).text).toBe('Lendo 2 fontes…');
    expect(headlineOf(triageOf([fine, ...loading], ctx)).text).toContain('2 lendo');
  });
});
