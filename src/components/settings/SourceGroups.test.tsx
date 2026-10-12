import { screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { BackendError } from '../../lib/backend';
import { HARNESS_IDS } from '../../lib/contract';
import { problem } from '../../mock/fixtures';
import { renderPanel, settingsWith } from '../../test/helpers';
import { SOURCE_GROUPS } from './SourceGroups';

const Codex = SOURCE_GROUPS.codex;
const Antigravity = SOURCE_GROUPS.antigravity;
const Opencode = SOURCE_GROUPS.opencode;
const Litellm = SOURCE_GROUPS.litellm;

describe('the on/off switch of every group', () => {
  it.each(HARNESS_IDS)('turns %s off and back on, saving each time', async (id) => {
    const Group = SOURCE_GROUPS[id];
    const { user, backend } = renderPanel(<Group />);
    const read = screen.getByRole('switch', { name: 'Ler esta fonte' });
    expect(read).toBeChecked();
    await user.click(read);
    await vi.waitFor(async () => expect((await backend.getSettings())[id].enabled).toBe(false));
    await user.click(read);
    await vi.waitFor(async () => expect((await backend.getSettings())[id].enabled).toBe(true));
  });

  it('leaves the other sources alone', async () => {
    const { user, backend } = renderPanel(<Codex />);
    await user.click(screen.getByRole('switch', { name: 'Ler esta fonte' }));
    await vi.waitFor(async () => expect((await backend.getSettings()).codex.enabled).toBe(false));
    const after = await backend.getSettings();
    expect(
      [after.claude, after.antigravity, after.opencode, after.litellm].map((s) => s.enabled),
    ).toEqual([true, true, true, true]);
  });
});

describe('Codex', () => {
  const binary = () => screen.getByLabelText('Caminho do executável do Codex');
  const save = () => screen.getByRole('button', { name: 'Salvar caminho' });

  it('needs the consent to start its app-server, and saves the change at once', async () => {
    const off = settingsWith((s) => {
      s.codex.consent = false;
    });
    const { user, backend } = renderPanel(<Codex />, { settings: off });
    const consent = screen.getByRole('switch', {
      name: 'Permitir que o monitor inicie o codex app-server',
    });
    expect(consent).not.toBeChecked();
    await user.click(consent);
    await vi.waitFor(async () => expect((await backend.getSettings()).codex.consent).toBe(true));
  });

  it('saves a typed path trimmed, and says so', async () => {
    const { user, backend } = renderPanel(<Codex />);
    await user.type(binary(), '  C:\\tools\\codex.cmd  ');
    await user.click(save());
    expect(await screen.findByText('Salvo.')).toBeVisible();
    expect((await backend.getSettings()).codex.binaryPath).toBe('C:\\tools\\codex.cmd');
  });

  it.each(['', '   '])('turns %j back into "search the PATH"', async (typed) => {
    const stored = settingsWith((s) => {
      s.codex.binaryPath = 'C:\\tools\\codex.cmd';
    });
    const { user, backend } = renderPanel(<Codex />, { settings: stored });
    await vi.waitFor(() => expect(binary()).toHaveValue('C:\\tools\\codex.cmd'));
    await user.clear(binary());
    if (typed) await user.type(binary(), typed);
    await user.click(save());
    await vi.waitFor(async () => expect((await backend.getSettings()).codex.binaryPath).toBeNull());
  });

  it('retracts "Salvo." as soon as the text changes again', async () => {
    const { user } = renderPanel(<Codex />);
    await user.type(binary(), 'codex');
    await user.click(save());
    await screen.findByText('Salvo.');
    await user.type(binary(), '2');
    expect(screen.queryByText('Salvo.')).toBeNull();
  });

  it('prints the backend reason beside the field when the save is refused', async () => {
    const { user, backend } = renderPanel(<Codex />);
    vi.spyOn(backend, 'saveSettings').mockRejectedValue(new BackendError(problem('internal')));
    await user.type(binary(), 'codex');
    await user.click(save());
    expect(await screen.findByText(/Algo deu errado\./)).toBeVisible();
    expect(screen.queryByText('Salvo.')).toBeNull();
  });
});

describe('Antigravity', () => {
  it('says the reading is unofficial and local before it is allowed', async () => {
    const off = settingsWith((s) => {
      s.antigravity.consent = false;
    });
    const { user, backend } = renderPanel(<Antigravity />, { settings: off });
    expect(screen.getByText(/Leitura não oficial/)).toBeVisible();
    expect(screen.getByText(/conversa só com 127\.0\.0\.1/)).toBeVisible();
    const consent = screen.getByRole('switch', {
      name: 'Permitir a leitura do servidor local da IDE',
    });
    expect(consent).not.toBeChecked();
    await user.click(consent);
    await vi.waitFor(async () =>
      expect((await backend.getSettings()).antigravity.consent).toBe(true),
    );
  });
});

describe('OpenCode', () => {
  it('keeps the key, the links, the local estimate and the folder in one place', () => {
    renderPanel(<Opencode />);
    expect(screen.getByLabelText('Chave de API do OpenCode Go')).toHaveAttribute(
      'type',
      'password',
    );
    expect(screen.getByRole('button', { name: /Abrir o console do OpenCode Go/ })).toBeVisible();
    expect(screen.getByRole('button', { name: /Ver cobrança do Zen/ })).toBeVisible();
    expect(screen.getByRole('switch', { name: /Estimativa local/ })).toBeVisible();
    expect(screen.getByLabelText('Pasta de dados do OpenCode')).toBeVisible();
  });

  it('opens the console and the billing page through the backend', async () => {
    const { user, backend } = renderPanel(<Opencode />);
    await user.click(screen.getByRole('button', { name: /Abrir o console do OpenCode Go/ }));
    await user.click(screen.getByRole('button', { name: /Ver cobrança do Zen/ }));
    expect(backend.openedLinks).toEqual(['opencode_go_console', 'opencode_billing']);
  });

  it('says so when a link could not be opened', async () => {
    const { user, backend } = renderPanel(<Opencode />);
    vi.spyOn(backend, 'openLink').mockRejectedValue(new BackendError(problem('internal')));
    await user.click(screen.getByRole('button', { name: /Ver cobrança do Zen/ }));
    expect(await screen.findByRole('alert')).toHaveTextContent('Algo deu errado.');
  });

  it('saves the local estimate consent at once', async () => {
    const off = settingsWith((s) => {
      s.opencode.localConsent = false;
    });
    const { user, backend } = renderPanel(<Opencode />, { settings: off });
    await user.click(screen.getByRole('switch', { name: /Estimativa local/ }));
    await vi.waitFor(async () =>
      expect((await backend.getSettings()).opencode.localConsent).toBe(true),
    );
  });

  it('saves the data folder trimmed, and blank means the default', async () => {
    const { user, backend } = renderPanel(<Opencode />);
    const folder = screen.getByLabelText('Pasta de dados do OpenCode');
    await user.type(folder, ' D:\\dados\\opencode ');
    await user.click(screen.getByRole('button', { name: 'Salvar pasta' }));
    await vi.waitFor(async () =>
      expect((await backend.getSettings()).opencode.dataDir).toBe('D:\\dados\\opencode'),
    );
    await user.clear(folder);
    await user.click(screen.getByRole('button', { name: 'Salvar pasta' }));
    await vi.waitFor(async () => expect((await backend.getSettings()).opencode.dataDir).toBeNull());
  });
});

describe('LiteLLM', () => {
  it('shows the connection form without a second title', async () => {
    renderPanel(<Litellm />);
    expect(await screen.findByLabelText('URL do servidor')).toHaveValue(
      'https://gateway.exemplo.dev',
    );
    expect(screen.queryByRole('heading')).toBeNull();
    expect(screen.getByRole('button', { name: 'Salvar e conectar' })).toBeEnabled();
  });
});
