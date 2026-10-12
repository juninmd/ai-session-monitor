import { screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { BackendError } from '../../lib/backend';
import type { ShimStatus } from '../../lib/contract';
import { problem } from '../../mock/fixtures';
import { renderPanel, settingsWith } from '../../test/helpers';
import { ClaudeGroup } from './ClaudeGroup';

const PROFILE = 'D:\\perfis\\claude';
const folder = () => screen.getByLabelText('Pasta de configuração do Claude Code');
const save = () => screen.getByRole('button', { name: 'Salvar pasta do Claude Code' });
const shimReads = (calls: string[]) => calls.filter((call) => call === 'claude_shim_status');

describe('the Claude Code folder', () => {
  it('saves a typed folder trimmed, says so, and leaves the rest of the section alone', async () => {
    const { user, backend } = renderPanel(<ClaudeGroup />);
    await user.type(folder(), `  ${PROFILE}  `);
    await user.click(save());
    expect(await screen.findByText('Salvo.')).toBeVisible();
    expect((await backend.getSettings()).claude).toMatchObject({
      configDir: PROFILE,
      shimConsent: true,
      oauthUsage: false,
    });
  });

  it.each(['', '   '])(
    'turns %j back into "follow CLAUDE_CONFIG_DIR or ~\\.claude"',
    async (typed) => {
      const stored = settingsWith((s) => {
        s.claude.configDir = PROFILE;
      });
      const { user, backend } = renderPanel(<ClaudeGroup />, { settings: stored });
      await vi.waitFor(() => expect(folder()).toHaveValue(PROFILE));
      await user.clear(folder());
      if (typed) await user.type(folder(), typed);
      await user.click(save());
      await vi.waitFor(async () =>
        expect((await backend.getSettings()).claude.configDir).toBeNull(),
      );
    },
  );

  it('looks at the settings.json of the new folder as soon as it is saved', async () => {
    const before = 'C:\\Users\\exemplo\\.claude\\settings.json';
    const after = `${PROFILE}\\settings.json`;
    const reads = vi
      .fn<() => Promise<ShimStatus>>()
      .mockResolvedValueOnce({ state: 'installed', chained: false, settingsPath: before })
      .mockResolvedValueOnce({ state: 'not_installed', chained: false, settingsPath: after });
    const { user } = renderPanel(<ClaudeGroup />, {
      prepare: (mock) => {
        vi.spyOn(mock, 'claudeShimStatus').mockImplementation(reads);
      },
    });
    expect(await screen.findByText(`Arquivo: ${before}`)).toBeVisible();
    await user.type(folder(), PROFILE);
    await user.click(save());
    expect(await screen.findByText(`Arquivo: ${after}`)).toBeVisible();
    expect(screen.getByText('Não instalado')).toBeVisible();
    expect(screen.queryByText(`Arquivo: ${before}`)).toBeNull();
    expect(reads).toHaveBeenCalledTimes(2);
  });

  it('does not look again when another Claude setting changes', async () => {
    const { user, backend } = renderPanel(<ClaudeGroup />);
    await screen.findByText('Instalado');
    await user.click(screen.getByRole('switch', { name: 'API de uso da conta' }));
    await user.click(await screen.findByRole('button', { name: 'Ativar mesmo assim' }));
    await vi.waitFor(async () =>
      expect((await backend.getSettings()).claude.oauthUsage).toBe(true),
    );
    expect(shimReads(backend.calls)).toHaveLength(1);
  });

  it('prints the backend reason beside the field when the folder is refused', async () => {
    const share = '\\\\servidor\\compartilhado\\claude';
    const { user, backend } = renderPanel(<ClaudeGroup />);
    vi.spyOn(backend, 'saveSettings').mockRejectedValue(
      new BackendError(problem('invalid_input', { field: 'configDir' })),
    );
    await user.type(folder(), share);
    await user.click(save());
    expect(
      await screen.findByText(/Campo com problema: pasta de configuração do Claude Code/),
    ).toBeVisible();
    expect(screen.queryByText('Salvo.')).toBeNull();
    expect(folder()).toHaveValue(share);
    expect(folder()).toHaveAttribute('aria-invalid', 'true');
  });
});
