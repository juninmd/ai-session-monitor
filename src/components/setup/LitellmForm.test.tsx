import { screen } from '@testing-library/react';
import type { UserEvent } from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { BackendError } from '../../lib/backend';
import { problem } from '../../mock/fixtures';
import { renderPanel } from '../../test/helpers';
import { LitellmForm } from './LitellmForm';

const SECRET = 'fab-litellm-key';
const URL_OK = 'https://gateway.exemplo.dev';
const url = () => screen.getByLabelText('URL do servidor');
const key = () => screen.getByLabelText('Chave de API');
const submit = () => screen.getByRole('button', { name: 'Salvar e conectar' });

async function connect(user: UserEvent, address = URL_OK, secret = SECRET) {
  if (address) await user.type(url(), address);
  if (secret) await user.type(key(), secret);
}

const setup = (scenario: 'fresh' | 'healthy' = 'fresh') =>
  renderPanel(<LitellmForm />, { scenario });

describe('validating before sending', () => {
  it('asks for the URL and the key, and puts the cursor on the first problem', async () => {
    const { user, backend } = setup();
    await user.click(submit());
    expect(screen.getByText('Informe a URL do servidor.')).toBeVisible();
    expect(screen.getByText('Cole a chave de API do LiteLLM.')).toBeVisible();
    expect(url()).toHaveAttribute('aria-invalid', 'true');
    expect(url()).toHaveFocus();
    expect(backend.calls).not.toContain('save_settings');
    expect(backend.calls).not.toContain('set_secret');
  });

  it('checks the URL when the user leaves the field', async () => {
    const { user } = setup();
    await user.type(url(), 'http://gateway.exemplo.dev');
    await user.tab();
    expect(screen.getByText('Use https://. HTTP só funciona para localhost.')).toBeVisible();
    await user.clear(url());
    await user.type(url(), URL_OK);
    await user.tab();
    expect(screen.queryByText(/HTTP só funciona/)).toBeNull();
  });

  it('accepts plain HTTP for loopback only', async () => {
    const { user } = setup();
    await user.type(url(), 'http://127.0.0.1:4000');
    await user.tab();
    expect(url()).not.toHaveAttribute('aria-invalid');
  });

  it('asks for an id when measuring a team or a user, and not for a key', async () => {
    const { user } = setup();
    expect(screen.queryByLabelText('ID do time')).toBeNull();
    await user.click(screen.getByRole('radio', { name: 'Time' }));
    await connect(user);
    await user.click(submit());
    expect(screen.getByText('Informe o ID.')).toBeVisible();
    expect(screen.getByLabelText('ID do time')).toHaveFocus();
    await user.click(screen.getByRole('radio', { name: 'Esta chave' }));
    expect(screen.queryByLabelText('ID do time')).toBeNull();
  });

  it('rejects a header name with spaces, inside the advanced section', async () => {
    const { user } = setup();
    await connect(user);
    await user.clear(screen.getByLabelText('Cabeçalho de autenticação'));
    await user.type(screen.getByLabelText('Cabeçalho de autenticação'), 'Bad Header');
    await user.click(submit());
    expect(screen.getByText('Use só letras, números e hífen.')).toBeVisible();
    expect(screen.getByLabelText('Cabeçalho de autenticação')).toHaveFocus();
    expect(screen.getByText('Avançado').closest('details')).toHaveAttribute('open');
  });
});

describe('saving', () => {
  it('stores the settings, then the key, then reads once, and forgets the key', async () => {
    const { user, backend } = setup();
    const settingsSave = vi.spyOn(backend, 'saveSettings');
    await connect(user);
    await user.click(submit());
    expect(await screen.findByText('Configuração salva. Lendo o servidor…')).toBeVisible();

    const order = backend.calls.filter((c) => /^(save_settings|set_secret|refresh)$/.test(c));
    expect(order).toEqual(['save_settings', 'set_secret', 'refresh']);
    expect((await backend.getSettings()).litellm.baseUrl).toBe(URL_OK);
    expect((await backend.secretStatus()).litellm).toBe(true);
    expect(key()).toHaveValue('');
    expect(document.body.innerHTML).not.toContain(SECRET);
    expect(JSON.stringify(settingsSave.mock.calls)).not.toContain(SECRET);
    expect(screen.getByText('Chave salva')).toBeVisible();
  });

  it('keeps a saved key when none is typed', async () => {
    const { user, backend } = setup('healthy');
    const setKey = vi.spyOn(backend, 'setSecret');
    await vi.waitFor(() => expect(url()).toHaveValue(URL_OK));
    await user.click(submit());
    expect(await screen.findByText('Configuração salva. Lendo o servidor…')).toBeVisible();
    expect(setKey).not.toHaveBeenCalled();
  });

  it('sends what the user chose: scope, id, header and trimmed text', async () => {
    const { user, backend } = setup();
    await connect(user, `  ${URL_OK}  `);
    await user.click(screen.getByRole('radio', { name: 'Usuário' }));
    await user.type(screen.getByLabelText('ID do usuário'), ' u-42 ');
    await user.clear(screen.getByLabelText('Cabeçalho de autenticação'));
    await user.type(screen.getByLabelText('Cabeçalho de autenticação'), 'X-Api-Key');
    await user.click(submit());
    await screen.findByText('Configuração salva. Lendo o servidor…');
    expect((await backend.getSettings()).litellm).toMatchObject({
      baseUrl: URL_OK,
      scope: 'user',
      entityId: 'u-42',
      authHeader: 'X-Api-Key',
    });
  });

  it('offers only the intervals the backend accepts', () => {
    setup();
    const values = (label: string) =>
      Array.from(screen.getByLabelText(label).querySelectorAll('option')).map((o) => o.value);
    expect(values('Atualizar a cada')[0]).toBe('60');
    expect(values('Atualizar a cada').at(-1)).toBe('3600');
    expect(values('Tempo limite')[0]).toBe('1');
    expect(values('Tempo limite').at(-1)).toBe('30');
  });

  it('disables the button while saving, so a second click cannot repeat it', async () => {
    const { user, backend } = setup();
    let release: () => void = () => {};
    const slow = vi.spyOn(backend, 'refresh').mockImplementation(
      () =>
        new Promise<void>((done) => {
          release = done;
        }),
    );
    await connect(user);
    await user.click(submit());
    const busy = await screen.findByRole('button', { name: 'Salvando…' });
    expect(busy).toBeDisabled();
    release();
    await screen.findByRole('button', { name: 'Salvar e conectar' });
    expect(slow).toHaveBeenCalledTimes(1);
  });

  it('shows what changed elsewhere instead of a stale draft', async () => {
    const { backend } = setup('healthy');
    await vi.waitFor(() => expect(url()).toHaveValue(URL_OK));
    const next = await backend.getSettings();
    next.litellm.baseUrl = 'https://outro.exemplo.dev';
    await backend.saveSettings(next);
    await vi.waitFor(() => expect(url()).toHaveValue('https://outro.exemplo.dev'));
  });
});

describe('errors from the backend', () => {
  it('print a rejected URL next to the URL field', async () => {
    const { user, backend } = setup();
    vi.spyOn(backend, 'saveSettings').mockRejectedValue(
      new BackendError(problem('invalid_url', { field: 'baseUrl' })),
    );
    await connect(user);
    await user.click(submit());
    expect(
      await screen.findByText(/Use https:\/\/\. HTTP só funciona para localhost, e a URL/),
    ).toBeVisible();
    expect(url()).toHaveAttribute('aria-invalid', 'true');
    expect(url()).toHaveFocus();
  });

  it('open the advanced section when the problem is inside it', async () => {
    const { user, backend } = setup();
    vi.spyOn(backend, 'saveSettings').mockRejectedValue(
      new BackendError(problem('invalid_input', { field: 'authHeader' })),
    );
    await connect(user);
    await user.click(submit());
    expect(await screen.findByText('Use só letras, números e hífen.')).toBeVisible();
    expect(screen.getByText('Avançado').closest('details')).toHaveAttribute('open');
  });

  it('print a rejected key next to the key field', async () => {
    const { user, backend } = setup();
    vi.spyOn(backend, 'setSecret').mockRejectedValue(
      new BackendError(problem('invalid_input', { field: 'value' })),
    );
    await connect(user);
    await user.click(submit());
    expect(await screen.findByText('Cole a chave de API.')).toBeVisible();
    expect(key()).toHaveAttribute('aria-invalid', 'true');
    expect(document.body).not.toHaveTextContent(SECRET);
  });

  it('print an error that names no field at the bottom of the form', async () => {
    const { user, backend } = setup();
    vi.spyOn(backend, 'saveSettings').mockRejectedValue(
      new BackendError(problem('settings_unreadable')),
    );
    await connect(user);
    await user.click(submit());
    expect(await screen.findByText(/Arquivo de configurações ilegível\./)).toBeVisible();
    // No field to move to, so the role is the only cue a screen reader gets.
    expect(screen.getByRole('alert')).toHaveTextContent(/Arquivo de configurações ilegível\./);
  });

  it('do not trust a field name that is not one of ours', async () => {
    const { user, backend } = setup();
    vi.spyOn(backend, 'saveSettings').mockRejectedValue(
      new BackendError(problem('invalid_input', { field: 'constructor' })),
    );
    await connect(user);
    await user.click(submit());
    expect(await screen.findByText(/Valor inválido\./)).toBeVisible();
  });
});
