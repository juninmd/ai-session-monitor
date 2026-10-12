// Sample views of the balance-style sources (DeepInfra and its siblings). Fabricated numbers only.
import type { HarnessView } from '../lib/contract';
import { type Env, fact, type Mood, viewOf, windowOf } from './fixtures';

const DEEPINFRA = {
  ok: { used: 2.5, limit: 10 },
  warn: { used: 8.1, limit: 10 },
  crit: { used: 10, limit: 10 },
} as const;

/** The limit window exists because this sample account has a cap; the reset moment is unknown. */
export function deepinfraView(env: Env, mood: Mood): HarnessView {
  const u = DEEPINFRA[mood];
  return viewOf('deepinfra', {
    source: { id: 'payment_checklist', official: true },
    readAt: env.now - 90,
    windows: [
      windowOf(env, {
        id: 'spend_limit',
        kind: 'budget',
        usedPct: (u.used / u.limit) * 100,
        windowMinutes: null,
        resetsInMin: null,
        amount: { used: u.used, limit: u.limit, unit: 'usd' },
      }),
    ],
    facts: [
      fact('balance', 12.5, 'usd'),
      fact('usage_recent', u.used, 'usd'),
      fact('usage_month', 12.34, 'usd'),
    ],
  });
}

/** A balance is money: facts only, no percentage and no reset. */
export function moonshotView(env: Env, _mood: Mood): HarnessView {
  return viewOf('moonshot', {
    source: { id: 'users_balance', official: true },
    readAt: env.now - 90,
    facts: [fact('balance', 49.58, 'usd'), fact('voucher', 46.58, 'usd'), fact('cash', 3, 'usd')],
  });
}

/** A balance is money: facts only, no percentage and no reset. */
export function novitaView(env: Env, _mood: Mood): HarnessView {
  return viewOf('novita', {
    source: { id: 'balance_detail', official: true },
    readAt: env.now - 90,
    facts: [
      fact('balance', 100, 'usd'),
      fact('cash', 80, 'usd'),
      fact('credit_limit', 20, 'usd'),
      fact('pending', 0, 'usd'),
      fact('owed', 0, 'usd'),
    ],
  });
}

const OLLAMA = { ok: 27.5, warn: 80, crit: 100 } as const;

/** A current plan: the monthly included credits, with the period's end as the reset. */
export function ollamaView(env: Env, mood: Mood): HarnessView {
  const used = OLLAMA[mood];
  return viewOf('ollama', {
    source: { id: 'balance', official: true },
    readAt: env.now - 90,
    windows: [
      windowOf(env, {
        id: 'monthly',
        kind: 'monthly',
        usedPct: used,
        windowMinutes: 43200,
        resetsInMin: 20000,
        amount: { used, limit: 100, unit: 'usd' },
      }),
    ],
    facts: [fact('included_balance', 100 - used, 'usd'), fact('purchased_balance', 25, 'usd')],
  });
}

const MINIMAX = {
  ok: { interval: 10, weekly: 5 },
  warn: { interval: 80, weekly: 60 },
  crit: { interval: 100, weekly: 90 },
} as const;

/** One model bucket: the rolling interval states its length and reset, the weekly one neither. */
export function minimaxView(env: Env, mood: Mood): HarnessView {
  const u = MINIMAX[mood];
  return viewOf('minimax', {
    source: { id: 'token_plan_remains', official: false },
    readAt: env.now - 90,
    windows: [
      {
        ...windowOf(env, {
          id: 'minimax_m:interval',
          kind: 'five_hour',
          usedPct: u.interval,
          windowMinutes: 300,
          resetsInMin: 240,
        }),
        group: 'MiniMax-M*',
      },
      {
        ...windowOf(env, {
          id: 'minimax_m:weekly',
          kind: 'weekly',
          usedPct: u.weekly,
          windowMinutes: null,
          resetsInMin: null,
        }),
        group: 'MiniMax-M*',
      },
    ],
  });
}
