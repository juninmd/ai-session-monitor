// Named dev scenarios (`?mock=<name>`) and the consent gates that decide what a source shows
// before it is allowed to read. All data is fabricated; see fixtures.ts.
import {
  defaultSettings,
  HARNESS_IDS,
  type HarnessId,
  type HarnessView,
  type SecretStatus,
  type Settings,
  type ShimStatus,
} from '../lib/contract';
import { deepinfraView, minimaxView, moonshotView, novitaView, ollamaView } from './balances';
import {
  antigravityView,
  BUILDERS,
  claudeView,
  codexView,
  litellmView,
  opencodeView,
} from './builders';
import { asStale, type Env, failed, type Mood, notice, problem, viewOf } from './fixtures';
import { deepseekView, kimiView, openrouterView, zaiView } from './keyed';

export const SCENARIO_NAMES = [
  'demo',
  'fresh',
  'healthy',
  'critical',
  'stale',
  'errors',
  'loading',
] as const;
export type ScenarioName = (typeof SCENARIO_NAMES)[number];

/** Anything that is not a known name falls back to the demo: the query string is untrusted. */
export function scenarioFromSearch(search: string): ScenarioName {
  const raw = new URLSearchParams(search).get('mock');
  return SCENARIO_NAMES.find((name) => name === raw) ?? 'demo';
}

type Maker = (env: Env) => HarnessView;
type Makers = Record<HarnessId, Maker>;

const everyone = (mood: Mood): Makers => ({
  claude: (env) => BUILDERS.claude(env, mood),
  codex: (env) => BUILDERS.codex(env, mood),
  antigravity: (env) => BUILDERS.antigravity(env, mood),
  opencode: (env) => BUILDERS.opencode(env, mood),
  kimi: (env) => BUILDERS.kimi(env, mood),
  zai: (env) => BUILDERS.zai(env, mood),
  openrouter: (env) => BUILDERS.openrouter(env, mood),
  deepseek: (env) => BUILDERS.deepseek(env, mood),
  deepinfra: (env) => BUILDERS.deepinfra(env, mood),
  moonshot: (env) => BUILDERS.moonshot(env, mood),
  novita: (env) => BUILDERS.novita(env, mood),
  ollama: (env) => BUILDERS.ollama(env, mood),
  minimax: (env) => BUILDERS.minimax(env, mood),
  litellm: (env) => BUILDERS.litellm(env, mood),
});

/** The weekly window of a view has already reset: the UI must say "reiniciou", not 0 %. */
function withExpiredWeek(view: HarnessView, env: Env): HarnessView {
  return {
    ...view,
    windows: view.windows.map((w) => (w.kind === 'weekly' ? { ...w, resetsAt: env.now - 600 } : w)),
  };
}

const loading: Makers = {
  claude: () => viewOf('claude', { state: 'loading' }),
  codex: () => viewOf('codex', { state: 'loading' }),
  antigravity: () => viewOf('antigravity', { state: 'loading' }),
  opencode: () => viewOf('opencode', { state: 'loading' }),
  kimi: () => viewOf('kimi', { state: 'loading' }),
  zai: () => viewOf('zai', { state: 'loading' }),
  openrouter: () => viewOf('openrouter', { state: 'loading' }),
  deepseek: () => viewOf('deepseek', { state: 'loading' }),
  deepinfra: () => viewOf('deepinfra', { state: 'loading' }),
  moonshot: () => viewOf('moonshot', { state: 'loading' }),
  novita: () => viewOf('novita', { state: 'loading' }),
  ollama: () => viewOf('ollama', { state: 'loading' }),
  minimax: () => viewOf('minimax', { state: 'loading' }),
  litellm: () => viewOf('litellm', { state: 'loading' }),
};

export const DATA: Record<ScenarioName, Makers> = {
  healthy: everyone('ok'),
  fresh: everyone('ok'),
  critical: everyone('crit'),
  loading,
  demo: {
    claude: (env) => claudeView(env, 'ok'),
    codex: (env) => codexView(env, 'warn'),
    antigravity: (env) => antigravityView(env, 'ok', true, true),
    opencode: (env) => opencodeView(env, 'ok'),
    kimi: (env) => kimiView(env, 'ok'),
    zai: (env) => zaiView(env, 'warn'),
    openrouter: (env) => openrouterView(env, 'warn'),
    deepseek: (env) => deepseekView(env, 'ok'),
    deepinfra: (env) => deepinfraView(env, 'ok'),
    moonshot: (env) => moonshotView(env, 'ok'),
    novita: (env) => novitaView(env, 'ok'),
    ollama: (env) => ollamaView(env, 'ok'),
    minimax: (env) => minimaxView(env, 'ok'),
    litellm: () => failed('litellm', problem('unauthorized', { httpStatus: '401' })),
  },
  stale: {
    claude: (env) => asStale(claudeView(env, 'ok'), env, 23, problem('network', {}, true)),
    codex: (env) => asStale(withExpiredWeek(codexView(env, 'ok'), env), env, 12),
    antigravity: (env) =>
      asStale(antigravityView(env, 'warn'), env, 40, problem('not_running', {}, true)),
    opencode: (env) => asStale(opencodeView(env, 'ok'), env, 35),
    kimi: (env) => asStale(kimiView(env, 'ok'), env, 30, problem('network', {}, true)),
    zai: (env) => asStale(zaiView(env, 'ok'), env, 25),
    openrouter: (env) => asStale(openrouterView(env, 'ok'), env, 20, problem('network', {}, true)),
    deepseek: (env) => asStale(deepseekView(env, 'ok'), env, 22),
    deepinfra: (env) => asStale(deepinfraView(env, 'ok'), env, 22),
    moonshot: (env) => asStale(moonshotView(env, 'ok'), env, 22),
    novita: (env) => asStale(novitaView(env, 'ok'), env, 22),
    ollama: (env) => asStale(ollamaView(env, 'ok'), env, 22),
    minimax: (env) => asStale(minimaxView(env, 'ok'), env, 22),
    litellm: (env) => ({
      ...asStale(
        litellmView(env, 'ok'),
        env,
        18,
        problem('rate_limited', { httpStatus: '429', retryAfterSecs: '240' }, true),
      ),
      nextAttemptAt: env.now + 240,
    }),
  },
  errors: {
    claude: () => failed('claude', problem('install_failed', { reason: 'write_failed' })),
    codex: () => failed('codex', problem('binary_not_found')),
    antigravity: () => failed('antigravity', problem('not_running', {}, true)),
    opencode: () =>
      failed('opencode', problem('unauthorized', { httpStatus: '401', ambiguous: 'true' })),
    kimi: () => failed('kimi', problem('unauthorized', { httpStatus: '401', ambiguous: 'true' })),
    zai: () => failed('zai', problem('endpoint_changed')),
    openrouter: () => failed('openrouter', problem('unauthorized', { httpStatus: '401' })),
    deepseek: () => failed('deepseek', problem('rate_limited', { httpStatus: '429' }, true)),
    deepinfra: () => failed('deepinfra', problem('rate_limited', { httpStatus: '429' }, true)),
    moonshot: () => failed('moonshot', problem('rate_limited', { httpStatus: '429' }, true)),
    novita: () => failed('novita', problem('rate_limited', { httpStatus: '429' }, true)),
    ollama: () => failed('ollama', problem('rate_limited', { httpStatus: '429' }, true)),
    minimax: () => failed('minimax', problem('rate_limited', { httpStatus: '429' }, true)),
    litellm: () =>
      failed(
        'litellm',
        problem('tls_error', { host: 'gateway.exemplo.dev', reason: 'ca_missing' }),
      ),
  },
};

export interface Start {
  settings: Settings;
  secrets: SecretStatus;
  shim: ShimStatus;
}

const SHIM_PATH = 'C:\\Users\\exemplo\\.claude\\settings.json';

export function startFor(scenario: ScenarioName): Start {
  if (scenario === 'fresh') {
    return {
      settings: structuredClone(defaultSettings),
      secrets: {
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
      },
      shim: { state: 'not_installed', chained: false, settingsPath: SHIM_PATH },
    };
  }
  const base = structuredClone(defaultSettings);
  const settings: Settings = {
    ...base,
    general: { ...base.general, firstRunDone: true },
    claude: { ...base.claude, shimConsent: true },
    codex: { ...base.codex, consent: true },
    antigravity: { ...base.antigravity, consent: true },
    opencode: { ...base.opencode, localConsent: true },
    litellm: { ...base.litellm, baseUrl: 'https://gateway.exemplo.dev' },
  };
  return {
    settings,
    secrets: {
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
    },
    shim: {
      state: scenario === 'errors' ? 'outdated' : 'installed',
      chained: scenario === 'demo',
      settingsPath: SHIM_PATH,
    },
  };
}

/** What a source shows before it may read: asks for consent, setup, or waits for a first sample. */
function gate(id: HarnessId, env: Env, firstSessionSeen: boolean): HarnessView | null {
  const { settings, secrets } = env;
  switch (id) {
    case 'claude':
      if (!settings.claude.shimConsent) return viewOf(id, { state: 'needs_consent' });
      if (!firstSessionSeen) {
        return viewOf(id, {
          state: 'no_data',
          notices: [notice('claude_waiting_first_session', 'info')],
        });
      }
      return null;
    case 'codex':
      return settings.codex.consent ? null : viewOf(id, { state: 'needs_consent' });
    case 'antigravity':
      return settings.antigravity.consent ? null : viewOf(id, { state: 'needs_consent' });
    case 'opencode':
      return secrets.opencode || settings.opencode.localConsent
        ? null
        : viewOf(id, { state: 'needs_setup' });
    case 'kimi':
      return secrets.kimi ? null : viewOf(id, { state: 'needs_setup' });
    case 'zai':
      return secrets.zai ? null : viewOf(id, { state: 'needs_setup' });
    case 'openrouter':
      return secrets.openrouter ? null : viewOf(id, { state: 'needs_setup' });
    case 'deepseek':
      return secrets.deepseek ? null : viewOf(id, { state: 'needs_setup' });
    case 'deepinfra':
      return secrets.deepinfra ? null : viewOf(id, { state: 'needs_setup' });
    case 'moonshot':
      return secrets.moonshot ? null : viewOf(id, { state: 'needs_setup' });
    case 'novita':
      return secrets.novita ? null : viewOf(id, { state: 'needs_setup' });
    case 'ollama':
      return secrets.ollama ? null : viewOf(id, { state: 'needs_setup' });
    case 'minimax':
      return secrets.minimax ? null : viewOf(id, { state: 'needs_setup' });
    case 'litellm':
      return settings.litellm.baseUrl !== null && secrets.litellm
        ? null
        : viewOf(id, { state: 'needs_setup' });
  }
}

export function viewsFor(
  scenario: ScenarioName,
  env: Env,
  firstSessionSeen: boolean,
): HarnessView[] {
  return HARNESS_IDS.map((id) => {
    if (!env.settings[id].enabled) return viewOf(id, { state: 'disabled' });
    return gate(id, env, firstSessionSeen) ?? DATA[scenario][id](env);
  });
}
