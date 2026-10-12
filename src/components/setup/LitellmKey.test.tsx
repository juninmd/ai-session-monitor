// The saved LiteLLM key belongs to the server it was entered for: the form stops offering to keep
// it once the draft points somewhere else, and shows what the backend did to it after a save.
import { screen } from '@testing-library/react';
import type { UserEvent } from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { BackendError } from '../../lib/backend';
import type { MockBackend } from '../../mock';
import { problem } from '../../mock/fixtures';
import { renderPanel } from '../../test/helpers';
import { LitellmForm } from './LitellmForm';

const SECRET = 'fab-litellm-key';
const URL_OK = 'https://gateway.exemplo.dev';
const ELSEWHERE = 'https://outro.exemplo.dev';
const KEEP = 'Deixe em branco para manter a chave salva';
const NEEDS_KEY = 'Cole a chave de API do LiteLLM.';
const REBIND = /vale só para a conexão atual/;
const SAVED = 'Configuração salva. Lendo o servidor…';

const url = () => screen.getByLabelText('URL do servidor');
const key = () => screen.getByLabelText('Chave de API');
const header = () => screen.getByLabelText('Cabeçalho de autenticação');
const submit = () => screen.getByRole('button', { name: 'Salvar e conectar' });

/** A connected setup: URL_OK with its key already stored. */
async function connected() {
  const harness = renderPanel(<LitellmForm />, { scenario: 'healthy' });
  await screen.findByText('Chave salva');
  await vi.waitFor(() => expect(url()).toHaveValue(URL_OK));
  return harness;
}

async function retype(user: UserEvent, address: string) {
  await user.clear(url());
  await user.type(url(), address);
}

/** What the real backend does to the key when a save moves it; the mock leaves it alone. */
function droppingTheKey(backend: MockBackend) {
  const original = backend.saveSettings;
  vi.spyOn(backend, 'saveSettings').mockImplementation(async (next) => {
    const saved = await original(next);
    await backend.clearSecret('litellm');
    return saved;
  });
}

describe('the saved key and the server it was entered for', () => {
  it('offers to keep the key while the server stays the same', async () => {
    await connected();
    expect(key()).toHaveAttribute('placeholder', KEEP);
    expect(screen.queryByText(REBIND)).toBeNull();
  });

  it('stops offering to keep it once the server changes, and says why', async () => {
    const { user } = await connected();
    await retype(user, ELSEWHERE);
    expect(key()).not.toHaveAttribute('placeholder', KEEP);
    expect(screen.getByText(REBIND)).toBeVisible();
  });

  it('offers it again when the draft goes back to the saved server', async () => {
    const { user } = await connected();
    await retype(user, ELSEWHERE);
    await retype(user, URL_OK);
    expect(key()).toHaveAttribute('placeholder', KEEP);
    expect(screen.queryByText(REBIND)).toBeNull();
  });

  it.each([
    ['another host', ELSEWHERE],
    ['another port', `${URL_OK}:8443`],
    ['another path', `${URL_OK}/proxy`],
    ['plain http to the same machine', 'http://localhost:4000'],
  ])('asks for the key again for %s, and sends nothing without it', async (_, address) => {
    const { user, backend } = await connected();
    await retype(user, address);
    await user.click(submit());
    expect(screen.getByText(NEEDS_KEY)).toBeVisible();
    expect(key()).toHaveAttribute('aria-invalid', 'true');
    expect(key()).toHaveFocus();
    expect(backend.calls).not.toContain('save_settings');
    expect(backend.calls).not.toContain('set_secret');
  });

  it('asks for the key again when the header that carries it changes', async () => {
    const { user, backend } = await connected();
    await user.clear(header());
    await user.type(header(), 'X-Api-Key');
    await user.click(submit());
    expect(screen.getByText(NEEDS_KEY)).toBeVisible();
    expect(backend.calls).not.toContain('save_settings');
  });

  it('sends the new server first and the key typed for it after', async () => {
    const { user, backend } = await connected();
    await retype(user, ELSEWHERE);
    await user.type(key(), SECRET);
    await user.click(submit());
    expect(await screen.findByText(SAVED)).toBeVisible();

    const order = backend.calls.filter((c) => /^(save_settings|set_secret|refresh)$/.test(c));
    expect(order).toEqual(['save_settings', 'set_secret', 'refresh']);
    expect((await backend.getSettings()).litellm.baseUrl).toBe(ELSEWHERE);
  });

  it.each([
    ['an upper-case host', 'https://GATEWAY.exemplo.dev'],
    ['a trailing slash', `${URL_OK}/`],
    ['the default port spelled out', `${URL_OK}:443`],
  ])('keeps the key when only the spelling changes: %s', async (_, address) => {
    const { user, backend } = await connected();
    const setKey = vi.spyOn(backend, 'setSecret');
    await retype(user, address);
    expect(key()).toHaveAttribute('placeholder', KEEP);
    await user.click(submit());
    expect(await screen.findByText(SAVED)).toBeVisible();
    expect(setKey).not.toHaveBeenCalled();
  });

  it('keeps the key when only the interval, the scope or the id changes', async () => {
    const { user, backend } = await connected();
    const setKey = vi.spyOn(backend, 'setSecret');
    await user.selectOptions(screen.getByLabelText('Atualizar a cada'), '600');
    await user.click(screen.getByRole('radio', { name: 'Usuário' }));
    await user.type(screen.getByLabelText('ID do usuário'), 'u-42');
    expect(key()).toHaveAttribute('placeholder', KEEP);
    await user.click(submit());
    expect(await screen.findByText(SAVED)).toBeVisible();
    expect(setKey).not.toHaveBeenCalled();
    expect((await backend.getSettings()).litellm).toMatchObject({
      pollIntervalSecs: 600,
      scope: 'user',
      entityId: 'u-42',
    });
  });
});

describe('after a save that moved the key', () => {
  it('shows that the key is gone when it could not be stored again', async () => {
    const { user, backend } = await connected();
    droppingTheKey(backend);
    vi.spyOn(backend, 'setSecret').mockRejectedValue(
      new BackendError(problem('internal', { where: 'secret_store' })),
    );
    await retype(user, ELSEWHERE);
    await user.type(key(), SECRET);
    await user.click(submit());

    expect(await screen.findByText('Sem chave salva')).toBeVisible();
    expect(screen.queryByText('Chave salva')).toBeNull();
    expect(document.body).not.toHaveTextContent(SECRET);
  });
});
