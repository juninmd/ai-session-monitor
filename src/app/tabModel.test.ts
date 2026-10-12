import { describe, expect, it } from 'vitest';
import type { HarnessView } from '../lib/contract';
import { antigravityView, claudeView, codexView, opencodeView } from '../mock/builders';
import { asStale, failed, problem, viewOf } from '../mock/fixtures';
import { ctxOf, envOf, NOW, settingsWith, win } from '../test/helpers';
import { STATUS_GLYPH, summaryOf, tabChip, traySummary } from './tabModel';

const ctx = ctxOf();
const env = envOf();
const chip = (view: HarnessView, c = ctx) => tabChip(view, summaryOf(view, c), c);

describe('tabChip by state', () => {
  it('shows a skeleton while loading', () => {
    expect(chip(viewOf('codex', { state: 'loading' }))).toEqual({
      kind: 'loading',
      sr: 'carregando',
    });
  });

  it.each([
    ['disabled', 'Desligado', 'desligado'],
    ['needs_consent', 'Conectar', 'precisa ser conectado'],
    ['needs_setup', 'Conectar', 'precisa ser conectado'],
    ['no_data', 'Sem dados', 'sem dados ainda'],
  ] as const)('%s reads "%s"', (state, text, sr) => {
    expect(chip(viewOf('codex', { state }))).toMatchObject({ kind: 'text', text, sr, glyph: null });
  });

  it('marks a failed read with a warning glyph, not just a word', () => {
    const view = failed('codex', problem('timeout', {}, true));
    expect(chip(view)).toMatchObject({ kind: 'text', text: 'Falha', glyph: 'warn', warn: true });
  });
});

describe('tabChip for a ready view', () => {
  it('shows the most constrained window and its status', () => {
    const result = chip(claudeView(env, 'ok'));
    expect(result).toMatchObject({ kind: 'pct', status: 'ok', text: '52%', pct: 52 });
    expect(result.sr).toBe('52% disponível, dentro do ritmo');
  });

  it.each([
    ['warn', 'atenção'],
    ['crit', 'crítico'],
  ] as const)('speaks the %s status in words', (mood, word) => {
    const result = chip(codexView(env, mood));
    expect(result.kind).toBe('pct');
    expect(result.sr).toContain(word);
  });

  it('says exhausted for an empty window', () => {
    const view = claudeView(env, 'crit');
    expect(chip(view)).toMatchObject({ kind: 'pct', status: 'out', text: '0%' });
  });

  it('gives a glyph to every status that is not ok', () => {
    expect(STATUS_GLYPH).toEqual({ ok: 'check', warn: 'warn', crit: 'alert', out: 'ban' });
  });

  it('dims stale data into a word with a clock, or a warning when the refresh failed', () => {
    const base = claudeView(env, 'ok');
    expect(chip(asStale(base, env, 20))).toMatchObject({
      text: 'Antigo',
      glyph: 'clock',
      warn: false,
    });
    expect(chip(asStale(base, env, 20, problem('network', {}, true)))).toMatchObject({
      text: 'Antigo',
      glyph: 'warn',
      warn: true,
    });
  });

  it('says a reset window reset instead of showing its old percentage', () => {
    const expired = viewOf('codex', { windows: [win({ resetsInMin: -5 })] });
    expect(chip(expired)).toMatchObject({ text: 'Reiniciou', glyph: 'refresh' });
  });

  it('shows a dash, never 0%, when every window is unknown', () => {
    const unknown = viewOf('codex', { windows: [win({ usedPct: null })] });
    const result = chip(unknown);
    expect(result).toMatchObject({ kind: 'text', text: '—' });
    expect(JSON.stringify(result)).not.toContain('0%');
  });

  it('says there is no quota when the source reported no windows', () => {
    expect(chip(viewOf('codex'))).toMatchObject({ text: 'Sem cota' });
  });

  it('ignores unknown pools when another window is readable', () => {
    expect(chip(antigravityView(env, 'ok', true))).toMatchObject({ kind: 'pct', text: '46%' });
  });
});

describe('traySummary', () => {
  const views = [claudeView(env, 'ok'), codexView(env, 'warn'), opencodeView(env, 'ok')];

  it('follows the lowest available window of all sources by default', () => {
    expect(traySummary(views, ctx)).toMatchObject({
      id: 'codex',
      summary: { availablePct: 18, status: 'warn' },
    });
  });

  it('follows the chosen source only', () => {
    const only = ctxOf(
      settingsWith((s) => {
        s.general.traySource = 'claude';
      }),
    );
    expect(traySummary(views, only)).toMatchObject({ id: 'claude', summary: { availablePct: 52 } });
  });

  it('is empty when the chosen source has nothing readable', () => {
    const only = ctxOf(
      settingsWith((s) => {
        s.general.traySource = 'litellm';
      }),
    );
    expect(traySummary(views, only)).toBeNull();
  });

  it('leaves stale and failed views out', () => {
    const mixed = [asStale(codexView(env, 'crit'), env, 30), failed('claude', problem('network'))];
    expect(traySummary(mixed, ctx)).toBeNull();
  });
});

describe('summaryOf', () => {
  it('is only defined for fresh ready views', () => {
    expect(summaryOf(viewOf('codex', { state: 'loading' }), ctx)).toBeNull();
    expect(summaryOf(asStale(codexView(env, 'ok'), env, 10), ctx)).toBeNull();
    expect(summaryOf(codexView(env, 'ok'), ctx)?.availablePct).toBe(72);
  });

  it('stops counting a window once its reset has passed', () => {
    const later = { ...ctx, nowSecs: NOW + 400 * 60 };
    // The five-hour window reset 200 minutes ago; the weekly one still speaks for the tab.
    expect(summaryOf(claudeView(env, 'ok'), later)).toMatchObject({
      windowId: 'weekly',
      availablePct: 69,
    });
  });
});
