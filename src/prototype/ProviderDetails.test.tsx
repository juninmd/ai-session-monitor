import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { App } from '../App';
import { createMockBackend } from '../mock';
import { viewOf } from '../mock/fixtures';
import { startFor } from '../mock/scenarios';
import { ctxOf, freezeClock, NOW, renderPanel, win } from '../test/helpers';
import { ProviderDetails } from './ProviderDetails';

describe('discreet provider details', () => {
  it('shows bounded bars and marked forecasts, with full projection text available', () => {
    const ctx = ctxOf();
    ctx.settings.general.showUsed = true;
    const view = viewOf('codex', {
      windows: [
        win({ id: 'risk', usedPct: 80, resetsInMin: 120 }),
        win({ id: 'fine', usedPct: 25, resetsInMin: 120 }),
        win({ id: 'early', usedPct: 1, resetsInMin: 295 }),
        win({ id: 'out', usedPct: 120 }),
      ],
    });
    renderPanel(<ProviderDetails view={view} ctx={ctx} />);
    const table = screen.getByRole('table', { name: 'Cotas e renovação' });
    const risk = table.querySelector('[data-window="risk"]') as HTMLElement;
    expect(within(risk).getByText('Esgota em 45 min')).toBeVisible();
    expect(risk.querySelector('.preview-detail-forecast')).toHaveAttribute(
      'title',
      expect.stringContaining('antes do reset'),
    );
    expect(risk.querySelector('.est')).toHaveTextContent('estimativa');
    expect(risk.querySelector('.t-bar i')).toHaveStyle({ inlineSize: '80%' });
    expect(within(table).getByText('Dura até renovar')).toBeVisible();
    expect(within(table).getByText('Sem base para prever')).toBeVisible();
    const out = table.querySelector('[data-window="out"]') as HTMLElement;
    expect(out.querySelector('.t-bar i')).toHaveStyle({ inlineSize: '100%' });
    expect(out.querySelector('.est')).toBeNull();
  });

  it('keeps unknown, expired and stale data neutral without a forecast', () => {
    const ctx = ctxOf();
    const view = viewOf('codex', {
      windows: [
        win({ id: 'unknown', usedPct: null }),
        { ...win({ id: 'expired', usedPct: 95 }), resetsAt: NOW },
      ],
    });
    renderPanel(<ProviderDetails view={view} ctx={ctx} />);
    const table = screen.getByRole('table', { name: 'Cotas e renovação' });
    expect(table.querySelector('.preview-detail-forecast')).toBeNull();
    expect(table.querySelector('.t-bar i')).toBeNull();
    expect(table.querySelector('[data-status]')).toBeNull();
  });

  it('suppresses stale forecasts and respects the available percentage setting', () => {
    const ctx = ctxOf();
    ctx.settings.general.showUsed = false;
    renderPanel(
      <ProviderDetails view={viewOf('codex', { windows: [win({ usedPct: 25 })] })} ctx={ctx} />,
    );
    const table = screen.getByRole('table', { name: 'Cotas e renovação' });
    expect(table.querySelector('.t-bar i')).toHaveStyle({ inlineSize: '75%' });
    renderPanel(
      <ProviderDetails
        view={viewOf('claude', { stale: true, windows: [win({ usedPct: 95 })] })}
        ctx={ctx}
      />,
    );
    const staleTable = screen.getAllByRole('table', { name: 'Cotas e renovação' })[1];
    expect(staleTable?.querySelector('.preview-detail-forecast')).toBeNull();
    expect(staleTable?.querySelector('.t-bar i')).toBeNull();
    expect(staleTable?.querySelector('[data-status]')).toBeNull();
  });

  it('loads history on demand and stops refreshing it when the outer details close', async () => {
    freezeClock();
    const backend = createMockBackend({ ...startFor('demo'), now: () => NOW });
    const history = vi.spyOn(backend, 'getHistory');
    const user = userEvent.setup();
    render(<App backend={backend} preview />);
    await screen.findByRole('tab', { name: /^Claude Code/ });
    expect(history).not.toHaveBeenCalled();
    await screen.findByRole('button', { name: 'Histórico' });
    expect(history).not.toHaveBeenCalled();
    await user.click(screen.getByRole('button', { name: 'Histórico' }));
    const chart = await screen.findByRole('img', { name: /Uso de .* ao longo da janela/ });
    expect(chart).toBeVisible();
    expect(history).toHaveBeenCalledTimes(1);
    expect(history).toHaveBeenCalledWith('claude');
    expect(screen.getByText('Ver em tabela').closest('details')).not.toHaveAttribute('open');
    await user.click(screen.getByText('Detalhes de Claude Code'));
    await waitFor(() => expect(screen.queryByRole('button', { name: 'Histórico' })).toBeNull());
    await user.click(screen.getByRole('button', { name: 'Atualizar agora' }));
    expect(history).toHaveBeenCalledTimes(1);
    await user.click(screen.getByRole('tab', { name: /^Codex/ }));
    const toggle = await screen.findByRole('button', { name: 'Histórico' });
    expect(toggle).toHaveAttribute('aria-expanded', 'false');
    await user.click(toggle);
    await screen.findByRole('img', { name: /Uso de .* ao longo da janela/ });
    expect(history).toHaveBeenLastCalledWith('codex');
  });

  it('omits history when collection is disabled', () => {
    const ctx = ctxOf();
    ctx.settings.general.historyEnabled = false;
    const spy = vi.fn();
    renderPanel(<ProviderDetails view={viewOf('codex', { windows: [win()] })} ctx={ctx} />, {
      prepare: (backend) => {
        vi.spyOn(backend, 'getHistory').mockImplementation(spy);
      },
    });
    expect(screen.queryByRole('button', { name: 'Histórico' })).toBeNull();
    expect(spy).not.toHaveBeenCalled();
  });
});
