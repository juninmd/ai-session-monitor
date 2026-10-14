// Boots the real entry point. The Tauri bridge is replaced by a fake that serves fabricated data.
import { screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { COMMANDS, EVENTS } from './lib/contract';
import { createMockBackend } from './mock';

const bridge = vi.hoisted(() => ({ invoke: vi.fn(), listen: vi.fn() }));
const roots = vi.hoisted(() => [] as Array<{ unmount: () => void }>);
vi.mock('@tauri-apps/api/core', () => ({ invoke: bridge.invoke }));
vi.mock('@tauri-apps/api/event', () => ({ listen: bridge.listen }));
// main.tsx creates its own root; keep hold of it so a finished test can take the app down.
vi.mock('react-dom/client', async (importOriginal) => {
  const actual = await importOriginal<typeof import('react-dom/client')>();
  return {
    ...actual,
    createRoot: (container: Element | DocumentFragment) => {
      const root = actual.createRoot(container);
      roots.push(root);
      return root;
    },
  };
});

async function boot(search = '') {
  document.body.innerHTML = '<div id="root"></div>';
  window.history.replaceState(null, '', `/${search}`);
  vi.resetModules();
  await import('./main');
}

beforeEach(() => {
  // The app owns its own React root here, outside Testing Library's act scope.
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', false);
  bridge.invoke.mockReset();
  bridge.listen.mockReset();
});

afterEach(() => {
  // A leftover app would keep calling the shared fake bridge during the next test.
  for (const root of roots.splice(0)) root.unmount();
  Reflect.deleteProperty(window, '__TAURI_INTERNALS__');
  delete document.documentElement.dataset.host;
  window.history.replaceState(null, '', '/');
  document.body.innerHTML = '';
});

describe('in a browser', () => {
  it('runs on fabricated data and marks the host as a browser', async () => {
    await boot();
    // The first boot in a worker is cold and the mock adds real latency; 1 s starves under CPU load.
    await screen.findByRole('heading', { name: /^Providers ativos/ }, { timeout: 5000 });
    expect(await screen.findByRole('tab', { name: /Claude Code/ })).toBeVisible();
    expect(document.documentElement.dataset.host).toBe('browser');
    expect(bridge.invoke).not.toHaveBeenCalled();
    expect(bridge.listen).not.toHaveBeenCalled();
  });

  it('picks the scenario named in the query string', async () => {
    await boot('?mock=errors');
    expect(await screen.findByRole('heading', { name: 'Falha ao instalar' })).toBeVisible();
  });

  it.each(['?mock=nope', '?mock=__proto__', '?mock=%00', '?mock='])(
    'falls back to the demo for %s',
    async (search) => {
      await boot(search);
      expect(await screen.findByRole('heading', { name: /^Providers ativos/ })).toBeVisible();
    },
  );
});

describe('inside Tauri', () => {
  async function fakeBridge(fail: unknown = null) {
    const model = createMockBackend();
    const served = {
      [COMMANDS.getSnapshot]: await model.getSnapshot(),
      [COMMANDS.getSettings]: await model.getSettings(),
      [COMMANDS.secretStatus]: await model.secretStatus(),
      [COMMANDS.getUpdateStatus]: await model.getUpdateStatus(),
    } as Record<string, unknown>;
    bridge.invoke.mockImplementation(async (command: string) => {
      if (fail !== null && command === COMMANDS.getSnapshot) throw fail;
      return served[command];
    });
    const unlisten = vi.fn();
    bridge.listen.mockResolvedValue(unlisten);
    Reflect.set(window, '__TAURI_INTERNALS__', {});
    return { unlisten };
  }

  it('talks to the bridge, not to the mock, and marks the host as Tauri', async () => {
    await fakeBridge();
    await boot();
    await screen.findByRole('tab', { name: /Claude Code/ });
    expect(document.documentElement.dataset.host).toBe('tauri');
    const commands = bridge.invoke.mock.calls.map(([command]) => command);
    expect(commands).toEqual(expect.arrayContaining([COMMANDS.getSnapshot, COMMANDS.getSettings]));
  });

  it('leaves one listener for every event, despite StrictMode', async () => {
    const { unlisten } = await fakeBridge();
    await boot();
    await screen.findByRole('heading', { name: /^Providers ativos/ });
    await vi.waitFor(() =>
      expect(bridge.listen.mock.calls.length - unlisten.mock.calls.length).toBe(4),
    );
    const events = new Set(bridge.listen.mock.calls.map(([event]) => event));
    expect(events).toEqual(
      new Set([EVENTS.snapshot, EVENTS.settingsChanged, EVENTS.flyoutShown, EVENTS.updateStatus]),
    );
  });

  it('shows a failed first read in words and never the raw rejection', async () => {
    await fakeBridge('ECONNRESET with token fab-token-123');
    await boot();
    expect(
      await screen.findByRole('heading', { name: 'Não foi possível carregar o monitor' }),
    ).toBeVisible();
    expect(document.body).not.toHaveTextContent('ECONNRESET');
    expect(document.body).not.toHaveTextContent('fab-token-123');
  });
});

describe('a broken page', () => {
  it('logs a failed start by error name only', async () => {
    const log = vi.spyOn(console, 'error').mockImplementation(() => {});
    document.body.innerHTML = '';
    vi.resetModules();
    await import('./main');
    await vi.waitFor(() => expect(log).toHaveBeenCalledWith('start failed', 'Error'));
    expect(JSON.stringify(log.mock.calls)).not.toContain('#root');
  });
});
