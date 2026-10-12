import { act, renderHook, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { BackendError } from '../lib/backend';
import type { Settings } from '../lib/contract';
import { createMockBackend, type MockBackend } from '../mock';
import { problem } from '../mock/fixtures';
import { NOW } from '../test/helpers';
import { useSettingsState } from './useSettingsState';

describe('useSettingsState', () => {
  const load = async (options: Parameters<typeof createMockBackend>[0] = {}) => {
    const backend = createMockBackend({ now: () => NOW, ...options });
    const hook = renderHook(() => useSettingsState(backend));
    await waitFor(() => expect(hook.result.current.loaded).toBe(true));
    return { backend, ...hook };
  };

  it('loads settings and which secrets exist', async () => {
    const { result } = await load({ scenario: 'healthy' });
    expect(result.current.settings.general.firstRunDone).toBe(true);
    expect(result.current.secrets).toEqual({
      litellm: true,
      opencode: true,
      kimi: true,
      zai: true,
      openrouter: true,
      deepseek: true,
      deepinfra: true,
      moonshot: true,
      novita: true,
      ollama: true,
      minimax: true,
    });
    expect(result.current.loadError).toBeNull();
  });

  it('never saves a patch on top of the placeholder defaults', async () => {
    const backend = createMockBackend({ scenario: 'healthy', now: () => NOW, latencyMs: 5 });
    const { result } = renderHook(() => useSettingsState(backend));
    // The stored file is not loaded yet: saving now must fetch it first.
    await act(async () => {
      await result.current.save((s) => ({ ...s, general: { ...s.general, autostart: true } }));
    });
    const stored = await backend.getSettings();
    expect(stored.general.autostart).toBe(true);
    expect(stored.general.firstRunDone).toBe(true);
    expect(stored.claude.shimConsent).toBe(true);
    expect(stored.litellm.baseUrl).toBe('https://gateway.exemplo.dev');
  });

  it('applies concurrent saves in order, each on top of the previous one', async () => {
    const { backend, result } = await load();
    await act(async () => {
      await Promise.all([
        result.current.save((s) => ({ ...s, general: { ...s.general, autostart: true } })),
        result.current.save((s) => ({ ...s, general: { ...s.general, notifyOnOut: true } })),
      ]);
    });
    const stored = await backend.getSettings();
    expect(stored.general).toMatchObject({ autostart: true, notifyOnOut: true });
    expect(result.current.settings.general).toMatchObject({ autostart: true, notifyOnOut: true });
  });

  it('rejects a bad save, leaves the stored value alone and keeps the queue moving', async () => {
    const { backend, result } = await load();
    const before = structuredClone(result.current.settings);
    let failure: unknown;
    await act(async () => {
      await result.current
        .save((s: Settings) => ({
          ...s,
          litellm: { ...s.litellm, baseUrl: 'http://gateway.exemplo.dev' },
        }))
        .catch((error: unknown) => {
          failure = error;
        });
    });
    expect(failure).toBeInstanceOf(BackendError);
    expect((failure as BackendError).error).toMatchObject({
      code: 'invalid_url',
      params: { field: 'baseUrl' },
    });
    expect(await backend.getSettings()).toEqual(before);

    await act(async () => {
      await result.current.save((s) => ({ ...s, general: { ...s.general, autostart: true } }));
    });
    expect((await backend.getSettings()).general.autostart).toBe(true);
  });

  it('does not replace settings with an identical copy', async () => {
    const { backend, result } = await load();
    const before = result.current.settings;
    await act(async () => {
      await backend.saveSettings(structuredClone(before));
    });
    expect(result.current.settings).toBe(before);
  });

  it('takes settings that changed somewhere else', async () => {
    const { backend, result } = await load();
    const next = structuredClone(result.current.settings);
    next.general.warnAtPct = 40;
    await act(async () => {
      await backend.saveSettings(next);
    });
    await waitFor(() => expect(result.current.settings.general.warnAtPct).toBe(40));
  });

  it('tracks which secrets are saved without ever holding their value', async () => {
    const { result } = await load({ scenario: 'fresh' });
    expect(result.current.secrets.opencode).toBe(false);
    await act(async () => {
      await result.current.setSecret('opencode', 'fab-key-9');
    });
    expect(result.current.secrets.opencode).toBe(true);
    expect(JSON.stringify(result.current)).not.toContain('fab-key-9');
    await act(async () => {
      await result.current.clearSecret('opencode');
    });
    expect(result.current.secrets.opencode).toBe(false);
  });

  describe('after a save', () => {
    const repoint = (s: Settings): Settings => ({
      ...s,
      litellm: { ...s.litellm, baseUrl: 'https://outro.exemplo.dev' },
    });

    /** The real backend drops the LiteLLM key when a save changes where it would be sent. */
    const droppingTheKey = (backend: MockBackend) => {
      const original = backend.saveSettings;
      vi.spyOn(backend, 'saveSettings').mockImplementation(async (next) => {
        const saved = await original(next);
        await backend.clearSecret('litellm');
        return saved;
      });
    };

    it('asks again which secrets exist, because a save can drop a stored key', async () => {
      const { backend, result } = await load({ scenario: 'healthy' });
      expect(result.current.secrets.litellm).toBe(true);
      droppingTheKey(backend);
      await act(async () => {
        await result.current.save(repoint);
      });
      expect(result.current.secrets.litellm).toBe(false);
    });

    it('asks again after a failed save too: the key can go before the write fails', async () => {
      const { backend, result } = await load({ scenario: 'healthy' });
      vi.spyOn(backend, 'saveSettings').mockImplementation(async () => {
        await backend.clearSecret('litellm');
        throw new BackendError(problem('internal', { where: 'settings_write' }, true));
      });
      await act(async () => {
        await result.current.save(repoint).catch(() => undefined);
      });
      expect(result.current.secrets.litellm).toBe(false);
    });

    it('reports the save as done when the secrets cannot be read again', async () => {
      const { backend, result } = await load({ scenario: 'healthy' });
      vi.spyOn(backend, 'secretStatus').mockRejectedValue(
        new BackendError(problem('internal', { where: 'secret_store' })),
      );
      let saved: Settings | undefined;
      await act(async () => {
        saved = await result.current.save(repoint);
      });
      expect(saved?.litellm.baseUrl).toBe('https://outro.exemplo.dev');
      expect(result.current.secrets.litellm).toBe(true);
      expect(result.current.loadError).toBeNull();
    });
  });

  describe('when the Credential Manager cannot be read', () => {
    const storeDown = () => new BackendError(problem('internal', { where: 'secret_store' }));
    const start = async () => {
      const backend = createMockBackend({ scenario: 'healthy', now: () => NOW });
      vi.spyOn(backend, 'secretStatus').mockRejectedValueOnce(storeDown());
      const hook = renderHook(() => useSettingsState(backend));
      await waitFor(() => expect(hook.result.current.secretsError?.code).toBe('internal'));
      return { backend, ...hook };
    };

    it('reports it apart from loadError, so the stored settings still load', async () => {
      const { result } = await start();
      await waitFor(() => expect(result.current.loaded).toBe(true));
      expect(result.current.loadError).toBeNull();
      expect(result.current.settings.general.firstRunDone).toBe(true);
      expect(result.current.secrets).toEqual({
        litellm: false,
        opencode: false,
        kimi: false,
        zai: false,
        openrouter: false,
        deepseek: false,
        deepinfra: false,
        moonshot: false,
        novita: false,
        ollama: false,
        minimax: false,
      });
    });

    it('reads the secrets again on reload and drops the failure', async () => {
      const { result } = await start();
      act(() => result.current.reload());
      await waitFor(() => expect(result.current.secretsError).toBeNull());
      expect(result.current.secrets).toEqual({
        litellm: true,
        opencode: true,
        kimi: true,
        zai: true,
        openrouter: true,
        deepseek: true,
        deepinfra: true,
        moonshot: true,
        novita: true,
        ollama: true,
        minimax: true,
      });
    });

    it('stops reporting it once a save finds the store readable again', async () => {
      const { result } = await start();
      await waitFor(() => expect(result.current.loaded).toBe(true));
      await act(async () => {
        await result.current.save((s) => ({ ...s, general: { ...s.general, autostart: true } }));
      });
      expect(result.current.secretsError).toBeNull();
      expect(result.current.secrets).toEqual({
        litellm: true,
        opencode: true,
        kimi: true,
        zai: true,
        openrouter: true,
        deepseek: true,
        deepinfra: true,
        moonshot: true,
        novita: true,
        ollama: true,
        minimax: true,
      });
    });

    it('stops reporting it once a key is stored, because the store answered', async () => {
      const { result } = await start();
      await act(async () => {
        await result.current.setSecret('opencode', 'fab-key-3');
      });
      expect(result.current.secretsError).toBeNull();
    });

    it('stops reporting it once a key is removed, because the store answered', async () => {
      const { result } = await start();
      await act(async () => {
        await result.current.clearSecret('opencode');
      });
      expect(result.current.secretsError).toBeNull();
    });
  });

  it('records consent before installing the shim, and withdraws it after removing it', async () => {
    const { backend, result } = await load({ scenario: 'fresh' });
    await act(async () => {
      await result.current.installShim();
    });
    expect((await backend.getSettings()).claude.shimConsent).toBe(true);
    expect(result.current.shim?.state).toBe('installed');
    const names = backend.calls.filter((c) => /save_settings|install_claude_shim/.test(c));
    expect(names).toEqual(['save_settings', 'install_claude_shim']);

    await act(async () => {
      await result.current.uninstallShim();
    });
    expect(result.current.shim?.state).toBe('not_installed');
    expect((await backend.getSettings()).claude.shimConsent).toBe(false);
  });

  it('keeps the settings unloaded and reports why when the read fails, then recovers', async () => {
    const backend = createMockBackend({ now: () => NOW });
    vi.spyOn(backend, 'getSettings').mockRejectedValueOnce(
      new BackendError(problem('settings_unreadable', {}, true)),
    );
    const { result } = renderHook(() => useSettingsState(backend));
    await waitFor(() => expect(result.current.loadError?.code).toBe('settings_unreadable'));
    expect(result.current.loaded).toBe(false);
    act(() => result.current.reload());
    await waitFor(() => expect(result.current.loaded).toBe(true));
    expect(result.current.loadError).toBeNull();
  });
});
