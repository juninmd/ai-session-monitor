import { screen, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { BackendError } from '../../lib/backend';
import type { ShimState, ShimStatus } from '../../lib/contract';
import { problem } from '../../mock/fixtures';
import { type PanelOptions, renderPanel, settingsWith } from '../../test/helpers';
import { ClaudeGroup } from './ClaudeGroup';

const PATH = 'C:\\Users\\exemplo\\.claude\\settings.json';
const shimAs =
  (state: ShimState, extra: Partial<ShimStatus> = {}): PanelOptions['prepare'] =>
  (backend) => {
    vi.spyOn(backend, 'claudeShimStatus').mockResolvedValue({
      state,
      chained: false,
      settingsPath: PATH,
      ...extra,
    });
  };

const button = (name: string) => screen.queryByRole('button', { name });
const callsTo = (calls: string[], ...names: string[]) => calls.filter((c) => names.includes(c));

describe('the status line command', () => {
  it('looks at the real file once when the group opens', async () => {
    const { backend } = renderPanel(<ClaudeGroup />);
    expect(screen.getByText('Verificando o Claude Code')).toBeVisible();
    expect(await screen.findByText('Instalado')).toBeVisible();
    expect(screen.getByText(`Arquivo: ${PATH}`)).toBeVisible();
    expect(callsTo(backend.calls, 'claude_shim_status')).toHaveLength(1);
  });

  it.each<[ShimState, boolean, string, string[], string[]]>([
    ['not_installed', false, 'Não instalado', ['Instalar comando'], ['Remover comando']],
    ['installed', false, 'Instalado', ['Remover comando'], ['Instalar comando']],
    [
      'installed',
      true,
      'Instalado, encadeado com a sua barra de status',
      ['Remover comando'],
      ['Instalar comando'],
    ],
    ['foreign', false, 'Outra barra de status em uso', ['Instalar comando'], ['Remover comando']],
    [
      'outdated',
      false,
      'Desatualizado: o executável não foi encontrado',
      ['Reinstalar comando', 'Remover comando'],
      ['Instalar comando'],
    ],
    ['unreadable', false, 'settings.json ilegível', [], ['Instalar comando', 'Remover comando']],
  ])(
    '%s (chained=%s) reads "%s" and offers only what makes sense',
    async (state, chained, chip, offered, absent) => {
      renderPanel(<ClaudeGroup />, { prepare: shimAs(state, { chained }) });
      expect(await screen.findByText(chip)).toBeVisible();
      for (const name of offered) expect(button(name)).toBeEnabled();
      for (const name of absent) expect(button(name)).toBeNull();
      expect(button('Verificar de novo')).toBeEnabled();
    },
  );

  it('explains a status line of the user that it will keep working', async () => {
    renderPanel(<ClaudeGroup />, { prepare: shimAs('foreign') });
    expect(
      await screen.findByText(/a sua barra de status atual continua funcionando/),
    ).toBeVisible();
  });

  it('refuses to touch a file it cannot read, and says so', async () => {
    renderPanel(<ClaudeGroup />, { prepare: shimAs('unreadable') });
    expect(await screen.findByText(/não mexe num arquivo que não consegue ler/)).toBeVisible();
  });

  it('does not invent a path the backend did not report', async () => {
    renderPanel(<ClaudeGroup />, { prepare: shimAs('installed', { settingsPath: null }) });
    await screen.findByText('Instalado');
    expect(screen.queryByText(/^Arquivo:/)).toBeNull();
  });

  it('installs after saving the consent, reads once and flips the chip', async () => {
    const { user, backend } = renderPanel(<ClaudeGroup />, { scenario: 'fresh' });
    await user.click(await screen.findByRole('button', { name: 'Instalar comando' }));
    expect(await screen.findByText('Instalado')).toBeVisible();
    expect(callsTo(backend.calls, 'save_settings', 'install_claude_shim', 'refresh')).toEqual([
      'save_settings',
      'install_claude_shim',
      'refresh',
    ]);
    expect((await backend.getSettings()).claude.shimConsent).toBe(true);
  });

  it('removes the command, withdraws the consent and reads once', async () => {
    const { user, backend } = renderPanel(<ClaudeGroup />);
    await user.click(await screen.findByRole('button', { name: 'Remover comando' }));
    expect(await screen.findByText('Não instalado')).toBeVisible();
    expect(callsTo(backend.calls, 'uninstall_claude_shim', 'save_settings', 'refresh')).toEqual([
      'uninstall_claude_shim',
      'save_settings',
      'refresh',
    ]);
    expect((await backend.getSettings()).claude.shimConsent).toBe(false);
  });

  it('looks again on request', async () => {
    const { user, backend } = renderPanel(<ClaudeGroup />);
    await user.click(await screen.findByRole('button', { name: 'Verificar de novo' }));
    await vi.waitFor(() => expect(callsTo(backend.calls, 'claude_shim_status')).toHaveLength(2));
  });

  it('reports a failed install in words and lets the user try again', async () => {
    const { user, backend } = renderPanel(<ClaudeGroup />, { scenario: 'fresh' });
    vi.spyOn(backend, 'installClaudeShim').mockRejectedValue(
      new BackendError(problem('install_failed')),
    );
    const install = await screen.findByRole('button', { name: 'Instalar comando' });
    await user.click(install);
    expect(await screen.findByRole('alert')).toHaveTextContent('Falha ao instalar.');
    expect(install).toBeEnabled();
  });

  it('shows a failed look at the file instead of a blank group', async () => {
    renderPanel(<ClaudeGroup />, {
      prepare: (backend) => {
        vi.spyOn(backend, 'claudeShimStatus').mockRejectedValue(
          new BackendError(problem('internal')),
        );
      },
    });
    expect(await screen.findByRole('alert')).toHaveTextContent('Algo deu errado.');
    expect(button('Verificar de novo')).toBeEnabled();
    expect(button('Instalar comando')).toBeNull();
  });
});

describe('the usage API opt-in', () => {
  const toggle = () =>
    screen.getByRole('switch', { name: 'Limites por modelo (Opus, Sonnet, Fable)' });

  it('is off, labelled unofficial and explained before it is touched', () => {
    renderPanel(<ClaudeGroup />);
    expect(toggle()).not.toBeChecked();
    expect(screen.getByText('não oficial')).toBeVisible();
    expect(
      screen.getByText(/Roda claude -p "\/usage" \(custo zero, usa o login do próprio Claude Code/),
    ).toBeVisible();
    expect(screen.queryByRole('alert')).toBeNull();
  });

  it('warns first and saves nothing until the user confirms', async () => {
    const { user, backend } = renderPanel(<ClaudeGroup />);
    await user.click(toggle());
    const warning = await screen.findByRole('alert');
    expect(warning).toHaveTextContent(/a conta pode ser limitada ou bloqueada/);
    expect(within(warning).getByRole('button', { name: 'Ativar mesmo assim' })).toBeEnabled();
    expect(within(warning).getByRole('button', { name: 'Cancelar' })).toBeEnabled();
    expect((await backend.getSettings()).claude.oauthUsage).toBe(false);
    expect(callsTo(backend.calls, 'save_settings')).toEqual([]);
  });

  it('stays off when the user cancels', async () => {
    const { user, backend } = renderPanel(<ClaudeGroup />);
    await user.click(toggle());
    await user.click(await screen.findByRole('button', { name: 'Cancelar' }));
    expect(screen.queryByRole('alert')).toBeNull();
    expect(toggle()).not.toBeChecked();
    expect((await backend.getSettings()).claude.oauthUsage).toBe(false);
  });

  it('turns on after the confirmation, and the warning goes away', async () => {
    const { user, backend } = renderPanel(<ClaudeGroup />);
    await user.click(toggle());
    await user.click(await screen.findByRole('button', { name: 'Ativar mesmo assim' }));
    await vi.waitFor(() => expect(toggle()).toBeChecked());
    expect(screen.queryByRole('alert')).toBeNull();
    expect((await backend.getSettings()).claude.oauthUsage).toBe(true);
  });

  it('puts the keyboard back on the switch when the warning is dismissed', async () => {
    const { user } = renderPanel(<ClaudeGroup />);
    await user.click(toggle());
    await user.click(await screen.findByRole('button', { name: 'Cancelar' }));
    expect(screen.queryByRole('alert')).toBeNull();
    expect(toggle()).toHaveFocus();
  });

  it('puts the keyboard back on the switch once the opt-in is confirmed', async () => {
    const { user } = renderPanel(<ClaudeGroup />);
    await user.click(toggle());
    await user.click(await screen.findByRole('button', { name: 'Ativar mesmo assim' }));
    await vi.waitFor(() => expect(toggle()).toBeChecked());
    expect(screen.queryByRole('alert')).toBeNull();
    expect(toggle()).toHaveFocus();
  });

  it('turns off in one step, with no warning', async () => {
    const on = settingsWith((s) => {
      s.claude.oauthUsage = true;
    });
    const { user, backend } = renderPanel(<ClaudeGroup />, { settings: on });
    await vi.waitFor(() => expect(toggle()).toBeChecked());
    await user.click(toggle());
    await vi.waitFor(() => expect(toggle()).not.toBeChecked());
    expect(screen.queryByRole('alert')).toBeNull();
    expect((await backend.getSettings()).claude.oauthUsage).toBe(false);
  });

  it('keeps the warning up and says why when the save is refused', async () => {
    const { user, backend } = renderPanel(<ClaudeGroup />);
    vi.spyOn(backend, 'saveSettings').mockRejectedValue(new BackendError(problem('internal')));
    await user.click(toggle());
    await user.click(await screen.findByRole('button', { name: 'Ativar mesmo assim' }));
    expect(await screen.findByText(/Algo deu errado\./)).toBeVisible();
    expect(toggle()).not.toBeChecked();
    expect(screen.getByRole('button', { name: 'Ativar mesmo assim' })).toBeEnabled();
  });
});
