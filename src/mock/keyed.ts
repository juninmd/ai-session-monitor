// Sample views of the key-only sources: Kimi Code, z.ai, OpenRouter and DeepSeek. Fabricated numbers only.
import type { HarnessView } from '../lib/contract';
import { type Env, fact, type Mood, viewOf, windowOf } from './fixtures';

const KIMI = {
  ok: { five: 25, week: 25, month: 40 },
  warn: { five: 78, week: 66, month: 71 },
  crit: { five: 100, week: 93, month: 88 },
} as const;

/** Five hours and week come with request counts; the monthly pool is a share only. */
export function kimiView(env: Env, mood: Mood): HarnessView {
  const u = KIMI[mood];
  return viewOf('kimi', {
    source: { id: 'coding_usages', official: false },
    plan: 'Intermediate',
    readAt: env.now - 90,
    windows: [
      windowOf(env, {
        id: 'five_hour',
        kind: 'five_hour',
        usedPct: u.five,
        windowMinutes: 300,
        resetsInMin: 140,
        amount: { used: (u.five * 200) / 100, limit: 200, unit: 'requests' },
      }),
      windowOf(env, {
        id: 'weekly',
        kind: 'weekly',
        usedPct: u.week,
        windowMinutes: null,
        resetsInMin: 4320,
        amount: { used: (u.week * 2048) / 100, limit: 2048, unit: 'requests' },
      }),
      windowOf(env, {
        id: 'monthly',
        kind: 'monthly',
        usedPct: u.month,
        windowMinutes: null,
        resetsInMin: 20000,
      }),
    ],
  });
}

const ZAI = {
  ok: { five: 12, week: 40, mcp: 25 },
  warn: { five: 81, week: 72, mcp: 60 },
  crit: { five: 100, week: 96, mcp: 91 },
} as const;

/** The MCP allowance is the monthly tool-call cap, not coding quota. */
export function zaiView(env: Env, mood: Mood): HarnessView {
  const u = ZAI[mood];
  return viewOf('zai', {
    source: { id: 'quota_limit', official: false },
    plan: 'Pro',
    readAt: env.now - 90,
    windows: [
      windowOf(env, {
        id: 'five_hour',
        kind: 'five_hour',
        usedPct: u.five,
        windowMinutes: 300,
        resetsInMin: 120,
      }),
      windowOf(env, {
        id: 'weekly',
        kind: 'weekly',
        usedPct: u.week,
        windowMinutes: 10080,
        resetsInMin: 4320,
      }),
      windowOf(env, {
        id: 'mcp',
        kind: 'monthly',
        usedPct: u.mcp,
        windowMinutes: null,
        resetsInMin: 20000,
        amount: { used: u.mcp * 10, limit: 1000, unit: 'requests' },
      }),
    ],
  });
}

const OPENROUTER = {
  ok: { used: 2.5, limit: 10 },
  warn: { used: 8.1, limit: 10 },
  crit: { used: 10, limit: 10 },
} as const;

/** The limit window exists because this sample key has a cap; the reset moment is not reported. */
export function openrouterView(env: Env, mood: Mood): HarnessView {
  const u = OPENROUTER[mood];
  return viewOf('openrouter', {
    source: { id: 'key_credits', official: true },
    readAt: env.now - 90,
    windows: [
      windowOf(env, {
        id: 'key_limit',
        kind: 'budget',
        usedPct: (u.used / u.limit) * 100,
        windowMinutes: null,
        resetsInMin: null,
        amount: { used: u.used, limit: u.limit, unit: 'usd' },
      }),
    ],
    facts: [
      fact('balance', 74.75, 'usd'),
      fact('usage_daily', 0.25, 'usd'),
      fact('usage_weekly', 1, 'usd'),
      fact('usage_monthly', u.used, 'usd'),
      fact('usage_total', u.used, 'usd'),
    ],
  });
}

/** A balance is money: facts per currency, no percentage and no reset. */
export function deepseekView(env: Env, _mood: Mood): HarnessView {
  return viewOf('deepseek', {
    source: { id: 'user_balance', official: true },
    readAt: env.now - 90,
    facts: [
      fact('balance_cny', 110, 'cny'),
      fact('granted_cny', 10, 'cny'),
      fact('topped_up_cny', 100, 'cny'),
      fact('balance_usd', 3.5, 'usd'),
      fact('granted_usd', 0.5, 'usd'),
      fact('topped_up_usd', 3, 'usd'),
    ],
  });
}
