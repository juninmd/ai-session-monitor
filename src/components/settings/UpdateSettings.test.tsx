import { screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { BackendError } from '../../lib/backend';
import { renderPanel } from '../../test/helpers';
import { UpdateSettings } from './UpdateSettings';

describe('update settings', () => {
  it('shows indeterminate progress when download length is unknown', async () => {
    const { backend } = renderPanel(<UpdateSettings />);
    await screen.findByText('Versão instalada: 0.1.0');
    backend.updater.status = {
      ...backend.updater.status,
      phase: 'downloading',
      downloadedBytes: 1024,
      totalBytes: null,
    };
    for (const listener of backend.updater.listeners) listener(backend.updater.status);
    const progress = await screen.findByRole('progressbar');
    expect(progress).not.toHaveAttribute('value');
    expect(screen.getByRole('button', { name: 'Buscar atualização' })).toBeDisabled();
  });

  it('keeps the app open when queued settings cannot be saved', async () => {
    const { user, backend } = renderPanel(<UpdateSettings />);
    await user.click(await screen.findByRole('button', { name: 'Buscar atualização' }));
    await user.click(await screen.findByRole('button', { name: 'Baixar atualização' }));
    await user.click(await screen.findByRole('button', { name: 'Instalar e reiniciar' }));
    vi.spyOn(backend, 'saveSettings').mockRejectedValue(
      new BackendError({ code: 'internal', retryable: false, params: {} }),
    );
    await user.click(screen.getByRole('button', { name: 'Confirmar instalação' }));
    await screen.findByRole('alert');
    expect(backend.calls).not.toContain('install_update');
  });
  it('checks, downloads and requires a separate installation confirmation', async () => {
    const { user, backend } = renderPanel(<UpdateSettings />);
    await user.click(await screen.findByRole('button', { name: 'Buscar atualização' }));
    await user.click(await screen.findByRole('button', { name: 'Baixar atualização' }));
    await user.click(await screen.findByRole('button', { name: 'Instalar e reiniciar' }));
    expect(backend.calls).not.toContain('install_update');
    await user.click(screen.getByRole('button', { name: 'Agora não' }));
    expect(screen.queryByRole('button', { name: 'Confirmar instalação' })).toBeNull();
    await user.click(screen.getByRole('button', { name: 'Instalar e reiniciar' }));
    await user.click(screen.getByRole('button', { name: 'Confirmar instalação' }));
    await vi.waitFor(() => expect(backend.calls).toContain('install_update'));
    expect(backend.calls.indexOf('save_settings')).toBeLessThan(
      backend.calls.indexOf('install_update'),
    );
  });

  it('persists automatic checks without initiating a download', async () => {
    const { user, backend } = renderPanel(<UpdateSettings />);
    await user.click(screen.getByRole('switch', { name: 'Buscar atualizações automaticamente' }));
    await vi.waitFor(async () =>
      expect((await backend.getSettings()).general.autoUpdateCheck).toBe(false),
    );
    expect(backend.calls).not.toContain('download_update');
  });

  it('blocks installation when signature validation fails and renders notes as text', async () => {
    const { backend } = renderPanel(<UpdateSettings />);
    backend.updater.status = {
      ...backend.updater.status,
      phase: 'error',
      failure: 'signature',
      availableVersion: '0.2.0',
      notes: '<script>alert(1)</script>',
    };
    for (const listener of backend.updater.listeners) listener(backend.updater.status);
    expect(
      await screen.findByText(
        'A assinatura do arquivo não foi validada. A instalação foi bloqueada.',
      ),
    ).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Instalar e reiniciar' })).toBeNull();
    expect(await screen.findByText('<script>alert(1)</script>')).toBeInTheDocument();
  });
});
