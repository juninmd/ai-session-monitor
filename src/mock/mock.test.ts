import { describe, expect, it, vi } from 'vitest';
import { BackendError } from '../lib/backend';
import {
  defaultSettings,
  HARNESS_IDS,
  type HarnessId,
  type Settings,
  SnapshotSchema,
} from '../lib/contract';
import { summarize } from '../lib/usage';
import { NOW } from '../test/helpers';
import { createMockBackend, SCENARIO_NAMES, scenarioFromSearch } from './index';

const backendOf = (scenario: (typeof SCENARIO_NAMES)[number]) =>
  createMockBackend({ scenario, now: () => NOW });

const stateOf = async (backend: ReturnType<typeof backendOf>, id: HarnessId) =>
  (await backend.getSnapshot()).views.find((view) => view.id === id)?.state;

describe('scenarioFromSearch', () => {
  it.each([
    ['?mock=fresh', 'fresh'],
    ['?mock=critical', 'critical'],
    ['?other=1&mock=stale', 'stale'],
    ['', 'demo'],
    ['?mock=', 'demo'],
    ['?mock=FRESH', 'demo'],
    ['?mock=nope', 'demo'],
    ['?mock=__proto__', 'demo'],
    ['?mock=constructor', 'demo'],
    ['?mock=healthy&mock=critical', 'healthy'],
  ])('%j selects %s', (search, scenario) => {
    expect(scenarioFromSearch(search)).toBe(scenario);
  });
});

describe('every scenario', () => {
  it.each(SCENARIO_NAMES)(
    '%s serves a snapshot the contract accepts, in tab order',
    async (name) => {
      const snapshot = await backendOf(name).getSnapshot();
      expect(SnapshotSchema.safeParse(snapshot).success).toBe(true);
      expect(snapshot.views.map((view) => view.id)).toEqual([...HARNESS_IDS]);
      expect(snapshot.generatedAt).toBe(NOW);
    },
  );

  it.each(SCENARIO_NAMES)('%s never carries a secret-looking value', async (name) => {
    const backend = backendOf(name);
    const text = JSON.stringify([await backend.getSnapshot(), await backend.getSettings()]);
    expect(text).not.toMatch(/sk-[A-Za-z0-9]{8,}|Bearer |password|api[_-]?key"\s*:\s*"/i);
  });
});

describe('scenario content', () => {
  it('starts "fresh" with every source waiting for the user', async () => {
    const backend = backendOf('fresh');
    const states = await Promise.all(HARNESS_IDS.map((id) => stateOf(backend, id)));
    expect(states).toEqual([
      'needs_consent',
      'needs_consent',
      'needs_consent',
      'needs_setup',
      'needs_setup',
      'needs_setup',
      'needs_setup',
    ]);
    expect((await backend.getSettings()).general.firstRunDone).toBe(false);
  });

  it('shows "loading" everywhere while loading', async () => {
    const backend = backendOf('loading');
    const states = await Promise.all(HARNESS_IDS.map((id) => stateOf(backend, id)));
    expect(new Set(states)).toEqual(new Set(['loading']));
  });

  it('mixes states in the demo', async () => {
    const backend = backendOf('demo');
    const states = await Promise.all(HARNESS_IDS.map((id) => stateOf(backend, id)));
    expect(states).toEqual(['ready', 'ready', 'ready', 'ready', 'ready', 'ready', 'error']);
  });

  it('makes every source stale in "stale", each with the reason it stopped', async () => {
    const { views } = await backendOf('stale').getSnapshot();
    expect(views.every((view) => view.state === 'ready' && view.stale)).toBe(true);
    expect(views.filter((view) => view.error !== null).length).toBeGreaterThanOrEqual(3);
  });

  it('fails every source in "errors", OpenCode with the ambiguous 401', async () => {
    const { views } = await backendOf('errors').getSnapshot();
    expect(views.every((view) => view.state === 'error')).toBe(true);
    const opencode = views.find((view) => view.id === 'opencode');
    expect(opencode?.error).toMatchObject({
      code: 'unauthorized',
      params: { httpStatus: '401', ambiguous: 'true' },
    });
  });

  it('keeps every source in the clear in "healthy" and in trouble in "critical"', async () => {
    const warnAt = defaultSettings.general.warnAtPct;
    const statuses = async (name: (typeof SCENARIO_NAMES)[number]) =>
      (await backendOf(name).getSnapshot()).views.map(
        (view) => summarize(view, NOW, warnAt)?.status,
      );
    expect(new Set(await statuses('healthy'))).toEqual(new Set(['ok']));
    const critical = await statuses('critical');
    expect(critical.every((status) => status === 'crit' || status === 'out')).toBe(true);
  });

  it('has an unknown pool, an expired week and a stale source across the scenarios', async () => {
    const demo = (await backendOf('demo').getSnapshot()).views;
    const pool = demo.find((view) => view.id === 'antigravity');
    expect(pool?.windows.some((w) => w.usedPct === null)).toBe(true);
    const stale = (await backendOf('stale').getSnapshot()).views;
    const codex = stale.find((view) => view.id === 'codex');
    expect(codex?.windows.some((w) => w.resetsAt !== null && w.resetsAt <= NOW)).toBe(true);
  });
});

describe('settings', () => {
  const change = async (backend: ReturnType<typeof backendOf>, edit: (draft: Settings) => void) => {
    const draft = await backend.getSettings();
    edit(draft);
    return backend.saveSettings(draft);
  };

  it('clamps numbers to the ranges the backend enforces', async () => {
    const saved = await change(backendOf('demo'), (s) => {
      s.general.intervalSecs = 5;
      s.general.warnAtPct = 99;
      s.litellm.pollIntervalSecs = 1;
      s.litellm.timeoutSecs = 500;
    });
    expect(saved.general).toMatchObject({ intervalSecs: 30, warnAtPct: 90 });
    expect(saved.litellm).toMatchObject({ pollIntervalSecs: 60, timeoutSecs: 30 });
  });

  it.each([
    ['http://gateway.exemplo.dev', 'invalid_url', 'baseUrl'],
    ['https://user:pw@gateway.exemplo.dev', 'invalid_url', 'baseUrl'],
    ['gateway.exemplo.dev', 'invalid_url', 'baseUrl'],
  ])('rejects the LiteLLM URL %j', async (baseUrl, code, field) => {
    const backend = backendOf('demo');
    const error = await change(backend, (s) => {
      s.litellm.baseUrl = baseUrl;
    }).catch((failure: unknown) => failure);
    expect(error).toBeInstanceOf(BackendError);
    expect((error as BackendError).error).toMatchObject({ code, params: { field } });
  });

  it.each([
    [
      'a team without an id',
      (s: Settings) => {
        s.litellm.scope = 'team';
        s.litellm.entityId = null;
      },
      'entityId',
    ],
    [
      'an invalid header',
      (s: Settings) => {
        s.litellm.authHeader = 'Bad Header';
      },
      'authHeader',
    ],
    [
      'a custom CA without a file',
      (s: Settings) => {
        s.litellm.tls = { mode: 'custom_ca', pemPath: ' ' };
      },
      'pemPath',
    ],
    [
      'insecure TLS without a host',
      (s: Settings) => {
        s.litellm.tls = { mode: 'insecure', host: '' };
      },
      'host',
    ],
  ])('rejects %s and names the field', async (_name, edit, field) => {
    const error = await change(backendOf('demo'), edit).catch((failure: unknown) => failure);
    expect((error as BackendError).error).toMatchObject({
      code: 'invalid_input',
      params: { field },
    });
  });

  it('leaves the stored settings untouched after a rejection', async () => {
    const backend = backendOf('demo');
    const before = await backend.getSettings();
    await change(backend, (s) => {
      s.litellm.baseUrl = 'http://gateway.exemplo.dev';
    }).catch(() => undefined);
    expect(await backend.getSettings()).toEqual(before);
  });

  it('refuses a payload the contract does not know', async () => {
    const backend = backendOf('demo');
    const draft = { ...(await backend.getSettings()), surprise: true } as unknown as Settings;
    await expect(backend.saveSettings(draft)).rejects.toThrow();
  });

  it('tells subscribers about a saved change', async () => {
    const backend = backendOf('demo');
    const seen = vi.fn();
    await backend.onSettings(seen);
    await change(backend, (s) => {
      s.general.autostart = true;
    });
    await Promise.resolve();
    expect(seen).toHaveBeenCalledTimes(1);
    expect(seen.mock.calls[0]?.[0].general.autostart).toBe(true);
  });
});
