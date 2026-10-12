import { screen, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { t } from '../../i18n';
import { BackendError } from '../../lib/backend';
import { ERROR_CODES, type ErrorCode, type HarnessId, type HarnessView } from '../../lib/contract';
import { codexView } from '../../mock/builders';
import { problem, viewOf } from '../../mock/fixtures';
import { ctxOf, envOf, renderPanel } from '../../test/helpers';
import { HarnessPanel } from '../HarnessPanel';

const failing = (
  id: HarnessId,
  error: ReturnType<typeof problem> | null,
  patch = {},
): HarnessView => viewOf(id, { state: 'error', error, ...patch });

function show(view: HarnessView) {
  return renderPanel(<HarnessPanel view={view} ctx={ctxOf()} />);
}

/** What each code offers, written out here on purpose: this table is the contract. */
const ACTION_LABEL: Record<ErrorCode, string> = {
  internal: 'Tentar de novo',
  timeout: 'Tentar de novo',
  network: 'Tentar de novo',
  bad_response: 'Tentar de novo',
  response_too_large: 'Tentar de novo',
  rate_limited: 'Tentar de novo',
  unauthorized: 'Abrir configurações',
  not_found: 'Abrir configurações',
  server_error: 'Tentar de novo',
  tls_error: 'Abrir configurações',
  invalid_url: 'Abrir configurações',
  invalid_input: 'Abrir configurações',
  settings_unreadable: 'Tentar de novo',
  install_failed: 'Reinstalar',
  unsupported_plan: 'Abrir configurações',
  oauth_expired: 'Tentar de novo',
  binary_not_found: 'Abrir configurações',
  spawn_failed: 'Abrir configurações',
  handshake_failed: 'Tentar de novo',
  protocol_error: 'Tentar de novo',
  not_signed_in: 'Tentar de novo',
  not_running: 'Tentar de novo',
  endpoint_changed: 'Tentar de novo',
};

describe('an error block', () => {
  it.each(ERROR_CODES)('explains %s in pt-BR and offers exactly one action', (code) => {
    show(failing('litellm', problem(code)));
    expect(screen.getByRole('heading', { level: 2 })).toHaveTextContent(t(`error.${code}.title`));
    const buttons = screen.getAllByRole('button');
    expect(buttons).toHaveLength(1);
    expect(buttons[0]).toHaveAccessibleName(ACTION_LABEL[code]);
    expect(document.body).not.toHaveTextContent(code);
  });

  it('adds the HTTP status when the backend knew it', () => {
    show(failing('litellm', problem('server_error', { httpStatus: '503' })));
    expect(screen.getByText(/Código HTTP: 503\./)).toBeVisible();
  });

  it('never prints params it was not told to', () => {
    show(
      failing('litellm', problem('network', { host: 'gateway.exemplo.dev', hint: 'ghp_notReal' })),
    );
    expect(document.body).not.toHaveTextContent('gateway.exemplo.dev');
    expect(document.body).not.toHaveTextContent('ghp_notReal');
  });

  it('degrades to a generic message when the view carries no error at all', () => {
    show(failing('litellm', null));
    expect(screen.getByRole('heading', { level: 2 })).toHaveTextContent('Algo deu errado');
    expect(screen.getAllByRole('button')).toHaveLength(1);
  });
});

describe('error actions', () => {
  it('retry reads that source again', async () => {
    const { user, backend } = show(failing('litellm', problem('timeout', {}, true)));
    const refresh = vi.spyOn(backend, 'refresh');
    await user.click(screen.getByRole('button', { name: 'Tentar de novo' }));
    expect(refresh).toHaveBeenCalledExactlyOnceWith('litellm');
  });

  it('settings opens the group of the source that failed', async () => {
    const { user, openSettings } = show(failing('codex', problem('binary_not_found')));
    await user.click(screen.getByRole('button', { name: 'Abrir configurações' }));
    expect(openSettings).toHaveBeenCalledExactlyOnceWith('codex');
  });

  it('reinstall puts the command back and reads again', async () => {
    const { user, backend } = show(failing('claude', problem('install_failed')));
    await user.click(screen.getByRole('button', { name: 'Reinstalar' }));
    await vi.waitFor(() => expect(backend.calls).toContain('refresh'));
    const steps = backend.calls.filter((c) =>
      /save_settings|install_claude_shim|^refresh$/.test(c),
    );
    expect(steps).toEqual(['save_settings', 'install_claude_shim', 'refresh']);
  });

  it('is disabled while it runs, so a double click cannot start it twice', async () => {
    const { user, backend } = show(failing('litellm', problem('timeout', {}, true)));
    let release: () => void = () => {};
    const refresh = vi.spyOn(backend, 'refresh').mockImplementation(
      () =>
        new Promise<void>((done) => {
          release = done;
        }),
    );
    const retry = screen.getByRole('button', { name: 'Tentar de novo' });
    await user.dblClick(retry);
    expect(refresh).toHaveBeenCalledTimes(1);
    expect(retry).toBeDisabled();
    release();
    await vi.waitFor(() => expect(retry).toBeEnabled());
  });

  it('reports its own failure in words and hides the raw text', async () => {
    const { user, backend } = show(failing('litellm', problem('timeout', {}, true)));
    vi.spyOn(backend, 'refresh').mockRejectedValue(new Error('socket hang up at 10.0.0.9'));
    await user.click(screen.getByRole('button', { name: 'Tentar de novo' }));
    const alert = await screen.findByRole('alert');
    expect(alert).toHaveTextContent('Algo deu errado');
    expect(alert).not.toHaveTextContent('10.0.0.9');
  });
});

describe('errors that name their cause', () => {
  it('send an unreadable IDE token to a retry, not to settings that hold no key', async () => {
    const { user, backend } = show(
      failing('antigravity', problem('unauthorized', { reason: 'no_token' })),
    );
    expect(screen.getByRole('heading', { level: 2 })).toHaveTextContent('Token local indisponível');
    expect(screen.getByText(/Reinicie o Antigravity/)).toBeVisible();
    const buttons = screen.getAllByRole('button');
    expect(buttons).toHaveLength(1);
    const refresh = vi.spyOn(backend, 'refresh');
    await user.click(screen.getByRole('button', { name: 'Tentar de novo' }));
    expect(refresh).toHaveBeenCalledExactlyOnceWith('antigravity');
  });

  it('say the IDE server is starting when it is open but offers no address', () => {
    show(failing('antigravity', problem('not_running', { reason: 'no_port' }, true)));
    expect(screen.getByRole('heading', { level: 2 })).toHaveTextContent(
      'Servidor local ainda não está pronto',
    );
    expect(document.body).not.toHaveTextContent('Programa fechado');
  });

  it('send a LiteLLM answer that is not a record to the settings, naming what to check', async () => {
    const { user, openSettings } = show(
      failing(
        'litellm',
        problem('endpoint_changed', { reason: 'no_record', host: 'h.exemplo.dev' }),
      ),
    );
    expect(screen.getByText(/Confira a URL e o escopo/)).toBeVisible();
    expect(document.body).not.toHaveTextContent('h.exemplo.dev');
    await user.click(screen.getByRole('button', { name: 'Abrir configurações' }));
    expect(openSettings).toHaveBeenCalledExactlyOnceWith('litellm');
  });

  it('point a missing CA file at the CA setting instead of blaming the server certificate', () => {
    show(failing('litellm', problem('tls_error', { reason: 'ca_missing' })));
    expect(screen.getByText(/arquivo da CA configurado não foi encontrado/)).toBeVisible();
    expect(screen.getByRole('button', { name: 'Abrir configurações' })).toBeVisible();
  });

  it('say which field a rejected value belongs to, in words and not as a code', () => {
    show(failing('claude', problem('invalid_input', { field: 'configDir' })));
    expect(
      screen.getByText(/Campo com problema: pasta de configuração do Claude Code\./),
    ).toBeVisible();
    expect(document.body).not.toHaveTextContent('configDir');
  });

  it('tell how long a rate-limited source asked to wait, before the HTTP status', () => {
    show(
      failing(
        'claude',
        problem('rate_limited', { retryAfterSecs: '120', httpStatus: '429' }, true),
      ),
    );
    expect(screen.getByText(/Tempo pedido pela fonte: 2 min\. Código HTTP: 429\.$/)).toBeVisible();
  });
});

describe('OpenCode authentication failures', () => {
  const rejected = (status: string) =>
    failing('opencode', problem('unauthorized', { httpStatus: status, ambiguous: 'true' }));

  it.each(['401', '403'])('say the key or subscription could not be verified (%s)', (status) => {
    show(rejected(status));
    expect(screen.getByRole('heading', { level: 2 })).toHaveTextContent(
      'Não foi possível verificar a chave ou a assinatura Go',
    );
    expect(screen.getByText(/Isso acontece com chaves válidas/)).toBeVisible();
    expect(screen.getByText(new RegExp(`Código HTTP: ${status}\\.`))).toBeVisible();
    expect(document.body).not.toHaveTextContent(/chave (está )?(inválida|incorreta|errada)/i);
  });

  it('offer the console, not a verdict', async () => {
    const { user, backend } = show(rejected('403'));
    const buttons = screen.getAllByRole('button');
    expect(buttons).toHaveLength(1);
    await user.click(screen.getByRole('button', { name: 'Abrir console do OpenCode' }));
    expect(backend.openedLinks).toEqual(['opencode_go_console']);
  });

  it('say so when the console link could not be opened', async () => {
    const { user, backend } = show(rejected('401'));
    vi.spyOn(backend, 'openLink').mockRejectedValue(new BackendError(problem('internal')));
    await user.click(screen.getByRole('button', { name: 'Abrir console do OpenCode' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('Algo deu errado');
  });
});

describe('an error with numbers from before', () => {
  const failedRead = (): HarnessView => {
    const env = envOf();
    return {
      ...codexView(env, 'ok'),
      state: 'error',
      readAt: env.now - 12 * 60,
      error: problem('network', {}, true),
    };
  };

  it('puts the reason first and the old numbers, dimmed and dated, below', () => {
    show(failedRead());
    const heading = screen.getByRole('heading', { level: 2, name: 'Sem conexão com a fonte' });
    const hero = screen.getByRole('heading', { name: 'Janela de 5 horas' });
    expect(heading.compareDocumentPosition(hero) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect(screen.getByText('Mostrando os últimos números lidos, de 12 min atrás.')).toBeVisible();
    expect(document.querySelector('.stack')).toHaveAttribute('data-stale', 'true');
    expect(within(hero.closest('section') as HTMLElement).queryByText(/No ritmo atual/)).toBeNull();
  });

  it('shows no numbers section when there were none', () => {
    show(failing('codex', problem('network', {}, true)));
    expect(document.querySelector('.stack')).toBeNull();
    expect(screen.queryByText(/Mostrando os últimos números/)).toBeNull();
  });
});
