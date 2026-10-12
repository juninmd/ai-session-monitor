import { screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import type { HarnessView, Notice } from '../../lib/contract';
import { notice, viewOf } from '../../mock/fixtures';
import { ctxOf, renderPanel, win } from '../../test/helpers';
import { HarnessPanel } from '../HarnessPanel';

function ready(view: HarnessView) {
  return renderPanel(<HarnessPanel view={view} ctx={ctxOf()} />);
}

const withNotices = (...notices: Notice[]) => viewOf('codex', { windows: [win()], notices });

describe('notices', () => {
  it('take the style of their severity', () => {
    ready(
      withNotices(
        notice('codex_rollout_fallback', 'info'),
        notice('codex_no_limits', 'warn'),
        notice('some_future_code', 'error'),
      ),
    );
    const warn = screen.getByText(/não informa limites/).closest('.banner');
    const error = screen.getByText('Aviso sem tradução: some_future_code').closest('.banner');
    const info = screen.getByText(/Lendo a última sessão gravada/).closest('.note');
    expect(warn).not.toBeNull();
    expect(warn).not.toHaveClass('error');
    expect(error).toHaveClass('error');
    expect(info).not.toBeNull();
  });

  it('name warnings and errors in words, since a tint says nothing to a screen reader', () => {
    ready(
      withNotices(
        notice('codex_rollout_fallback', 'info'),
        notice('codex_no_limits', 'warn'),
        notice('some_future_code', 'error'),
      ),
    );
    expect(screen.getByText(/não informa limites/).closest('.banner')).toHaveTextContent(
      /^Atenção: /,
    );
    expect(
      screen.getByText('Aviso sem tradução: some_future_code').closest('.banner'),
    ).toHaveTextContent(/^Erro: /);
    expect(
      screen.getByText(/Lendo a última sessão gravada/).closest('.note'),
    ).not.toHaveTextContent('Informação');
  });

  it('put warnings above the numbers and plain information below them', () => {
    ready(withNotices(notice('codex_rollout_fallback', 'info'), notice('codex_no_limits', 'warn')));
    const hero = screen.getByRole('heading', { name: 'Janela de 5 horas' });
    const follows = (el: Element) =>
      hero.compareDocumentPosition(el) & Node.DOCUMENT_POSITION_FOLLOWING;
    expect(follows(screen.getByText(/Lendo a última sessão gravada/))).toBeTruthy();
    expect(follows(screen.getByText(/não informa limites/))).toBeFalsy();
  });

  it('show a code nobody translated yet, with the code in the message', () => {
    ready(withNotices(notice('some_future_code', 'info')));
    expect(screen.getByText('Aviso sem tradução: some_future_code')).toBeVisible();
  });

  it('collapse repeats', () => {
    ready(withNotices(notice('codex_no_limits', 'warn'), notice('codex_no_limits', 'warn')));
    expect(screen.getAllByText(/não informa limites/)).toHaveLength(1);
  });

  it('offer to reinstall a missing command and do it', async () => {
    const view = viewOf('claude', {
      windows: [win()],
      notices: [notice('claude_shim_outdated', 'warn')],
    });
    const { user, backend } = ready(view);
    await user.click(screen.getByRole('button', { name: 'Reinstalar' }));
    await vi.waitFor(() => expect(backend.calls).toContain('install_claude_shim'));
  });

  it('send "informar a chave" to the OpenCode settings', async () => {
    const view = viewOf('opencode', {
      windows: [win()],
      notices: [notice('opencode_no_key', 'info')],
    });
    const { user, openSettings } = ready(view);
    await user.click(screen.getByRole('button', { name: 'Informar a chave' }));
    expect(openSettings).toHaveBeenCalledExactlyOnceWith('opencode');
  });

  it('say why a notice action failed without echoing the raw error', async () => {
    const view = viewOf('claude', {
      windows: [win()],
      notices: [notice('claude_shim_outdated', 'warn')],
    });
    const { user, backend } = ready(view);
    vi.spyOn(backend, 'installClaudeShim').mockRejectedValue(new Error('EPERM C:\\private\\file'));
    await user.click(screen.getByRole('button', { name: 'Reinstalar' }));
    const alert = await screen.findByRole('alert');
    expect(alert).toHaveTextContent('Algo deu errado');
    expect(alert).not.toHaveTextContent('EPERM');
  });
});
