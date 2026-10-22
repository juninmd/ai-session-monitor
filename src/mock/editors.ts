// Sample views of the editor sources: Cursor and GitHub Copilot. Fabricated numbers only.
import type { HarnessView, Notice } from '../lib/contract';
import { type Env, type Mood, notice, viewOf, windowOf } from './fixtures';

const CURSOR = {
  ok: { monthly: 36, spent: 2.4 },
  warn: { monthly: 82, spent: 11.6 },
  crit: { monthly: 100, spent: 20 },
} as const;
const CURSOR_CAP = 20;
const CURSOR_CYCLE_MIN = 43200;

/** The included allowance is a share only; the on-demand cap is money, so it carries the amount. */
export function cursorView(env: Env, mood: Mood): HarnessView {
  const u = CURSOR[mood];
  const cycle = { windowMinutes: CURSOR_CYCLE_MIN, resetsInMin: 14 * 1440 + 360 };
  return viewOf('cursor', {
    source: { id: 'usage_summary', official: false },
    plan: 'Pro Plus',
    readAt: env.now - 120,
    windows: [
      windowOf(env, { id: 'monthly', kind: 'monthly', usedPct: u.monthly, ...cycle }),
      windowOf(env, {
        id: 'on_demand',
        kind: 'budget',
        usedPct: (u.spent / CURSOR_CAP) * 100,
        ...cycle,
        amount: { used: u.spent, limit: CURSOR_CAP, unit: 'usd' },
      }),
    ],
  });
}

const COPILOT = {
  ok: { pct: 28, used: 84 },
  warn: { pct: 83, used: 249 },
  // An overdrawn quota reads 100 % while the amount keeps the real count.
  crit: { pct: 100, used: 312 },
} as const;
const COPILOT_ENTITLEMENT = 300;

/** The reset is the 1st of the next UTC month; no window states a length, so none can raise a pace alarm. */
export function copilotView(env: Env, mood: Mood, derived = false): HarnessView {
  const u = COPILOT[mood];
  const notices: Notice[] = derived ? [notice('copilot_reset_derived', 'info')] : [];
  return viewOf('copilot', {
    source: { id: 'copilot_internal_user', official: false },
    plan: 'Individual Pro',
    readAt: env.now - 150,
    windows: [
      windowOf(env, {
        id: 'premium_requests',
        kind: 'monthly',
        usedPct: u.pct,
        windowMinutes: null,
        resetsInMin: 9 * 1440 + 200,
        amount: { used: u.used, limit: COPILOT_ENTITLEMENT, unit: 'requests' },
      }),
    ],
    notices,
    meta: {
      token_source: env.secrets.copilot ? 'pasted' : 'local',
      ...(derived ? { reset_derived: 'true' } : {}),
    },
  });
}
