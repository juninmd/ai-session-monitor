import { act, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { BackendError } from './lib/backend';
import type { MockBackend } from './mock';
import { claudeView } from './mock/builders';
import { problem } from './mock/fixtures';
import { envOf, NOW, renderApp } from './test/helpers';

const tab = (name: string) => screen.getByRole('tab', { name: new RegExp(`^${name}\\b`) });
const ready = () => screen.findByRole('heading', { name: 'Sessão de 5 horas' });
const refreshes = (calls: readonly string[]) => calls.filter((call) => call === 'refresh').length;
const footer = () => document.querySelector('.fly-foot') as HTMLElement;

describe('when the flyout is shown again', () => {
  it('refocuses the selected tab and reads everything once', async () => {
    const { user, backend } = renderApp();
    await ready();
    await user.click(tab('Codex'));
    screen.getByRole('button', { name: 'Atualizar agora' }).focus();
    const before = refreshes(backend.calls);

    act(() => backend.emitFlyoutShown());

    expect(tab('Codex')).toHaveFocus();
    await vi.waitFor(() => expect(refreshes(backend.calls)).toBe(before + 1));
  });

  it('puts focus on the settings heading when settings are what was open', async () => {
    const { user, backend } = renderApp();
    await ready();
    await user.click(screen.getByRole('button', { name: 'Configurações' }));
    screen.getByRole('button', { name: 'Voltar' }).focus();

    act(() => backend.emitFlyoutShown());

    expect(screen.getByRole('heading', { level: 2, name: 'Configurações' })).toHaveFocus();
  });

  it('brings the countdowns up to date at once instead of waiting for the next tick', async () => {
    const { backend } = renderApp();
    await ready();
    expect(screen.getByText('Lido há 15 s')).toBeVisible();

    vi.setSystemTime((NOW + 3 * 60) * 1000);
    expect(screen.queryByText('Lido há 3 min')).toBeNull();
    act(() => backend.emitFlyoutShown());

    expect(screen.getByText(/^Lido há 3 min/)).toBeVisible();
  });

  it('says so when the refresh it triggers fails', async () => {
    const { backend } = renderApp();
    await ready();
    vi.spyOn(backend, 'refresh').mockRejectedValue(new BackendError(problem('internal')));
    act(() => backend.emitFlyoutShown());
    expect(await screen.findByRole('alert')).toHaveTextContent('Algo deu errado.');
  });
});

describe('the 20 second tick', () => {
  const run = (seconds: number) => act(() => vi.advanceTimersByTime(seconds * 1000));

  it('advances countdowns and ages without a new snapshot', async () => {
    const { backend } = renderApp({}, ['setInterval', 'clearInterval']);
    await ready();
    const asked = backend.calls.length;
    const countdown = () => document.querySelector('.hero-time');
    expect(countdown()).toHaveTextContent('1h53m');

    run(60);

    expect(countdown()).toHaveTextContent('1h52m');
    expect(footer()).toHaveTextContent('Lido há 1 min');
    expect(backend.calls).toHaveLength(asked);
  });

  it('keeps the keyboard where it was', async () => {
    const { user } = renderApp({}, ['setInterval', 'clearInterval']);
    await ready();
    await user.click(tab('Codex'));
    tab('Codex').focus();
    run(40);
    expect(tab('Codex')).toHaveFocus();
    expect(tab('Codex')).toHaveAttribute('aria-selected', 'true');
  });
});

describe('the live region', () => {
  const live = () => document.querySelector('output[aria-live="polite"]') as HTMLElement;

  it('stays silent for the first snapshot, however bad it is', async () => {
    renderApp({ scenario: 'critical' });
    await screen.findByRole('heading', { name: 'Sessão de 5 horas' });
    expect(live()).toBeEmptyDOMElement();
  });

  it('speaks a change of status once it happens, and each later one', async () => {
    const { backend } = renderApp();
    await ready();
    expect(live()).toBeEmptyDOMElement();

    act(() => backend.override(claudeView(envOf(), 'crit')));
    await vi.waitFor(() => expect(live()).toHaveTextContent('Claude Code: esgotado.'));

    act(() => backend.override(claudeView(envOf(), 'warn')));
    await vi.waitFor(() =>
      expect(live()).toHaveTextContent(/^Claude Code: atenção, \d+% disponível\.$/),
    );
  });

  it('is polite and read as a whole', async () => {
    renderApp();
    await ready();
    expect(live()).toHaveAttribute('aria-atomic', 'true');
    expect(live()).toHaveClass('vh');
  });
});

describe('when the first snapshot cannot be loaded', () => {
  const failFirstRead = (backend: MockBackend) => {
    vi.spyOn(backend, 'getSnapshot').mockRejectedValueOnce(new BackendError(problem('internal')));
  };

  it('says so with one way to try again, and recovers', async () => {
    const { user } = renderApp({ prepare: failFirstRead });
    expect(
      await screen.findByRole('heading', { name: 'Não foi possível carregar o monitor' }),
    ).toBeVisible();
    expect(screen.getAllByRole('button', { name: 'Tentar de novo' })).toHaveLength(1);
    await user.click(screen.getByRole('button', { name: 'Tentar de novo' }));
    expect(await ready()).toBeVisible();
    expect(
      screen.queryByRole('heading', { name: 'Não foi possível carregar o monitor' }),
    ).toBeNull();
  });

  it('does not stand in the way of the settings', async () => {
    const { user } = renderApp({ prepare: failFirstRead });
    await screen.findByRole('heading', { name: 'Não foi possível carregar o monitor' });
    await user.click(screen.getByRole('button', { name: 'Configurações' }));
    expect(screen.getByRole('region', { name: 'Configurações' })).toBeVisible();
    expect(
      screen.queryByRole('heading', { name: 'Não foi possível carregar o monitor' }),
    ).toBeNull();
  });
});

describe('when the event bridge is broken', () => {
  it('warns that the numbers may stop moving, and still shows what it has', async () => {
    renderApp({
      prepare: (backend) => {
        vi.spyOn(backend, 'onSnapshot').mockRejectedValue(new Error('no bridge'));
      },
    });
    expect(await ready()).toBeVisible();
    expect(await screen.findByText(/Não foi possível acompanhar as atualizações/)).toBeVisible();
  });
});

describe('a manual refresh during a rate-limit cool-down', () => {
  it('is still sent but tells the person why nothing will happen yet', async () => {
    const { user, backend } = renderApp({ scenario: 'stale' });
    await screen.findByRole('tab', { name: /^LiteLLM\b/ });
    await user.click(tab('LiteLLM'));
    await user.click(screen.getByRole('button', { name: 'Atualizar agora' }));

    const live = document.querySelector('output[aria-live="polite"]') as HTMLElement;
    expect(live).toHaveTextContent(
      'LiteLLM: a fonte pediu um intervalo, então a leitura manual espera. Nova tentativa em 4 min.',
    );
    expect(refreshes(backend.calls)).toBeGreaterThan(0);
  });

  it('says nothing special for a source that is not cooling', async () => {
    const { user } = renderApp({ scenario: 'stale' });
    await ready();
    await user.click(screen.getByRole('button', { name: 'Atualizar agora' }));
    const live = document.querySelector('output[aria-live="polite"]') as HTMLElement;
    expect(live).not.toHaveTextContent(/Nova tentativa/);
  });
});
