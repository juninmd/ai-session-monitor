import { screen, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { SettingsDrawer } from '../../home/SettingsDrawer';
import { renderApp, renderPanel } from '../../test/helpers';
import { VendorKey } from '../setup/KeyConnect';
import { LitellmForm } from '../setup/LitellmForm';

/** The open source group, which is where every draft in these cases lives. */
const group = (id: string) => document.getElementById(`grp-${id}`) as HTMLDetailsElement;
const codexName = () => within(group('codex')).getByText('Codex', { selector: '.grp-name' });
/** The section drawer's own arrow: the footer behind it is another way out, and not this one. */
const back = () => within(screen.getByRole('dialog')).getByRole('button', { name: 'Voltar' });

describe('pending settings edits', () => {
  it('preserves the LiteLLM destination and pasted key across external settings changes', async () => {
    const { user, backend } = renderPanel(<LitellmForm />, { scenario: 'healthy' });
    const field = screen.getByLabelText('URL do servidor');
    await vi.waitFor(async () =>
      expect(field).toHaveValue((await backend.getSettings()).litellm.baseUrl),
    );
    await user.clear(field);
    await user.type(field, 'https://draft.exemplo.dev');
    const key = screen.getByLabelText('Chave de API');
    await user.type(key, 'fabricated-draft-key');
    const next = await backend.getSettings();
    next.litellm.baseUrl = 'https://other.exemplo.dev';
    await backend.saveSettings(next);
    await vi.waitFor(() => expect(field).toHaveValue('https://draft.exemplo.dev'));
    expect(key).toHaveValue('fabricated-draft-key');
    expect(document.body).not.toHaveTextContent('fabricated-draft-key');
  });
  it('waits for an in-flight save before allowing the group to close', async () => {
    const { user, backend } = renderPanel(<SettingsDrawer section="codex" onSection={() => {}} />);
    const field = await screen.findByLabelText('Caminho do executável do Codex');
    await user.type(field, 'saved-path.exe');
    const saveSettings = backend.saveSettings;
    let resume = () => {};
    const gate = new Promise<void>((resolve) => {
      resume = resolve;
    });
    vi.spyOn(backend, 'saveSettings').mockImplementation(async (settings) => {
      await gate;
      return saveSettings(settings);
    });
    await user.click(screen.getByRole('button', { name: 'Salvar caminho' }));
    expect(field).toBeDisabled();
    await user.click(codexName());
    expect(await screen.findByText('Salvando alterações')).toBeVisible();
    expect(screen.getByRole('button', { name: 'Descartar alterações' })).toBeDisabled();
    resume();
    await user.click(await screen.findByRole('button', { name: 'Continuar' }));
    expect(group('codex')).not.toHaveAttribute('open');
    expect((await backend.getSettings()).codex.binaryPath).toBe('saved-path.exe');
  });
  it('requires explicit discard before collapsing an edited source', async () => {
    const { user } = renderPanel(<SettingsDrawer section="codex" onSection={() => {}} />);
    await screen.findByRole('heading', { name: 'Fontes de dados' });
    const field = document.getElementById('codex-binary') as HTMLInputElement;
    await user.clear(field);
    await user.type(field, 'C:\\fabricated\\codex.exe');
    await user.click(codexName());
    expect(field).toBeVisible();
    expect(field).toHaveValue('C:\\fabricated\\codex.exe');
    expect(await screen.findByRole('button', { name: 'Continuar editando' })).toBeVisible();
    await user.click(screen.getByRole('button', { name: 'Continuar editando' }));
    expect(field).toHaveValue('C:\\fabricated\\codex.exe');
    await user.click(codexName());
    await user.click(screen.getByRole('button', { name: 'Descartar alterações' }));
    expect(group('codex')).not.toHaveAttribute('open');
    await user.click(codexName());
    expect(await screen.findByLabelText('Caminho do executável do Codex')).toHaveValue('');
  });

  it('does not navigate away from an edited form without confirmation', async () => {
    const { user } = renderApp({ scenario: 'healthy' });
    await user.click(screen.getByRole('button', { name: 'Configurações' }));
    await user.click(await screen.findByRole('button', { name: /^Fontes de dados/ }));
    await user.click(codexName());
    await user.type(await screen.findByLabelText('Caminho do executável do Codex'), 'edited');
    // The drawer's arrow leaves the section, and the guard stops it there.
    await user.click(back());
    expect(await screen.findByRole('alertdialog')).toBeVisible();
    expect(group('codex')).toHaveAttribute('open');
    expect(screen.getByLabelText('Caminho do executável do Codex')).toHaveValue('edited');
  });

  it('keeps an edited path when settings arrive from another surface', async () => {
    const { user, backend } = renderPanel(<SettingsDrawer section="codex" onSection={() => {}} />);
    const field = await screen.findByLabelText('Caminho do executável do Codex');
    await user.type(field, 'my-draft.exe');
    const next = await backend.getSettings();
    next.codex.binaryPath = 'other-surface.exe';
    await backend.saveSettings(next);
    await vi.waitFor(() => expect(field).toHaveValue('my-draft.exe'));
    await user.click(screen.getByRole('button', { name: 'Salvar caminho' }));
    await screen.findByText('Salvo.');
    expect((await backend.getSettings()).codex.binaryPath).toBe('my-draft.exe');
  });

  it('blocks installation for a pasted key held anywhere, and never echoes it', async () => {
    // A key form stays mounted behind the settings screen (the first-run wizard is kept alive), so
    // the Atualizações drawer can be open with a pasted key waiting next to it.
    const { user, backend } = renderPanel(
      <>
        <SettingsDrawer section="update" onSection={() => {}} />
        <VendorKey harness="openrouter" />
      </>,
    );
    const key = await screen.findByLabelText('Chave de API do OpenRouter');
    await user.type(key, 'fabricated-sensitive-key');
    await user.click(await screen.findByRole('button', { name: 'Buscar atualização' }));
    await user.click(await screen.findByRole('button', { name: 'Baixar atualização' }));
    expect(await screen.findByRole('button', { name: 'Instalar e reiniciar' })).toBeDisabled();
    await user.click(screen.getByRole('button', { name: 'Ver alterações pendentes' }));
    expect(key).toHaveFocus();
    expect(document.body).not.toHaveTextContent('fabricated-sensitive-key');
    expect(backend.calls).not.toContain('install_update');
  });

  it('guards Escape and the way out of the section, and asks only once after a discard', async () => {
    const { user, backend } = renderApp({ scenario: 'healthy' });
    await user.click(screen.getByRole('button', { name: 'Configurações' }));
    await user.click(await screen.findByRole('button', { name: /^Fontes de dados/ }));
    await user.click(codexName());
    await user.type(await screen.findByLabelText('Caminho do executável do Codex'), 'draft');

    // Escape leaves the section, and the guard stops it: the edit survives the second press.
    await user.keyboard('{Escape}');
    expect(await screen.findByRole('alertdialog')).toBeVisible();
    await user.keyboard('{Escape}');
    expect(screen.queryByRole('alertdialog')).toBeNull();
    expect(screen.getByLabelText('Caminho do executável do Codex')).toHaveValue('draft');
    // Quit is guarded by the same registry, wherever the draft is held. The footer sits behind the
    // scrim while a drawer is open and jsdom does not hit-test: this case is about the guard.
    await user.click(screen.getByRole('button', { name: 'Sair' }));
    expect(backend.calls).not.toContain('quit_app');
    await user.click(screen.getByRole('button', { name: 'Continuar editando' }));

    // The arrow leaves the section for the menu, without asking a second time for edits that were
    // just thrown away.
    await user.click(back());
    await user.click(await screen.findByRole('button', { name: 'Descartar alterações' }));
    expect(screen.queryByRole('alertdialog')).toBeNull();
    expect(screen.getByRole('button', { name: /^Fontes de dados/ })).toBeVisible();
    expect(screen.queryByLabelText('Caminho do executável do Codex')).toBeNull();
  });
});
