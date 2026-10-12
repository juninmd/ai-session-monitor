import { screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { BackendError } from '../../lib/backend';
import { problem } from '../../mock/fixtures';
import { renderPanel } from '../../test/helpers';
import { OpencodeKey } from '../setup/OpencodeParts';

const SECRET = 'fab-key-1';
const field = () => screen.getByLabelText('Chave de API do OpenCode Go');

function watchConsole() {
  const spies = (['log', 'info', 'warn', 'error', 'debug'] as const).map((method) =>
    vi.spyOn(console, method).mockImplementation(() => {}),
  );
  return () =>
    spies
      .flatMap((spy) => spy.mock.calls)
      .flat()
      .join(' ');
}

describe('a secret field', () => {
  it('is a write-only password input that browsers are told not to remember', () => {
    renderPanel(<OpencodeKey />, { scenario: 'fresh' });
    expect(field()).toHaveAttribute('type', 'password');
    expect(field()).toHaveAttribute('autocomplete', 'off');
    expect(field()).toHaveAttribute('autocapitalize', 'off');
    expect(field()).toHaveAttribute('spellcheck', 'false');
    expect(field()).toHaveValue('');
  });

  it('offers no way to reveal what is stored', () => {
    renderPanel(<OpencodeKey />, { scenario: 'healthy' });
    expect(screen.queryByRole('button', { name: /mostrar|revelar|ver chave/i })).toBeNull();
    expect(field()).toHaveValue('');
  });

  it('says whether a key is saved, and only that', async () => {
    renderPanel(<OpencodeKey />, { scenario: 'fresh' });
    expect(screen.getByText('Sem chave salva')).toBeVisible();
    expect(screen.queryByRole('button', { name: 'Remover chave' })).toBeNull();
  });

  it('shows a saved key as a chip and an empty input', async () => {
    renderPanel(<OpencodeKey />, { scenario: 'healthy' });
    expect(await screen.findByText('Chave salva')).toBeVisible();
    expect(field()).toHaveAttribute('placeholder', 'Deixe em branco para manter a chave salva');
    expect(field()).toHaveValue('');
  });
});

describe('saving a secret', () => {
  it('sends it once, trimmed, then forgets it everywhere on screen', async () => {
    const { user, backend } = renderPanel(<OpencodeKey />, { scenario: 'fresh' });
    const logged = watchConsole();
    const send = vi.spyOn(backend, 'setSecret');
    await user.type(field(), `  ${SECRET}  `);
    await user.click(screen.getByRole('button', { name: 'Salvar chave' }));

    expect(send).toHaveBeenCalledExactlyOnceWith('opencode', SECRET);
    expect(await screen.findByText('Chave salva.')).toBeVisible();
    expect(field()).toHaveValue('');
    expect(document.body.innerHTML).not.toContain(SECRET);
    expect(logged()).not.toContain(SECRET);
    expect(JSON.stringify(await backend.getSettings())).not.toContain(SECRET);
    expect(JSON.stringify(await backend.getSnapshot())).not.toContain(SECRET);
  });

  it('flips the chip and offers removal once saved', async () => {
    const { user } = renderPanel(<OpencodeKey />, { scenario: 'fresh' });
    await user.type(field(), SECRET);
    await user.click(screen.getByRole('button', { name: 'Salvar chave' }));
    expect(await screen.findByText('Chave salva')).toBeVisible();
    expect(screen.getByRole('button', { name: 'Remover chave' })).toBeVisible();
  });

  it.each(['', '   '])('refuses %j before asking the backend', async (typed) => {
    const { user, backend } = renderPanel(<OpencodeKey />, { scenario: 'fresh' });
    const send = vi.spyOn(backend, 'setSecret');
    if (typed) await user.type(field(), typed);
    await user.click(screen.getByRole('button', { name: 'Salvar chave' }));
    expect(send).not.toHaveBeenCalled();
    expect(field()).toHaveAttribute('aria-invalid', 'true');
    const message = screen.getByText('Cole a chave de API.');
    expect(field().getAttribute('aria-describedby')).toContain(
      message.closest('p')?.id ?? 'missing',
    );
    // Focus stays on the Save button, so only the role gets the complaint to a screen reader.
    expect(screen.getByRole('alert')).toHaveTextContent('Cole a chave de API.');
  });

  it('clears the complaint as soon as the user types', async () => {
    const { user } = renderPanel(<OpencodeKey />, { scenario: 'fresh' });
    await user.click(screen.getByRole('button', { name: 'Salvar chave' }));
    await user.type(field(), 'x');
    expect(screen.queryByText('Cole a chave de API.')).toBeNull();
    expect(field()).not.toHaveAttribute('aria-invalid');
  });

  it('shows the backend reason next to the field, without echoing what was typed', async () => {
    const { user, backend } = renderPanel(<OpencodeKey />, { scenario: 'fresh' });
    const logged = watchConsole();
    vi.spyOn(backend, 'setSecret').mockRejectedValue(
      new BackendError(problem('invalid_input', { field: 'value', echo: SECRET })),
    );
    await user.type(field(), SECRET);
    await user.click(screen.getByRole('button', { name: 'Salvar chave' }));
    expect(await screen.findByText('Cole a chave de API.')).toBeVisible();
    expect(field()).toHaveAttribute('aria-invalid', 'true');
    expect(document.body).not.toHaveTextContent(SECRET);
    expect(logged()).not.toContain(SECRET);
  });

  it('keeps an unexpected failure generic', async () => {
    const { user, backend } = renderPanel(<OpencodeKey />, { scenario: 'fresh' });
    vi.spyOn(backend, 'setSecret').mockRejectedValue(new Error(`vault rejected ${SECRET}`));
    await user.type(field(), SECRET);
    await user.click(screen.getByRole('button', { name: 'Salvar chave' }));
    expect(await screen.findByText(/Algo deu errado\./)).toBeVisible();
    expect(document.body).not.toHaveTextContent('vault rejected');
  });

  it('disables the button while the key is being stored', async () => {
    const { user, backend } = renderPanel(<OpencodeKey />, { scenario: 'fresh' });
    let release: () => void = () => {};
    const send = vi.spyOn(backend, 'setSecret').mockImplementation(
      () =>
        new Promise((done) => {
          release = () => done({ litellm: false, opencode: true, kimi: false, zai: false });
        }),
    );
    await user.type(field(), SECRET);
    const save = screen.getByRole('button', { name: 'Salvar chave' });
    await user.click(save);
    await user.click(save);
    expect(send).toHaveBeenCalledTimes(1);
    expect(save).toBeDisabled();
    release();
    await vi.waitFor(() => expect(save).toBeEnabled());
  });
});

describe('removing a secret', () => {
  it('clears the flag and says so', async () => {
    const { user, backend } = renderPanel(<OpencodeKey />, { scenario: 'healthy' });
    await user.click(await screen.findByRole('button', { name: 'Remover chave' }));
    expect(await screen.findByText('Chave removida.')).toBeVisible();
    expect(screen.getByText('Sem chave salva')).toBeVisible();
    expect((await backend.secretStatus()).opencode).toBe(false);
  });

  it('keeps the keyboard in the field once the button that was used is gone', async () => {
    const { user } = renderPanel(<OpencodeKey />, { scenario: 'healthy' });
    await user.click(await screen.findByRole('button', { name: 'Remover chave' }));
    expect(await screen.findByText('Chave removida.')).toBeVisible();
    expect(screen.queryByRole('button', { name: 'Remover chave' })).toBeNull();
    expect(field()).toHaveFocus();
  });

  it('says why removal failed', async () => {
    const { user, backend } = renderPanel(<OpencodeKey />, { scenario: 'healthy' });
    vi.spyOn(backend, 'clearSecret').mockRejectedValue(new BackendError(problem('internal')));
    await user.click(await screen.findByRole('button', { name: 'Remover chave' }));
    expect(await screen.findByText(/Algo deu errado\./)).toBeVisible();
  });
});
