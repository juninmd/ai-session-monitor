// Sample views of the two key-only sources, Kimi Code and z.ai. Fabricated numbers only.
import type { HarnessView } from '../lib/contract';
import { type Env, type Mood, viewOf, windowOf } from './fixtures';

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
