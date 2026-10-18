import { act, screen, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { RANK } from './home/testHarness';
import { BackendError } from './lib/backend';
import { problem } from './mock/fixtures';
import { renderApp } from './test/helpers';

const NAMES = [
  'Claude Code',
  'Codex',
  'Antigravity',
  'OpenCode',
  'Kimi Code',
  'z.ai',
  'OpenRouter',
  'DeepSeek',
  'DeepInfra',
  'Moonshot',
  'Novita',
  'Ollama Cloud',
  'MiniMax',
  'LiteLLM',
];
const tile = (name: string) => screen.getByRole('button', { name: new RegExp(`^${name}`) });
const nav = (name: string) => screen.getByRole('button', { name: new RegExp(`^${name}`) });
// The reading follows the name with no space ("Codex18%"), so the match stops at the name.
const ready = () => screen.findByRole('button', { name: /^Codex/ });
const gear = () => screen.getByRole('button', { name: 'Configurações' });
const drawer = () => screen.getByRole('dialog');

describe('the shell', () => {
  it('shows the title, the surfaces and every monitored provider in urgency order', async () => {
    renderApp();
    expect(screen.getByRole('heading', { level: 1, name: 'AI Session Monitor' })).toBeVisible();
    // The switch only appears once the stored settings say what is being monitored.
    expect(await screen.findByRole('button', { name: /^Triagem/ })).toHaveAttribute(
      'aria-pressed',
      'true',
    );
    expect(nav('Horizonte')).toBeVisible();
    expect(nav('Canais de cota')).toBeVisible();
    await ready();
    const tiles = [...document.querySelectorAll<HTMLElement>('button[data-tier]')];
    expect(tiles).toHaveLength(NAMES.length);
    const ranks = tiles.map((node) => RANK[node.getAttribute('data-tier') ?? 'unknown'] ?? 6);
    expect(ranks).toEqual([...ranks].sort((a, b) => a - b));
    for (const name of NAMES) expect(tile(name)).toBeVisible();
  });

  it('puts the lowest availability on the title, where the tray icon takes it from', async () => {
    renderApp();
    await ready();
    expect(screen.getByRole('heading', { level: 1 })).toHaveAttribute(
      'title',
      'Menor disponível: 18%',
    );
  });

  it('says there is no reading yet before the first snapshot', () => {
    renderApp({ latencyMs: 50 });
    expect(screen.getByRole('heading', { level: 1 })).toHaveAttribute(
      'title',
      'Sem leitura disponível',
    );
    expect(screen.getByText('Lendo as fontes…')).toBeVisible();
  });

  it('shows the age of the newest reading, from the clock it was given', async () => {
    renderApp();
    await ready();
    expect(screen.getByText('Lido há 15 s')).toBeVisible();
  });
});

describe('moving between surfaces', () => {
  it('shows the surface that was clicked, and starts it at the top', async () => {
    const { user } = renderApp();
    await ready();
    const body = document.querySelector('.fly-body') as HTMLElement;
    body.scrollTop = 160;
    await user.click(nav('Horizonte'));
    expect(screen.getByRole('heading', { name: 'Chego até a renovação?' })).toBeVisible();
    expect(document.querySelector('button[data-tier]')).toBeNull();
    expect(body.scrollTop).toBe(0);
    await user.click(nav('Canais de cota'));
    expect(screen.getByRole('heading', { name: 'O que ainda dá para ler' })).toBeVisible();
    await user.click(nav('Triagem'));
    expect(await ready()).toBeVisible();
  });

  it('shows the failure of a tool in its own drawer, with one way forward', async () => {
    const { user } = renderApp();
    await ready();
    await user.click(tile('LiteLLM'));
    const panel = drawer();
    expect(
      await within(panel).findByRole('heading', { name: 'Chave ou login recusado' }),
    ).toBeVisible();
    const block = panel.querySelector('.card.notice') as HTMLElement;
    expect(within(block).getAllByRole('button')).toHaveLength(1);
  });

  it('opens the detail only for the provider that was activated', async () => {
    const { user } = renderApp();
    await ready();
    expect(screen.queryByRole('dialog')).toBeNull();
    await user.click(tile('Antigravity'));
    expect(within(drawer()).getByRole('heading', { level: 2, name: 'Antigravity' })).toBeVisible();
  });
});

describe('the buttons', () => {
  it('ask the backend to read everything now', async () => {
    const { user, backend } = renderApp();
    await ready();
    await user.click(screen.getByRole('button', { name: 'Atualizar agora' }));
    await user.click(screen.getByRole('button', { name: 'Atualizar' }));
    expect(backend.calls.filter((call) => call === 'refresh')).toHaveLength(2);
  });

  it('quit the app from the footer', async () => {
    const { user, backend } = renderApp();
    await ready();
    await user.click(screen.getByRole('button', { name: 'Sair' }));
    expect(backend.calls).toContain('quit_app');
  });

  it.each([
    ['Atualizar agora', 'refresh'],
    ['Atualizar', 'refresh'],
    ['Sair', 'quit'],
  ] as const)('say so in words when "%s" fails, without the raw error', async (label, method) => {
    const { user, backend } = renderApp();
    await ready();
    vi.spyOn(backend, method).mockRejectedValue(new Error('pipe closed at 0x7ffe'));
    await user.click(screen.getByRole('button', { name: label }));
    const alert = await screen.findByRole('alert');
    expect(alert).toHaveTextContent('Algo deu errado.');
    expect(alert).not.toHaveTextContent('0x7ffe');
  });

  it('opens the update settings from the banner, without downloading anything', async () => {
    const { user, backend } = renderApp();
    await ready();
    await act(async () => {
      await backend.checkUpdate();
    });
    const banner = await screen.findByText('Versão 0.2.0 disponível');
    await user.click(within(banner.closest('.update-banner') as HTMLElement).getByRole('button'));
    expect(await screen.findByRole('heading', { name: /^Atualizações$/ })).toBeVisible();
    expect(backend.calls).not.toContain('download_update');
    expect(backend.calls).not.toContain('install_update');
  });
});

describe('settings', () => {
  it('open from the gear, replace the surfaces, and close from the same button', async () => {
    const { user } = renderApp();
    await ready();
    expect(gear()).toHaveAttribute('aria-pressed', 'false');
    await user.click(gear());
    expect(gear()).toHaveAttribute('aria-pressed', 'true');
    expect(screen.queryByRole('button', { name: /^Triagem/ })).toBeNull();
    expect(screen.getByRole('region', { name: 'Configurações' })).toBeVisible();
    expect(screen.getByRole('heading', { level: 2, name: 'Configurações' })).toHaveFocus();
    await user.click(gear());
    expect(nav('Triagem')).toBeVisible();
    expect(gear()).toHaveAttribute('aria-pressed', 'false');
  });

  it('give focus back to the row the user came from', async () => {
    const { user } = renderApp();
    await ready();
    await user.click(tile('Codex'));
    await user.click(within(drawer()).getByRole('button', { name: 'Configurar Codex' }));
    await user.click(screen.getByRole('button', { name: 'Voltar' }));
    expect(tile('Codex')).toHaveFocus();
    expect(tile('Codex')).toHaveAttribute('aria-pressed', 'true');
  });

  it('open on the group of the source whose error asked for it', async () => {
    const { user } = renderApp({ scenario: 'errors' });
    await screen.findByRole('button', { name: /^Triagem/ });
    await user.click(tile('Codex'));
    await user.click(await within(drawer()).findByRole('button', { name: 'Abrir configurações' }));
    const group = document.getElementById('grp-codex') as HTMLDetailsElement;
    await vi.waitFor(() => expect(group).toHaveAttribute('open'));
    await vi.waitFor(() => expect(group.querySelector('summary')).toHaveFocus());
  });
});

describe('Escape', () => {
  it('hides the flyout from the surfaces', async () => {
    const { user, backend } = renderApp();
    await ready();
    await user.keyboard('{Escape}');
    expect(backend.calls).toContain('hide_flyout');
  });

  it('closes the drawer first, and hides only once nothing else is open', async () => {
    const { user, backend } = renderApp();
    await ready();
    await user.click(tile('Codex'));
    await user.keyboard('{Escape}');
    expect(screen.queryByRole('dialog')).toBeNull();
    expect(backend.calls).not.toContain('hide_flyout');
    await user.keyboard('{Escape}');
    expect(backend.calls.filter((call) => call === 'hide_flyout')).toHaveLength(1);
  });

  it('goes back from settings first, and hides only on the next press', async () => {
    const { user, backend } = renderApp();
    await ready();
    await user.click(gear());
    await user.keyboard('{Escape}');
    expect(nav('Triagem')).toBeVisible();
    expect(backend.calls).not.toContain('hide_flyout');
    await user.keyboard('{Escape}');
    expect(backend.calls.filter((call) => call === 'hide_flyout')).toHaveLength(1);
  });

  it('says so when the flyout could not be hidden', async () => {
    const { user, backend } = renderApp();
    await ready();
    vi.spyOn(backend, 'hideFlyout').mockRejectedValue(new BackendError(problem('internal')));
    await user.keyboard('{Escape}');
    expect(await screen.findByRole('alert')).toHaveTextContent('Algo deu errado.');
  });

  it('is ignored while a text field is composing', async () => {
    const { user, backend } = renderApp();
    await ready();
    await user.click(gear());
    await act(async () => {
      document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', isComposing: true }));
    });
    expect(screen.getByRole('region', { name: 'Configurações' })).toBeVisible();
    expect(backend.calls).not.toContain('hide_flyout');
  });
});
