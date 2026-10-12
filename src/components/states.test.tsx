import { screen, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { BackendError } from '../lib/backend';
import type { HarnessId, HarnessView } from '../lib/contract';
import { notice, problem, viewOf } from '../mock/fixtures';
import { ctxOf, renderPanel, settingsWith } from '../test/helpers';
import { HarnessPanel } from './HarnessPanel';

const panel = (view: HarnessView) => <HarnessPanel view={view} ctx={ctxOf()} />;
const heading = (name: string | RegExp) => screen.getByRole('heading', { level: 2, name });
const button = (name: string | RegExp) => screen.getByRole('button', { name });

describe('disabled', () => {
  const disabled = (id: HarnessId) => viewOf(id, { state: 'disabled' });
  const off = settingsWith((s) => {
    s.codex.enabled = false;
  });

  it('says the source is off and offers a way back', () => {
    renderPanel(panel(disabled('codex')), { settings: off });
    expect(heading('Codex está desligado')).toBeVisible();
    expect(screen.getByText('Ative a fonte para voltar a ler os limites.')).toBeVisible();
  });

  it('turns the source on with one click', async () => {
    const { user, backend } = renderPanel(panel(disabled('codex')), { settings: off });
    await user.click(button('Ativar'));
    expect((await backend.getSettings()).codex.enabled).toBe(true);
  });

  it('opens that source in the settings', async () => {
    const { user, openSettings } = renderPanel(panel(disabled('codex')), { settings: off });
    await user.click(button('Abrir configurações'));
    expect(openSettings).toHaveBeenCalledExactlyOnceWith('codex');
  });

  it('says why enabling failed, in words', async () => {
    const { user, backend } = renderPanel(panel(disabled('codex')), { settings: off });
    vi.spyOn(backend, 'saveSettings').mockRejectedValue(
      new BackendError(problem('settings_unreadable')),
    );
    await user.click(button('Ativar'));
    expect(await screen.findByRole('alert')).toHaveTextContent('Arquivo de configurações ilegível');
  });
});

describe('loading', () => {
  it('shows a busy placeholder with a spoken label and nothing to click', () => {
    renderPanel(panel(viewOf('litellm', { state: 'loading' })));
    expect(document.querySelector('[aria-busy="true"]')).not.toBeNull();
    expect(screen.getByText('Carregando a primeira leitura')).toBeInTheDocument();
    expect(screen.queryAllByRole('button')).toHaveLength(0);
  });
});

describe('needs_consent', () => {
  const consent = (id: HarnessId) => panel(viewOf(id, { state: 'needs_consent' }));

  it('asks Claude Code for permission before touching its settings', async () => {
    renderPanel(consent('claude'), { scenario: 'fresh' });
    expect(heading('Conectar ao Claude Code')).toBeVisible();
    expect(screen.getByText('O que será lido')).toBeVisible();
    expect(button('Instalar comando')).toBeEnabled();
    expect(button('Agora não')).toBeEnabled();
    expect(screen.getByText(/o Windows esconde ícones novos/)).toBeVisible();
  });

  it.each(['claude', 'codex', 'antigravity'] as const)(
    'puts the %s logo on its card and keeps the lock beside it',
    (id) => {
      renderPanel(consent(id), { scenario: 'fresh' });
      const badge = heading(/^Conectar ao /)
        .closest('section')
        ?.querySelector('.badge');
      expect(badge?.querySelector(`svg.logo[data-logo="${id}"]`)).not.toBeNull();
      expect(badge?.querySelector('svg.badge-ico')).not.toBeNull();
    },
  );

  it('installs the command, then reads once', async () => {
    const { user, backend } = renderPanel(consent('claude'), { scenario: 'fresh' });
    await user.click(button('Instalar comando'));
    await vi.waitFor(() => expect(backend.calls).toContain('install_claude_shim'));
    const steps = backend.calls.filter((call) =>
      ['save_settings', 'install_claude_shim', 'refresh'].includes(call),
    );
    expect(steps).toEqual(['save_settings', 'install_claude_shim', 'refresh']);
    expect((await backend.getSettings()).claude.shimConsent).toBe(true);
  });

  it('reports a failed install and offers nothing it did not do', async () => {
    const { user, backend } = renderPanel(consent('claude'), { scenario: 'fresh' });
    vi.spyOn(backend, 'installClaudeShim').mockRejectedValue(
      new BackendError(problem('install_failed')),
    );
    await user.click(button('Instalar comando'));
    expect(await screen.findByRole('alert')).toHaveTextContent(
      'Falha ao instalar. Não foi possível instalar o comando',
    );
  });

  it('switches the source off on "Agora não" instead of nagging again', async () => {
    const { user, backend } = renderPanel(consent('claude'), { scenario: 'fresh' });
    await user.click(button('Agora não'));
    expect((await backend.getSettings()).claude).toMatchObject({
      enabled: false,
      shimConsent: false,
    });
  });

  it('asks Codex for permission to start its app-server', async () => {
    const { user, backend } = renderPanel(consent('codex'), { scenario: 'fresh' });
    expect(heading('Conectar ao Codex')).toBeVisible();
    await user.click(button('Permitir leitura'));
    await vi.waitFor(async () => expect((await backend.getSettings()).codex.consent).toBe(true));
    expect(backend.calls).toContain('refresh');
  });

  it('warns that the Antigravity path is unofficial before asking', async () => {
    const { user, backend } = renderPanel(consent('antigravity'), { scenario: 'fresh' });
    expect(heading('Conectar ao Antigravity')).toBeVisible();
    expect(screen.getByText(/Esse caminho não é oficial/)).toBeVisible();
    await user.click(button('Permitir leitura local'));
    await vi.waitFor(async () =>
      expect((await backend.getSettings()).antigravity.consent).toBe(true),
    );
  });

  it('reveals what will be read on demand', async () => {
    const { user } = renderPanel(consent('codex'), { scenario: 'fresh' });
    const details = screen.getByText('O que será lido').closest('details');
    expect(details).not.toHaveAttribute('open');
    await user.click(screen.getByText('O que será lido'));
    expect(details).toHaveAttribute('open');
    expect(within(details as HTMLElement).getByText(/só leitura/)).toBeVisible();
  });
});

describe('needs_setup', () => {
  const setup = (id: HarnessId) => panel(viewOf(id, { state: 'needs_setup' }));

  it('lets OpenCode start from a key, a console link or a local estimate', async () => {
    const { user, backend } = renderPanel(setup('opencode'), { scenario: 'fresh' });
    expect(heading('Conectar ao OpenCode Go')).toBeVisible();
    expect(screen.getByLabelText('Chave de API do OpenCode Go')).toHaveAttribute(
      'type',
      'password',
    );
    await user.click(button(/Abrir o console do OpenCode Go/));
    expect(backend.openedLinks).toEqual(['opencode_go_console']);
    const local = screen.getByRole('switch', { name: /Estimativa local/ });
    expect(local).not.toBeChecked();
    await user.click(local);
    await vi.waitFor(async () =>
      expect((await backend.getSettings()).opencode.localConsent).toBe(true),
    );
  });

  it('puts the OpenCode logo on its card and keeps the key beside it', () => {
    renderPanel(setup('opencode'), { scenario: 'fresh' });
    const badge = heading('Conectar ao OpenCode Go').closest('section')?.querySelector('.badge');
    expect(badge?.querySelector('svg.logo[data-logo="opencode"]')).not.toBeNull();
    expect(badge?.querySelector('svg.badge-ico')).not.toBeNull();
  });

  it('shows the LiteLLM connection form', () => {
    renderPanel(setup('litellm'), { scenario: 'fresh' });
    const title = screen.getByRole('heading', { name: 'Conectar ao LiteLLM' });
    expect(title).toBeVisible();
    expect(title.querySelector('svg.logo[data-logo="litellm"]')).not.toBeNull();
    expect(screen.getByLabelText('URL do servidor')).toBeVisible();
    expect(screen.getByLabelText('Chave de API')).toHaveAttribute('type', 'password');
    expect(button('Salvar e conectar')).toBeEnabled();
  });

  it('renders the same surface for consent and setup states of one source', () => {
    const asConsent = viewOf('opencode', { state: 'needs_consent' });
    renderPanel(panel(asConsent), { scenario: 'fresh' });
    expect(heading('Conectar ao OpenCode Go')).toBeVisible();
  });

  it('says why Claude asks for the command again, and reinstalls from the notice', async () => {
    const view = viewOf('claude', {
      state: 'needs_setup',
      notices: [notice('claude_shim_outdated', 'warn')],
    });
    const { user, backend } = renderPanel(panel(view));
    expect(screen.getByText(/não foi encontrado/)).toBeVisible();
    await user.click(button('Reinstalar'));
    await vi.waitFor(() => expect(backend.calls).toContain('install_claude_shim'));
  });

  it('keeps every notice of a gated view: warnings above its card, information below', () => {
    const view = viewOf('claude', {
      state: 'needs_setup',
      notices: [notice('some_future_code', 'info'), notice('claude_shim_outdated', 'warn')],
    });
    renderPanel(panel(view));
    const card = heading('Conectar ao Claude Code');
    const follows = (el: Element) =>
      card.compareDocumentPosition(el) & Node.DOCUMENT_POSITION_FOLLOWING;
    expect(follows(screen.getByText('Aviso sem tradução: some_future_code'))).toBeTruthy();
    expect(follows(screen.getByText(/não foi encontrado/))).toBeFalsy();
  });
});

describe('no_data', () => {
  it('explains the wait for Claude with the notice the backend sent', () => {
    const view = viewOf('claude', {
      state: 'no_data',
      notices: [{ code: 'claude_waiting_first_session', severity: 'info', params: {} }],
    });
    renderPanel(panel(view));
    expect(heading('Aguardando a primeira leitura')).toBeVisible();
    expect(screen.getByText(/depois da primeira resposta de uma sessão/)).toBeVisible();
  });

  it('falls back to a sentence for that source when no notice came', () => {
    renderPanel(panel(viewOf('claude', { state: 'no_data' })));
    expect(screen.getByText(/Abra o Claude Code e envie uma mensagem/)).toBeVisible();
  });

  it('uses the generic sentence for other sources', () => {
    renderPanel(panel(viewOf('codex', { state: 'no_data' })));
    expect(screen.getByText(/ainda não informou nada/)).toBeVisible();
  });

  it('asks the backend to read that source again', async () => {
    const { user, backend } = renderPanel(panel(viewOf('codex', { state: 'no_data' })));
    await user.click(button('Ler agora'));
    expect(backend.calls).toContain('refresh');
  });

  it('shows a failed read in place', async () => {
    const { user, backend } = renderPanel(panel(viewOf('codex', { state: 'no_data' })));
    vi.spyOn(backend, 'refresh').mockRejectedValue(new BackendError(problem('timeout', {}, true)));
    await user.click(button('Ler agora'));
    expect(await screen.findByRole('alert')).toHaveTextContent('A leitura demorou demais');
  });
});
