import { screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { HarnessPanel } from '../components/HarnessPanel';
import { EstimateMark } from '../components/panel/EstimateMark';
import { Facts } from '../components/panel/Facts';
import { claudeView, codexView, opencodeView } from '../mock/builders';
import { viewOf, windowOf } from '../mock/fixtures';
import { ctxOf, envOf, renderPanel, settingsWith, win } from '../test/helpers';
import { announcementFor } from './announce';
import { hasGuessedCadence } from './estimate';
import { sharePct } from './quota';
import { summaryOf, tabChip, traySummary } from './tabModel';
import { gaugeLabel, gaugeTip, modelWindow, paceLine } from './windowModel';

const used = settingsWith((s) => {
  s.general.showUsed = true;
});
const usedCtx = ctxOf(used);
const env = envOf();

describe('the used/available wording', () => {
  it('complements the share and keeps the stored number untouched', () => {
    expect(sharePct(52, false)).toBe(52);
    expect(sharePct(52, true)).toBe(48);
    const m = modelWindow('claude', win(), usedCtx);
    expect(m.window.usedPct).toBe(48);
    expect(m.shown).toBe(48);
    expect(m.available).toBe(52);
  });

  it('says which one in the gauge label and tooltip', () => {
    const m = modelWindow('claude', win(), usedCtx);
    expect(gaugeLabel(m)).toBe('Sessão de 5 horas: 48% usado');
    expect(gaugeTip(m, usedCtx)).toMatch(/^48% usado\./);
    const plain = modelWindow('claude', win(), ctxOf());
    expect(gaugeLabel(plain)).toBe('Sessão de 5 horas: 52% disponível');
  });

  it('keeps status and pace on the used share (colors never change)', () => {
    const a = modelWindow('claude', win({ usedPct: 90 }), ctxOf());
    const b = modelWindow('claude', win({ usedPct: 90 }), usedCtx);
    expect([b.status, b.pace]).toEqual([a.status, a.pace]);
  });

  it('flips the tab chip text, its spoken form and its bar, not its status', () => {
    const view = claudeView(env, 'ok');
    const chip = tabChip(view, summaryOf(view, usedCtx), usedCtx);
    expect(chip).toMatchObject({ kind: 'pct', status: 'ok', text: '48%', pct: 48 });
    expect(chip.sr).toBe('48% usado, dentro do ritmo');
    expect(traySummary([view], usedCtx)?.summary.status).toBe('ok');
  });

  it('flips the live announcement', () => {
    const summary = summaryOf(codexView(env, 'warn'), ctxOf());
    expect(announcementFor('codex', 'ok', 'warn', summary, true)).toBe(
      'Codex: atenção, 82% usado.',
    );
  });

  it('prints used in the hero, the meter and the amount headline', () => {
    renderPanel(<HarnessPanel view={claudeView(env, 'ok')} ctx={usedCtx} />, { settings: used });
    const hero = screen.getByRole('heading', { name: 'Sessão de 5 horas' }).closest('section');
    if (!hero) throw new Error('no hero');
    expect(within(hero).getByText('48%')).toBeVisible();
    expect(within(hero).getByText('usado')).toBeVisible();
    expect(within(hero).getByRole('meter')).toHaveAttribute('value', '48');
  });
});

describe('the estimate marker', () => {
  it('reads as "estimativa" to a screen reader', () => {
    renderPanel(<EstimateMark />);
    expect(screen.getByText('~')).toHaveAttribute('aria-hidden', 'true');
    expect(screen.getByText('estimativa:')).toHaveClass('vh');
  });

  it('marks a pace projection, never a measured state', () => {
    const risk = paceLine(modelWindow('claude', win({ usedPct: 90 }), ctxOf()), ctxOf());
    expect(risk).toMatchObject({ estimate: true });
    const fine = paceLine(modelWindow('claude', win({ usedPct: 20 }), ctxOf()), ctxOf());
    expect(fine).toMatchObject({ estimate: true });
    const out = paceLine(modelWindow('claude', win({ usedPct: 100 }), ctxOf()), ctxOf());
    expect(out).toMatchObject({ estimate: false });
    const early = paceLine(modelWindow('claude', win({ usedPct: 1 }), ctxOf()), ctxOf());
    expect(early).toMatchObject({ estimate: false });
  });

  it('shows the marker on a pace sentence', () => {
    renderPanel(<HarnessPanel view={claudeView(env, 'ok')} ctx={ctxOf()} />);
    const sentence = screen.getByText('No ritmo atual, dura além do reset.');
    expect(within(sentence).getByText('~')).toBeInTheDocument();
  });

  it('knows a guessed antigravity cadence by its missing length', () => {
    const guessed = windowOf(env, {
      id: 'gemini:five_hour',
      kind: 'five_hour',
      usedPct: 40,
      windowMinutes: null,
      resetsInMin: 100,
    });
    const stated = { ...guessed, windowMinutes: 300 };
    expect(hasGuessedCadence('antigravity', guessed)).toBe(true);
    expect(hasGuessedCadence('antigravity', stated)).toBe(false);
    expect(hasGuessedCadence('opencode', guessed)).toBe(false);
    expect(hasGuessedCadence('antigravity', { ...guessed, kind: 'other' })).toBe(false);
    const m = modelWindow('antigravity', guessed, ctxOf());
    expect(gaugeLabel(m)).toContain('(estimativa)');
  });

  it('marks the OpenCode local facts and nothing else', () => {
    const local = opencodeView(
      envOf({ settings: settingsWith((s) => (s.opencode.localConsent = true)) }),
      'ok',
    );
    renderPanel(<Facts view={local} nowSecs={ctxOf().nowSecs} />);
    const costs = screen.getAllByText('~');
    expect(costs).toHaveLength(local.facts.length);
    const codex = viewOf('codex', {
      facts: [{ id: 'credits_balance', number: 3, text: null, unit: 'credits' }],
    });
    renderPanel(<Facts view={codex} nowSecs={ctxOf().nowSecs} />);
    expect(screen.getAllByText('~')).toHaveLength(local.facts.length);
  });
});

describe('earned reset credits', () => {
  const fact = (number: number) => ({ id: 'reset_credits', number, text: null, unit: 'resets' });
  it.each([
    [1, '1 disponível'],
    [3, '3 disponíveis'],
    [0, '0 disponíveis'],
  ])('%s reads "%s"', (n, text) => {
    renderPanel(<Facts view={viewOf('codex', { facts: [fact(n)] })} nowSecs={ctxOf().nowSecs} />);
    expect(screen.getByText('Recargas de reset')).toBeVisible();
    expect(screen.getByText(text)).toBeVisible();
  });
});
