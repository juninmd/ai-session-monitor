// Cursor and GitHub Copilot: the first-run cards, the settings groups and the links they open.
import { screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { BackendError } from '../../lib/backend';
import type { HarnessId, HarnessView } from '../../lib/contract';
import { problem, viewOf } from '../../mock/fixtures';
import { ctxOf, renderPanel, settingsWith } from '../../test/helpers';
import { HarnessPanel } from '../HarnessPanel';
import { SOURCE_GROUPS } from '../settings/SourceGroups';

const panel = (view: HarnessView) => <HarnessPanel view={view} ctx={ctxOf()} />;
const heading = (name: string | RegExp) => screen.getByRole('heading', { level: 2, name });
const button = (name: string | RegExp) => screen.getByRole('button', { name });
const badge = (title: string) => heading(title).closest('section')?.querySelector('.badge');
const CursorGroup = SOURCE_GROUPS.cursor;
const CopilotGroup = SOURCE_GROUPS.copilot;

describe('Cursor first run', () => {
  const asks = (id: HarnessId) => panel(viewOf(id, { state: 'needs_consent' }));

  it('says what is read and that the route is unofficial before asking', async () => {
    const { user, backend } = renderPanel(asks('cursor'), { scenario: 'fresh' });
    expect(heading('Conectar ao Cursor')).toBeVisible();
    expect(screen.getByText(/lê o token de login que o Cursor guarda/)).toBeVisible();
    expect(screen.getByText(/Esse caminho não é oficial/)).toBeVisible();
    await user.click(screen.getByText('O que será lido'));
    expect(screen.getByText(/enviado só ao cursor\.com/)).toBeVisible();
    await user.click(button('Permitir leitura'));
    await vi.waitFor(async () => expect((await backend.getSettings()).cursor.consent).toBe(true));
    expect(backend.calls).toContain('refresh');
  });

  it('puts the Cursor logo on the card and switches the source off on "Agora não"', async () => {
    const { user, backend } = renderPanel(asks('cursor'), { scenario: 'fresh' });
    expect(
      badge('Conectar ao Cursor')?.querySelector('svg.logo[data-logo="cursor"]'),
    ).not.toBeNull();
    await user.click(button('Agora não'));
    expect((await backend.getSettings()).cursor).toMatchObject({ enabled: false, consent: false });
  });

  it('asks to sign in to Cursor once the consent is given, instead of asking again', async () => {
    const given = settingsWith((s) => {
      s.cursor.consent = true;
    });
    const { user, backend } = renderPanel(panel(viewOf('cursor', { state: 'needs_setup' })), {
      settings: given,
    });
    expect(await screen.findByRole('heading', { level: 2, name: 'Entre no Cursor' })).toBeVisible();
    expect(screen.queryByRole('button', { name: 'Permitir leitura' })).toBeNull();
    await user.click(button('Ler agora'));
    expect(backend.calls).toContain('refresh');
  });
});

describe('Copilot first run', () => {
  const setup = () => panel(viewOf('copilot', { state: 'needs_setup' }));

  it('offers a pasted token, the local-token consent and the settings page', async () => {
    const { user, backend } = renderPanel(setup(), { scenario: 'fresh' });
    expect(heading('Conectar ao GitHub Copilot')).toBeVisible();
    expect(screen.getByLabelText('Token do GitHub (opcional)')).toHaveAttribute('type', 'password');
    await user.click(screen.getByText('O que será lido'));
    expect(screen.getByText(/enviado só a api\.github\.com/)).toBeVisible();
    const local = screen.getByRole('switch', { name: /Ler o token da extensão do Copilot/ });
    expect(local).not.toBeChecked();
    await user.click(local);
    await vi.waitFor(async () =>
      expect((await backend.getSettings()).copilot.localConsent).toBe(true),
    );
    await user.click(button(/Abrir as configurações do GitHub Copilot/));
    expect(backend.openedLinks).toEqual(['copilot_settings']);
  });

  it('stores the pasted token as the copilot secret, and never shows it', async () => {
    const { user, backend } = renderPanel(setup(), { scenario: 'fresh' });
    await user.type(screen.getByLabelText('Token do GitHub (opcional)'), 'ghu_fabricated_token');
    await user.click(button('Salvar chave'));
    await vi.waitFor(async () => expect((await backend.secretStatus()).copilot).toBe(true));
    expect(document.body).not.toHaveTextContent('ghu_fabricated_token');
  });

  it('puts the Copilot logo on its card', () => {
    renderPanel(setup(), { scenario: 'fresh' });
    expect(
      badge('Conectar ao GitHub Copilot')?.querySelector('svg.logo[data-logo="copilot"]'),
    ).not.toBeNull();
  });
});

describe('the settings groups', () => {
  it('lets Cursor be read only with the consent, and opens its dashboard', async () => {
    const off = settingsWith((s) => {
      s.cursor.consent = false;
    });
    const { user, backend } = renderPanel(<CursorGroup />, { settings: off });
    const consent = screen.getByRole('switch', {
      name: 'Permitir a leitura do token de login do Cursor',
    });
    expect(consent).not.toBeChecked();
    expect(screen.getByText(/Leitura não oficial/)).toBeVisible();
    await user.click(consent);
    await vi.waitFor(async () => expect((await backend.getSettings()).cursor.consent).toBe(true));
    await user.click(button(/Abrir o painel do Cursor/));
    expect(backend.openedLinks).toEqual(['cursor_dashboard']);
  });

  it('keeps the Copilot token, the local consent and the settings link together', async () => {
    const { user, backend } = renderPanel(<CopilotGroup />);
    expect(screen.getByLabelText('Token do GitHub (opcional)')).toHaveAttribute('type', 'password');
    await vi.waitFor(() =>
      expect(screen.getByRole('switch', { name: /Ler o token da extensão/ })).toBeChecked(),
    );
    await user.click(button(/Abrir as configurações do GitHub Copilot/));
    expect(backend.openedLinks).toEqual(['copilot_settings']);
  });

  it('says so when the dashboard could not be opened', async () => {
    const { user, backend } = renderPanel(<CursorGroup />);
    vi.spyOn(backend, 'openLink').mockRejectedValue(new BackendError(problem('internal')));
    await user.click(button(/Abrir o painel do Cursor/));
    expect(await screen.findByRole('alert')).toHaveTextContent('Algo deu errado.');
  });
});
