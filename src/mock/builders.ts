// One sample view per source and mood. Fabricated numbers only; see fixtures.ts.
import type { HarnessId, HarnessView, Notice } from '../lib/contract';
import { type Env, fact, type Mood, notice, viewOf, windowOf } from './fixtures';

const CLAUDE = {
  ok: { five: 48, week: 31, fiveReset: 113, opus: 18, sonnet: 29 },
  warn: { five: 84, week: 59, fiveReset: 113, opus: 45, sonnet: 62 },
  crit: { five: 100, week: 45, fiveReset: 47, opus: 55, sonnet: 70 },
} as const;

export function claudeView(env: Env, mood: Mood): HarnessView {
  const u = CLAUDE[mood];
  const windows = [
    windowOf(env, {
      id: 'five_hour',
      kind: 'five_hour',
      usedPct: u.five,
      windowMinutes: 300,
      resetsInMin: u.fiveReset,
    }),
    windowOf(env, {
      id: 'weekly',
      kind: 'weekly',
      usedPct: u.week,
      windowMinutes: 10080,
      resetsInMin: 6432,
    }),
  ];
  if (env.settings.claude.oauthUsage) {
    for (const [group, used] of [
      ['opus', u.opus],
      ['sonnet', u.sonnet],
    ] as const) {
      windows.push(
        windowOf(env, {
          id: `${group}:weekly`,
          kind: 'weekly',
          group,
          usedPct: used,
          windowMinutes: 10080,
          resetsInMin: 6432,
        }),
      );
    }
  }
  return viewOf('claude', {
    source: { id: 'statusline', official: true },
    plan: 'Max 5×',
    readAt: env.now - 4 * 60,
    windows,
    meta: env.settings.claude.oauthUsage ? { extra_source: 'oauth_usage' } : {},
  });
}

const CODEX = {
  ok: { five: 26, week: 28, fiveReset: 185, weekReset: 5980 },
  warn: { five: 78, week: 82, fiveReset: 95, weekReset: 3100 },
  crit: { five: 94, week: 97, fiveReset: 60, weekReset: 2400 },
} as const;

export function codexView(env: Env, mood: Mood): HarnessView {
  const u = CODEX[mood];
  return viewOf('codex', {
    source: { id: 'app_server', official: true },
    plan: 'Plus',
    readAt: env.now - 20,
    windows: [
      windowOf(env, {
        id: 'five_hour',
        kind: 'five_hour',
        usedPct: u.five,
        windowMinutes: 300,
        resetsInMin: u.fiveReset,
      }),
      windowOf(env, {
        id: 'weekly',
        kind: 'weekly',
        usedPct: u.week,
        windowMinutes: 10080,
        resetsInMin: u.weekReset,
      }),
    ],
    facts: [fact('credits_balance', 120, 'credits')],
  });
}

const POOLS = {
  ok: { gem5: 54, gemWeek: 29, cg5: 39, cgWeek: 12 },
  warn: { gem5: 81, gemWeek: 48, cg5: 92, cgWeek: 30 },
  crit: { gem5: 96, gemWeek: 70, cg5: 100, cgWeek: 55 },
} as const;

export function antigravityView(env: Env, mood: Mood, unknownPool = false): HarnessView {
  const u = POOLS[mood];
  const unknown = (value: number) => (unknownPool ? null : value);
  const pool = (group: string, five: number | null, week: number | null) => [
    windowOf(env, {
      id: `${group}:five_hour`,
      kind: 'five_hour',
      group,
      usedPct: five,
      windowMinutes: 300,
      resetsInMin: 132,
    }),
    windowOf(env, {
      id: `${group}:weekly`,
      kind: 'weekly',
      group,
      usedPct: week,
      windowMinutes: 10080,
      resetsInMin: 5120,
    }),
  ];
  return viewOf('antigravity', {
    source: { id: 'language_server', official: false },
    readAt: env.now - 15,
    windows: [
      ...pool('gemini', u.gem5, u.gemWeek),
      ...pool('claude_gpt', unknown(u.cg5), unknown(u.cgWeek)),
    ],
    notices: unknownPool
      ? [notice('antigravity_pool_unknown', 'warn', { pool: 'claude_gpt' })]
      : [],
  });
}

const OPENCODE = {
  ok: { rolling: 12, weekly: 12, monthly: 21 },
  warn: { rolling: 80, weekly: 91, monthly: 64 },
  crit: { rolling: 100, weekly: 97, monthly: 88 },
} as const;

/** Windows need the Go key; the facts need the local-estimate consent; either alone still reads. */
export function opencodeView(env: Env, mood: Mood): HarnessView {
  const u = OPENCODE[mood];
  const { opencode } = env.settings;
  const windows = env.secrets.opencode
    ? [
        windowOf(env, {
          id: 'rolling',
          kind: 'five_hour',
          usedPct: u.rolling,
          windowMinutes: 300,
          resetsInMin: 245,
        }),
        windowOf(env, {
          id: 'weekly',
          kind: 'weekly',
          usedPct: u.weekly,
          windowMinutes: 10080,
          resetsInMin: 8100,
        }),
        windowOf(env, {
          id: 'monthly',
          kind: 'monthly',
          usedPct: u.monthly,
          windowMinutes: null,
          resetsInMin: 21000,
        }),
      ]
    : [];
  const notices: Notice[] = [];
  if (mood === 'crit' && windows.length > 0) {
    notices.push(notice('opencode_rate_limited', 'warn', { window: 'rolling' }));
  }
  if (!env.secrets.opencode) notices.push(notice('opencode_no_key', 'info'));
  if (opencode.localConsent) notices.push(notice('opencode_local_estimate', 'info'));
  // The real collector flags an estimate that sits next to real windows.
  const mixed = windows.length > 0 && opencode.localConsent;
  return viewOf('opencode', {
    meta: mixed ? { local_estimate: 'true' } : {},
    source: { id: windows.length > 0 ? 'go_usage' : 'local_db', official: false },
    readAt: env.now - 60,
    windows,
    facts: opencode.localConsent
      ? [
          fact('local_cost_5h', 0.42, 'usd'),
          fact('local_cost_7d', 3.18, 'usd'),
          fact('local_tokens_5h', 184000, 'tokens'),
        ]
      : [],
    notices,
  });
}

const LITELLM_SPENT = { ok: 12.4, warn: 44.1, crit: 51.3 } as const;
const LITELLM_LIMIT = 50;

export function litellmView(env: Env, mood: Mood): HarnessView {
  const { litellm } = env.settings;
  const spent = LITELLM_SPENT[mood];
  let host = 'gateway.exemplo.dev';
  try {
    if (litellm.baseUrl) host = new URL(litellm.baseUrl).hostname;
  } catch {
    // An unparsable URL never reaches the mock through saveSettings; keep the sample host.
  }
  return viewOf('litellm', {
    source: { id: 'key_info', official: true },
    readAt: env.now - 60,
    windows: [
      windowOf(env, {
        id: 'budget',
        kind: 'budget',
        usedPct: (spent / LITELLM_LIMIT) * 100,
        windowMinutes: 43200,
        resetsInMin: 8880,
        amount: { used: spent, limit: LITELLM_LIMIT, unit: 'usd' },
      }),
    ],
    facts: [
      fact('rpm_limit', 60, 'requests'),
      fact('tpm_limit', 200000, 'tokens'),
      fact('expires', env.now + 45 * 86_400, 'epoch_s'),
    ],
    meta: { alias: 'dev-pessoal', host, scope: litellm.scope, level: litellm.scope },
  });
}

export const BUILDERS: Record<HarnessId, (env: Env, mood: Mood) => HarnessView> = {
  claude: claudeView,
  codex: codexView,
  antigravity: (env, mood) => antigravityView(env, mood),
  opencode: opencodeView,
  litellm: litellmView,
};
