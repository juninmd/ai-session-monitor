import { beforeEach, describe, expect, it, vi } from 'vitest';
import ipc from '../../contract/ipc.json';
import settings from '../../contract/settings.json';
import snapshot from '../../contract/snapshot.json';

const invoke = vi.hoisted(() => vi.fn());
const listen = vi.hoisted(() => vi.fn());
vi.mock('@tauri-apps/api/core', () => ({ invoke }));
vi.mock('@tauri-apps/api/event', () => ({ listen }));

import { BackendError, createTauriBackend } from './backend';
import { COMMANDS, EVENTS } from './contract';

const backend = createTauriBackend();

beforeEach(() => {
  invoke.mockReset();
  listen.mockReset();
});

describe('commands', () => {
  it.each([
    [
      'getUpdateStatus',
      () => backend.getUpdateStatus(),
      COMMANDS.getUpdateStatus,
      undefined,
      ipc.updateStatus,
    ],
    ['checkUpdate', () => backend.checkUpdate(), COMMANDS.checkUpdate, undefined, ipc.updateStatus],
    [
      'downloadUpdate',
      () => backend.downloadUpdate(),
      COMMANDS.downloadUpdate,
      undefined,
      ipc.updateStatus,
    ],
    [
      'installUpdate',
      () => backend.installUpdate(),
      COMMANDS.installUpdate,
      undefined,
      ipc.updateStatus,
    ],
    ['getSnapshot', () => backend.getSnapshot(), COMMANDS.getSnapshot, undefined, snapshot],
    ['getSettings', () => backend.getSettings(), COMMANDS.getSettings, undefined, settings.default],
    [
      'saveSettings',
      () => backend.saveSettings(settings.default as never),
      COMMANDS.saveSettings,
      { settings: settings.default },
      settings.default,
    ],
    [
      'secretStatus',
      () => backend.secretStatus(),
      COMMANDS.secretStatus,
      undefined,
      ipc.secretStatus,
    ],
    [
      'setSecret',
      () => backend.setSecret('opencode', 'k'),
      COMMANDS.setSecret,
      { kind: 'opencode', value: 'k' },
      ipc.secretStatus,
    ],
    [
      'clearSecret',
      () => backend.clearSecret('litellm'),
      COMMANDS.clearSecret,
      { kind: 'litellm' },
      ipc.secretStatus,
    ],
    [
      'claudeShimStatus',
      () => backend.claudeShimStatus(),
      COMMANDS.claudeShimStatus,
      undefined,
      ipc.shimStatuses[0],
    ],
    [
      'installClaudeShim',
      () => backend.installClaudeShim(),
      COMMANDS.installClaudeShim,
      undefined,
      ipc.shimStatuses[1],
    ],
    [
      'uninstallClaudeShim',
      () => backend.uninstallClaudeShim(),
      COMMANDS.uninstallClaudeShim,
      undefined,
      ipc.shimStatuses[0],
    ],
    [
      'getHistory',
      () => backend.getHistory('claude', 4),
      COMMANDS.getHistory,
      { harness: 'claude', limit: 4 },
      ipc.history,
    ],
  ] as const)(
    '%s calls its command and parses the reply',
    async (_n, run, command, args, reply) => {
      invoke.mockResolvedValue(reply);
      await expect(run()).resolves.toEqual(reply);
      expect(invoke).toHaveBeenCalledWith(command, args);
    },
  );

  it.each([
    ['refresh (all)', () => backend.refresh(), COMMANDS.refresh, { harness: undefined }],
    ['refresh (one)', () => backend.refresh('codex'), COMMANDS.refresh, { harness: 'codex' }],
    [
      'openLink',
      () => backend.openLink('opencode_billing'),
      COMMANDS.openLink,
      { link: 'opencode_billing' },
    ],
    ['hideFlyout', () => backend.hideFlyout(), COMMANDS.hideFlyout, undefined],
    [
      'setFlyoutHeight',
      () => backend.setFlyoutHeight(512),
      COMMANDS.setFlyoutHeight,
      { height: 512 },
    ],
    ['quit', () => backend.quit(), COMMANDS.quitApp, undefined],
    ['clearHistory', () => backend.clearHistory(), COMMANDS.clearHistory, undefined],
  ] as const)('%s sends the right arguments', async (_n, run, command, args) => {
    invoke.mockResolvedValue(null);
    await run();
    expect(invoke).toHaveBeenCalledWith(command, args);
  });
});

describe('failures', () => {
  it('surfaces a backend error with its code and parameters', async () => {
    const failure = { code: 'unauthorized', retryable: false, params: { httpStatus: '401' } };
    invoke.mockRejectedValue(failure);
    const error = await backend.getSnapshot().catch((e: unknown) => e);
    expect(error).toBeInstanceOf(BackendError);
    expect((error as BackendError).error).toEqual(failure);
  });

  it('collapses an unknown rejection to internal and never repeats its text', async () => {
    invoke.mockRejectedValue('failed with token sk-secret-123');
    const error = (await backend.getSnapshot().catch((e: unknown) => e)) as BackendError;
    expect(error.error.code).toBe('internal');
    expect(JSON.stringify(error.error)).not.toContain('sk-secret');
    expect(error.message).not.toContain('sk-secret');
  });

  it('rejects a reply that does not match the contract as a protocol error', async () => {
    invoke.mockResolvedValue({ generatedAt: 'yesterday', views: [] });
    const error = (await backend.getSnapshot().catch((e: unknown) => e)) as BackendError;
    expect(error.error).toEqual({
      code: 'protocol_error',
      retryable: false,
      params: { command: 'get_snapshot' },
    });
  });
});

describe('events', () => {
  it('forwards valid snapshots and drops malformed ones loudly', async () => {
    const stop = vi.fn();
    let handler: (message: { payload: unknown }) => void = () => {};
    listen.mockImplementation(async (_event: string, cb: typeof handler) => {
      handler = cb;
      return stop;
    });
    const received: unknown[] = [];
    const log = vi.spyOn(console, 'error').mockImplementation(() => {});

    const unsubscribe = await backend.onSnapshot((s) => received.push(s));
    expect(listen).toHaveBeenCalledWith(EVENTS.snapshot, expect.any(Function));
    handler({ payload: snapshot });
    handler({ payload: { nope: true } });

    expect(received).toEqual([snapshot]);
    expect(log).toHaveBeenCalledTimes(1);
    expect(unsubscribe).toBe(stop);
  });

  it('reports a malformed event to onInvalid without leaking its payload', async () => {
    let handler: (message: { payload: unknown }) => void = () => {};
    listen.mockImplementation(async (_event: string, cb: typeof handler) => {
      handler = cb;
      return vi.fn();
    });
    vi.spyOn(console, 'error').mockImplementation(() => {});
    const onInvalid = vi.fn();
    await backend.onSnapshot(() => {}, onInvalid);
    handler({ payload: { secret: 'x' } });
    expect(onInvalid).toHaveBeenCalledTimes(1);
    expect(onInvalid).toHaveBeenCalledWith();
  });

  it('notifies when the flyout is shown', async () => {
    let handler: () => void = () => {};
    listen.mockImplementation(async (_event: string, cb: typeof handler) => {
      handler = cb;
      return () => {};
    });
    const shown = vi.fn();
    await backend.onFlyoutShown(shown);
    handler();
    expect(listen).toHaveBeenCalledWith(EVENTS.flyoutShown, expect.any(Function));
    expect(shown).toHaveBeenCalledOnce();
  });
});
