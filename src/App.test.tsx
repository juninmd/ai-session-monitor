import { act, screen, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
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
  'LiteLLM',
];
const tab = (name: string) => screen.getByRole('tab', { name: new RegExp(`^${name}\\b`) });
const ready = () => screen.findByRole('heading', { name: 'Sessão de 5 horas' });
const gear = () => screen.getByRole('button', { name: 'Configurações' });

describe('the shell', () => {
  it('shows the title, the nine tools in order and the first panel', async () => {
    renderApp();
    expect(screen.getByRole('heading', { level: 1, name: 'AI Session Monitor' })).toBeVisible();
    const tabs = screen.getAllByRole('tab');
    expect(tabs).toHaveLength(NAMES.length);
    for (const [index, name] of NAMES.entries()) {
      expect(tabs[index]).toHaveAccessibleName(new RegExp(`^${name}\\b`));
    }
    expect(tabs[0]).toHaveAttribute('aria-selected', 'true');
    expect(await ready()).toBeVisible();
    expect(screen.getByRole('tabpanel')).toHaveAccessibleName('Claude Code');
  });

  it('summarises every tool in its tab, in words as well as numbers', async () => {
    renderApp();
    await ready();
    expect(tab('Claude Code')).toHaveAccessibleName('Claude Code 52% disponível, dentro do ritmo');
    expect(tab('Codex')).toHaveAccessibleName('Codex 18% disponível, atenção');
    expect(tab('Antigravity')).toHaveAccessibleName('Antigravity 46% disponível, dentro do ritmo');
    expect(tab('OpenCode')).toHaveAccessibleName('OpenCode 79% disponível, dentro do ritmo');
    expect(tab('LiteLLM')).toHaveAccessibleName('LiteLLM falha na leitura');
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
    expect(screen.getByText('Ainda sem leitura')).toBeVisible();
  });

  it('shows the age of the newest reading, from the clock it was given', async () => {
    renderApp();
    await ready();
    expect(screen.getByText('Lido há 15 s')).toBeVisible();
  });
});

describe('moving between tools', () => {
  it('shows the panel of the tab that was clicked, and names it after the tab', async () => {
    const { user } = renderApp();
    await ready();
    await user.click(tab('Codex'));
    expect(tab('Codex')).toHaveAttribute('aria-selected', 'true');
    expect(tab('Claude Code')).toHaveAttribute('aria-selected', 'false');
    expect(screen.getByRole('tabpanel')).toHaveAccessibleName('Codex');
    expect(await screen.findByRole('heading', { name: 'Janela de 5 horas' })).toBeVisible();
    expect(screen.queryByRole('heading', { name: 'Sessão de 5 horas' })).toBeNull();
  });

  it('follows the arrow keys', async () => {
    const { user } = renderApp();
    await ready();
    tab('Claude Code').focus();
    await user.keyboard('{ArrowRight}');
    expect(tab('Codex')).toHaveFocus();
    expect(screen.getByRole('tabpanel')).toHaveAccessibleName('Codex');
    await user.keyboard('{End}');
    expect(tab('LiteLLM')).toHaveFocus();
    expect(screen.getByRole('tabpanel')).toHaveAccessibleName('LiteLLM');
  });

  it('starts every panel at the top', async () => {
    const { user } = renderApp();
    await ready();
    const body = document.querySelector('.fly-body') as HTMLElement;
    body.scrollTop = 160;
    expect(body.scrollTop).toBe(160);
    await user.click(tab('Antigravity'));
    expect(body.scrollTop).toBe(0);
  });

  it('shows the failure of a tool in its own panel, with one way forward', async () => {
    const { user } = renderApp();
    await ready();
    await user.click(tab('LiteLLM'));
    const panel = screen.getByRole('tabpanel');
    expect(await within(panel).findByRole('heading', { level: 2 })).toHaveTextContent(
      'Chave ou login recusado',
    );
    expect(within(panel).getAllByRole('button')).toHaveLength(1);
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
});

describe('settings', () => {
  it('open from the gear, replace the tabs, and close from the same button', async () => {
    const { user } = renderApp();
    await ready();
    expect(gear()).toHaveAttribute('aria-pressed', 'false');
    await user.click(gear());
    expect(gear()).toHaveAttribute('aria-pressed', 'true');
    expect(screen.queryByRole('tablist')).toBeNull();
    expect(screen.getByRole('region', { name: 'Configurações' })).toBeVisible();
    expect(screen.getByRole('heading', { level: 2, name: 'Configurações' })).toHaveFocus();
    await user.click(gear());
    expect(screen.getByRole('tablist')).toBeVisible();
    expect(gear()).toHaveAttribute('aria-pressed', 'false');
  });

  it('give focus back to the tab the user came from', async () => {
    const { user } = renderApp();
    await ready();
    await user.click(tab('Codex'));
    await user.click(gear());
    await user.click(screen.getByRole('button', { name: 'Voltar' }));
    expect(tab('Codex')).toHaveFocus();
    expect(tab('Codex')).toHaveAttribute('aria-selected', 'true');
  });

  it('open on the group of the source whose error asked for it', async () => {
    const { user } = renderApp({ scenario: 'errors' });
    await screen.findByRole('heading', { name: 'Falha ao instalar' });
    await user.click(tab('Codex'));
    await user.click(await screen.findByRole('button', { name: 'Abrir configurações' }));
    const group = document.getElementById('grp-codex') as HTMLDetailsElement;
    await vi.waitFor(() => expect(group).toHaveAttribute('open'));
    await vi.waitFor(() => expect(group.querySelector('summary')).toHaveFocus());
  });
});

describe('Escape', () => {
  it('hides the flyout from the tabs', async () => {
    const { user, backend } = renderApp();
    await ready();
    await user.keyboard('{Escape}');
    expect(backend.calls).toContain('hide_flyout');
  });

  it('goes back from settings first, and hides only on the next press', async () => {
    const { user, backend } = renderApp();
    await ready();
    await user.click(gear());
    await user.keyboard('{Escape}');
    expect(screen.getByRole('tablist')).toBeVisible();
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
