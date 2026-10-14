import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { App } from '../App';
import type { ViewContext } from '../app/windowModel';
import { defaultSettings, type HarnessView, type WindowKind } from '../lib/contract';
import { createMockBackend } from '../mock';
import { viewOf, windowOf } from '../mock/fixtures';
import { startFor } from '../mock/scenarios';
import { freezeClock, NOW, renderPanel } from '../test/helpers';
import { ProviderDetails } from './ProviderDetails';
import { ProviderGrid } from './ProviderGrid';
import { gridQuotas } from './quotaModel';

const settings = structuredClone(defaultSettings);
settings.general.showUsed = true;
const ctx: ViewContext = { nowSecs: NOW, settings };
const env = { now: NOW, settings, secrets: startFor('demo').secrets };
const quota = (id: string, kind: WindowKind, usedPct: number | null = 25) =>
  windowOf(env, { id, kind, usedPct, windowMinutes: 300, resetsInMin: 120 });
const grid = (view: HarnessView, context = ctx) =>
  render(<ProviderGrid views={[view]} selected={view.id} ctx={context} onSelect={vi.fn()} />);

describe('compact provider quotas', () => {
  it('summarizes the limiting pool and keeps every quota in the compact details', () => {
    const windows = [
      quota('session', 'five_hour'),
      quota('week', 'weekly'),
      quota('month', 'monthly'),
      quota('day', 'daily'),
      quota('budget', 'budget'),
      quota('custom', 'other'),
      { ...quota('gemini-week', 'weekly', 90), group: 'gemini' },
      { ...quota('claude-week', 'weekly', null), group: 'claude_gpt' },
    ];
    const view = viewOf('codex', { windows });
    grid(view);
    const tab = screen.getByRole('tab');
    expect(tab.querySelectorAll('.preview-quota')).toHaveLength(1);
    expect(tab).toHaveAccessibleName(/Semana: 90% usado/);
    expect(within(tab).getByText('+7 cotas nos detalhes')).toBeVisible();
    expect(within(tab).queryByText('Claude e GPT-OSS')).toBeNull();
    renderPanel(<ProviderDetails view={view} ctx={ctx} />);
    const table = screen.getByRole('table', { name: 'Cotas e renovação' });
    expect(table.querySelectorAll('tr[data-window]')).toHaveLength(8);
    for (const label of [
      '5 horas',
      'Semana',
      'Mês',
      'Dia',
      'Orçamento',
      'Gemini',
      'Claude e GPT-OSS',
    ]) {
      expect(within(table).getAllByText(label).length).toBeGreaterThan(0);
    }
    expect(
      within(table.querySelector('[data-window="claude-week"]') as HTMLElement).getByText('—'),
    ).toBeVisible();
  });
  it('does not print or fill fresh percentages for unknown, expired or stale readings', () => {
    const view = viewOf('codex', {
      windows: [
        quota('unknown', 'weekly', null),
        { ...quota('expired', 'monthly', 100), resetsAt: NOW },
      ],
    });
    const { rerender } = grid(view);
    expect(screen.getByRole('tab').querySelectorAll('.t-bar i')).toHaveLength(0);
    expect(screen.getByText('—')).toBeVisible();
    expect(screen.getByText('Expirou')).toBeVisible();
    expect(screen.getByRole('tab').querySelector('[data-status]')).toBeNull();
    rerender(
      <ProviderGrid
        views={[viewOf('codex', { stale: true, windows: [quota('old', 'weekly', 95)] })]}
        selected="codex"
        ctx={ctx}
        onSelect={vi.fn()}
      />,
    );
    expect(screen.getByText('Antigo')).toBeVisible();
    expect(screen.getByRole('tab')).not.toHaveAccessibleName(/95%/);
    expect(screen.getByRole('tab').querySelectorAll('.t-bar i')).toHaveLength(0);
  });
  it('respects available usage mode and clamps over-budget bars', () => {
    const view = viewOf('codex', { windows: [quota('session', 'five_hour', 120)] });
    const { rerender } = grid(view);
    expect(screen.getByText('100%')).toBeVisible();
    expect(screen.getByRole('tab').querySelector('i')).toHaveStyle({ inlineSize: '100%' });
    const available = {
      ...ctx,
      settings: { ...settings, general: { ...settings.general, showUsed: false } },
    };
    rerender(
      <ProviderGrid
        views={[viewOf('codex', { windows: [quota('session', 'five_hour', 25)] })]}
        selected="codex"
        ctx={available}
        onSelect={vi.fn()}
      />,
    );
    expect(screen.getByText('75%')).toBeVisible();
    expect(screen.getByRole('tab')).toHaveAccessibleName(/75% disponível/);
  });
  it('moves unofficial-source information into a hover and keyboard tooltip', async () => {
    const user = userEvent.setup();
    grid(viewOf('antigravity', { source: { id: 'language_server', official: false } }));
    const info = screen.getByRole('img', { name: 'Leitura não oficial' });
    expect(screen.queryByRole('tooltip')).toBeNull();
    await user.hover(info);
    expect(await screen.findByRole('tooltip')).toHaveTextContent('Leitura não oficial');
    await user.hover(screen.getByRole('tooltip'));
    expect(screen.getByRole('tooltip')).toBeVisible();
    await user.keyboard('{Escape}');
    expect(screen.queryByRole('tooltip')).toBeNull();
    await user.tab();
    expect(screen.getByRole('tab')).toHaveFocus();
    expect(await screen.findByRole('tooltip')).toBeVisible();
    await user.keyboard('{Escape}');
    expect(screen.queryByRole('tooltip')).toBeNull();
  });
  it('caps visible quotas at three while retaining a later critical window', () => {
    const view = viewOf('codex', {
      windows: [
        quota('session', 'five_hour'),
        quota('week', 'weekly'),
        quota('month', 'monthly'),
        quota('day', 'daily'),
        quota('budget', 'budget', 95),
      ],
    });
    const summary = gridQuotas(view, ctx);
    expect(summary.windows).toHaveLength(3);
    expect(summary.windows.map((window) => window.id)).toContain('budget');
    expect(summary.extra).toBe(2);
  });
  it('switches grouped summaries to the most constrained live pool', () => {
    const view = viewOf('antigravity', {
      windows: [
        { ...quota('gemini', 'five_hour', 10), group: 'gemini' },
        { ...quota('claude', 'five_hour', 95), group: 'claude_gpt' },
        { ...quota('expired', 'weekly', 100), resetsAt: NOW, group: 'gemini' },
      ],
    });
    expect(gridQuotas(view, ctx).group).toBe('Claude e GPT-OSS');
    expect(gridQuotas(view, ctx).windows.map((window) => window.id)).toEqual(['claude']);
  });
  it('keeps facts and notices collapsed and opens provider configuration from the brief details', async () => {
    const view = viewOf('codex', {
      windows: [quota('session', 'five_hour')],
      facts: [{ id: 'test_balance', number: 12, text: null, unit: 'USD' }],
      notices: [{ code: 'test_notice', severity: 'warn', params: {} }],
    });
    const { user, openSettings } = renderPanel(<ProviderDetails view={view} ctx={ctx} />);
    const summary = screen.getByText('Informações adicionais');
    expect(summary.closest('details')).not.toHaveAttribute('open');
    expect(screen.getByText('· 1 aviso')).toBeVisible();
    await user.click(summary);
    expect(summary.closest('details')).toHaveAttribute('open');
    expect(screen.getByText('test_balance')).toBeVisible();
    await user.click(screen.getByRole('button', { name: /^Configurar$/ }));
    expect(openSettings).toHaveBeenCalledWith('codex');
  });
  it('starts with details collapsed and collapses them on provider change', async () => {
    freezeClock();
    const user = userEvent.setup();
    render(<App backend={createMockBackend({ ...startFor('demo'), now: () => NOW })} preview />);
    await screen.findByRole('tab', { name: /^Claude Code/ });
    expect(screen.getByRole('tabpanel').querySelector('details')).not.toHaveAttribute('open');
    await user.click(screen.getByText('Detalhes de Claude Code'));
    expect(screen.getByRole('tabpanel').querySelector('details')).toHaveAttribute('open');
    await user.click(screen.getByRole('tab', { name: /^Codex/ }));
    expect(screen.getByRole('tabpanel').querySelector('details')).not.toHaveAttribute('open');
  });
});
