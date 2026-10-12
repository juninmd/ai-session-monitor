import { describe, expect, it, vi } from 'vitest';
import { BackendError } from '../lib/backend';
import { HarnessHistorySchema, type HarnessId, type HarnessView } from '../lib/contract';
import { NOW } from '../test/helpers';
import { failed, problem } from './fixtures';
import { createMockBackend } from './index';

const make = (scenario: Parameters<typeof createMockBackend>[0] = {}) =>
  createMockBackend({ now: () => NOW, ...scenario });

const viewOf = async (backend: ReturnType<typeof make>, id: HarnessId): Promise<HarnessView> => {
  const view = (await backend.getSnapshot()).views.find((candidate) => candidate.id === id);
  if (!view) throw new Error(`no view for ${id}`);
  return view;
};

describe('secrets', () => {
  it('are write-only flags: the value never comes back', async () => {
    const backend = make({ scenario: 'fresh' });
    const status = await backend.setSecret('opencode', 'fab-key-1');
    expect(status).toEqual({ litellm: false, opencode: true });
    expect(
      JSON.stringify([status, await backend.secretStatus(), await backend.getSnapshot()]),
    ).not.toContain('fab-key-1');
  });

  it.each(['', '   ', '\n\t'])('refuse a blank value %j and keep the flag', async (value) => {
    const backend = make({ scenario: 'fresh' });
    const error = await backend.setSecret('litellm', value).catch((failure: unknown) => failure);
    expect(error).toBeInstanceOf(BackendError);
    expect((error as BackendError).error).toMatchObject({
      code: 'invalid_input',
      params: { field: 'value' },
    });
    expect((await backend.secretStatus()).litellm).toBe(false);
  });

  it('can be removed again', async () => {
    const backend = make({ scenario: 'healthy' });
    expect(await backend.clearSecret('opencode')).toEqual({ litellm: true, opencode: false });
  });

  it('decide whether a source can read: OpenCode gets its windows once the key is saved', async () => {
    const backend = make({ scenario: 'fresh' });
    expect((await viewOf(backend, 'opencode')).state).toBe('needs_setup');
    await backend.setSecret('opencode', 'fab-key-1');
    const view = await viewOf(backend, 'opencode');
    expect(view.state).toBe('ready');
    expect(view.windows.length).toBeGreaterThan(0);
  });

  it('are not enough for LiteLLM without a URL', async () => {
    const backend = make({ scenario: 'fresh' });
    await backend.setSecret('litellm', 'fab-key-2');
    expect((await viewOf(backend, 'litellm')).state).toBe('needs_setup');
  });
});

describe('the Claude command', () => {
  it('moves through not installed, installed and removed', async () => {
    const backend = make({ scenario: 'fresh' });
    expect((await backend.claudeShimStatus()).state).toBe('not_installed');
    const draft = await backend.getSettings();
    draft.claude.shimConsent = true;
    await backend.saveSettings(draft);
    expect((await backend.installClaudeShim()).state).toBe('installed');
    expect(await backend.uninstallClaudeShim()).toMatchObject({
      state: 'not_installed',
      chained: false,
    });
  });

  it('refuses to install before the user agreed', async () => {
    const error = await make({ scenario: 'fresh' })
      .installClaudeShim()
      .catch((failure: unknown) => failure);
    expect((error as BackendError).error).toMatchObject({
      code: 'invalid_input',
      params: { field: 'shimConsent' },
    });
  });

  it.each([
    ['demo', 'installed', true],
    ['healthy', 'installed', false],
    ['errors', 'outdated', false],
  ] as const)('starts "%s" as %s (chained: %s)', async (scenario, state, chained) => {
    expect(await make({ scenario }).claudeShimStatus()).toMatchObject({ state, chained });
  });

  it('waits for a first session: the second refresh finds data', async () => {
    const backend = make({ scenario: 'fresh' });
    const draft = await backend.getSettings();
    draft.claude.shimConsent = true;
    await backend.saveSettings(draft);
    await backend.installClaudeShim();

    const waiting = await viewOf(backend, 'claude');
    expect(waiting.state).toBe('no_data');
    expect(waiting.notices.map((n) => n.code)).toEqual(['claude_waiting_first_session']);

    await backend.refresh('claude');
    expect((await viewOf(backend, 'claude')).state).toBe('no_data');
    await backend.refresh('claude');
    expect((await viewOf(backend, 'claude')).state).toBe('ready');
  });
});

describe('events', () => {
  it('push a snapshot after a refresh and stop after unsubscribing', async () => {
    const backend = make();
    const seen = vi.fn();
    const stop = await backend.onSnapshot(seen);
    await backend.refresh();
    await Promise.resolve();
    expect(seen).toHaveBeenCalledTimes(1);
    stop();
    await backend.refresh();
    await Promise.resolve();
    expect(seen).toHaveBeenCalledTimes(1);
  });

  it('let a test swap one view and tell the UI', async () => {
    const backend = make();
    const seen = vi.fn();
    await backend.onSnapshot(seen);
    backend.override(failed('codex', problem('network', {}, true)));
    expect((await viewOf(backend, 'codex')).state).toBe('error');
    expect(seen).toHaveBeenCalledTimes(1);
  });

  it('announce that the flyout was shown', async () => {
    const backend = make();
    const shown = vi.fn();
    await backend.onFlyoutShown(shown);
    backend.emitFlyoutShown();
    expect(shown).toHaveBeenCalledTimes(1);
  });

  it('stamp a refreshed source with the current time, but not stale data', async () => {
    let clock = NOW;
    const backend = createMockBackend({ scenario: 'stale', now: () => clock });
    const before = await viewOf(backend, 'codex');
    clock = NOW + 120;
    await backend.refresh('codex');
    expect((await viewOf(backend, 'codex')).readAt).toBe(before.readAt);

    const healthy = createMockBackend({ scenario: 'healthy', now: () => clock });
    await healthy.refresh('codex');
    expect((await viewOf(healthy, 'codex')).readAt).toBe(NOW + 120);
  });
});

describe('commands without a visible result', () => {
  it('are recorded in order so tests can check them', async () => {
    const backend = make();
    await backend.openLink('opencode_go_console');
    await backend.setFlyoutHeight(512);
    await backend.hideFlyout();
    await backend.quit();
    expect(backend.calls).toEqual(['open_link', 'set_flyout_height', 'hide_flyout', 'quit_app']);
    expect(backend.openedLinks).toEqual(['opencode_go_console']);
    expect(backend.lastFlyoutHeight).toBe(512);
  });

  it('hand out copies, so a caller cannot edit the stored state by accident', async () => {
    const backend = make();
    const settings = await backend.getSettings();
    settings.general.autostart = true;
    expect((await backend.getSettings()).general.autostart).toBe(false);
    const secrets = await backend.secretStatus();
    secrets.litellm = !secrets.litellm;
    expect(await backend.secretStatus()).not.toEqual(secrets);
  });

  it('can simulate a slow backend', async () => {
    vi.useFakeTimers();
    const backend = createMockBackend({ latencyMs: 20, now: () => NOW });
    const done = vi.fn();
    void backend.getSnapshot().then(done);
    await vi.advanceTimersByTimeAsync(19);
    expect(done).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(1);
    expect(done).toHaveBeenCalledTimes(1);
  });
});

describe('history', () => {
  it('serves demo history that matches the contract for the demo and none for a fresh install', async () => {
    const demo = await make({ scenario: 'demo' }).getHistory('claude');
    expect(HarnessHistorySchema.parse(demo)).toEqual(demo);
    expect(demo.epochs.length).toBeGreaterThan(8);
    expect(await make({ scenario: 'fresh' }).getHistory('claude')).toEqual({
      harness: 'claude',
      epochs: [],
    });
  });

  it('ends the current epoch at the reading the panel shows', async () => {
    const backend = make({ scenario: 'demo' });
    const window = (await viewOf(backend, 'claude')).windows[0];
    const epochs = (await backend.getHistory('claude', 1)).epochs.filter(
      (epoch) => epoch.windowId === window?.id,
    );
    expect(epochs).toHaveLength(1);
    expect(epochs[0]?.resetsAt).toBe(window?.resetsAt);
    expect(epochs[0]?.points.at(-1)?.usedPct).toBe(window?.usedPct);
  });

  it('refuses a limit outside 1..8 and clears on request', async () => {
    const backend = make({ scenario: 'demo' });
    for (const bad of [0, 9, 1.5]) {
      const error = await backend.getHistory('claude', bad).catch((failure: unknown) => failure);
      expect((error as BackendError).error).toMatchObject({
        code: 'invalid_input',
        params: { field: 'limit' },
      });
    }
    await backend.clearHistory();
    expect((await backend.getHistory('claude')).epochs).toEqual([]);
  });

  it('serves nothing while the setting is off', async () => {
    const backend = make({ scenario: 'demo' });
    const settings = await backend.getSettings();
    await backend.saveSettings({
      ...settings,
      general: { ...settings.general, historyEnabled: false },
    });
    expect((await backend.getHistory('claude')).epochs).toEqual([]);
  });
});
