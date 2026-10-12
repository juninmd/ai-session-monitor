import { screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { BackendError } from '../../../lib/backend';
import { claudeView } from '../../../mock/builders';
import { ctxOf, envOf, renderPanel, settingsWith } from '../../../test/helpers';
import { HarnessPanel } from '../../HarnessPanel';

const view = () => claudeView(envOf(), 'ok');

function show(options: Parameters<typeof renderPanel>[1] = {}, settingsOn = true) {
  const settings = settingsWith((s) => {
    s.general.historyEnabled = settingsOn;
  });
  return renderPanel(<HarnessPanel view={view()} ctx={ctxOf(settings)} />, {
    scenario: 'demo',
    settings,
    ...options,
  });
}

const toggle = () => screen.getByRole('button', { name: 'Histórico' });

describe('the Histórico section', () => {
  it('is collapsed by default, labelled and does not ask for the history yet', () => {
    const { backend } = show();
    expect(toggle()).toHaveAttribute('aria-expanded', 'false');
    const body = document.getElementById(toggle().getAttribute('aria-controls') ?? '');
    expect(body).toHaveAttribute('hidden');
    expect(backend.calls).not.toContain('get_history');
  });

  it('opens from the keyboard and draws the chart with a text alternative', async () => {
    const { user, backend } = show();
    toggle().focus();
    await user.keyboard('{Enter}');
    expect(toggle()).toHaveAttribute('aria-expanded', 'true');
    const chart = await screen.findByRole('img', { name: /Uso de .* ao longo da janela/ });
    expect(chart.querySelectorAll('polyline.bd-cur')).toHaveLength(1);
    expect(chart.querySelectorAll('polyline.bd-past').length).toBeGreaterThan(0);
    expect(chart.querySelector('desc')).toHaveTextContent(/janela atual/);
    expect(screen.getByText(/^Pico de \d+% .* nesta janela\./)).toBeVisible();
    expect(backend.calls).toContain('get_history');
  });

  it('gives the same numbers as a table', async () => {
    const { user } = show();
    await user.click(toggle());
    await screen.findByRole('img');
    await user.click(screen.getByText('Ver em tabela'));
    const table = screen.getByRole('table', { name: 'Picos de uso por janela' });
    expect(within(table).getAllByRole('row').length).toBeGreaterThan(2);
    expect(within(table).getByText(/\(atual\)/)).toBeVisible();
  });

  it('switches window with the selector', async () => {
    const { user } = show();
    await user.click(toggle());
    await screen.findByRole('img');
    await user.selectOptions(screen.getByLabelText('Janela'), 'weekly');
    expect(screen.getByRole('img', { name: /Uso de Semana/ })).toBeInTheDocument();
  });

  it('says so when nothing is stored yet', async () => {
    const { user } = show({ scenario: 'fresh' });
    await user.click(toggle());
    expect(await screen.findByText(/Ainda sem histórico/)).toBeVisible();
    expect(screen.queryByRole('img')).toBeNull();
  });

  it('turns a failed load into one plain sentence, never the error text', async () => {
    const { user, backend } = show();
    vi.spyOn(backend, 'getHistory').mockRejectedValue(
      new BackendError({ code: 'internal', retryable: false, params: { where: 'secret-path' } }),
    );
    await user.click(toggle());
    await waitFor(() =>
      expect(screen.getByText(/Não foi possível carregar o histórico/)).toBeVisible(),
    );
    expect(document.body.textContent).not.toContain('secret-path');
  });

  it('is absent while the setting is off', () => {
    show({}, false);
    expect(screen.queryByRole('button', { name: 'Histórico' })).toBeNull();
  });
});
