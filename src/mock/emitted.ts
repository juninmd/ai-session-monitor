// Views shaped the way each collector crate really emits them: window ids, labels and groups,
// fact ids and units, meta and notice params. They carry the edge cases the friendly builders in
// builders.ts leave out. Fabricated numbers and names; nothing here comes from an account.
import type { Amount, HarnessView, UsageWindow } from '../lib/contract';
import { type Env, fact, notice, viewOf, type WindowSpec, windowOf } from './fixtures';

const DAY = 86_400;

/** A window as a collector builds it: id, label and group set after the numbers. */
function built(
  env: Env,
  spec: WindowSpec,
  extra: { label?: string; group?: string } = {},
): UsageWindow {
  return { ...windowOf(env, spec), label: extra.label ?? null, group: extra.group ?? null };
}

const usd = (used: number, limit: number): Amount => ({ used, limit, unit: 'usd' });

/** Statusline windows, the gateway spend cap, and the OAuth per-model extras. */
export function claudeEmitted(env: Env): HarnessView {
  const week = { kind: 'weekly', windowMinutes: 10080, resetsInMin: 6432 } as const;
  return viewOf('claude', {
    source: { id: 'statusline', official: true },
    plan: 'Max 5x',
    readAt: env.now - 120,
    windows: [
      built(env, {
        id: 'five_hour',
        kind: 'five_hour',
        usedPct: 40,
        windowMinutes: 300,
        resetsInMin: 90,
      }),
      built(env, { id: 'weekly', usedPct: 31, ...week }),
      built(env, {
        id: 'spend_limit',
        kind: 'budget',
        usedPct: 32,
        windowMinutes: 1440,
        resetsInMin: 600,
        amount: usd(3.2, 10),
      }),
      built(env, { id: 'opus:weekly', usedPct: 18, ...week }, { group: 'opus' }),
      built(env, { id: 'sonnet:weekly', usedPct: 29, ...week }, { group: 'sonnet' }),
      built(
        env,
        { id: 'claude_design:weekly', usedPct: 5, ...week },
        { group: 'claude_design', label: 'Claude Design' },
      ),
    ],
    meta: { extra_source: 'oauth_usage' },
  });
}

/** A length that matches no known cadence arrives as primary/secondary with kind other. */
export function codexEmitted(env: Env): HarnessView {
  return viewOf('codex', {
    source: { id: 'rollout', official: false },
    plan: 'Pro',
    readAt: env.now - 3600,
    windows: [
      built(env, {
        id: 'primary',
        kind: 'other',
        usedPct: 30,
        windowMinutes: 2880,
        resetsInMin: 1200,
      }),
      built(env, {
        id: 'secondary',
        kind: 'other',
        usedPct: 55,
        windowMinutes: 20160,
        resetsInMin: 9000,
      }),
    ],
    facts: [fact('credits_balance', 40, 'credits')],
    notices: [notice('codex_rollout_fallback', 'info', { reason: 'binary_not_found' })],
  });
}

/** A named pool, then a group the server did not name, with a bucket that states no cadence. */
export function antigravityEmitted(env: Env): HarnessView {
  const bucket = (pool: string, kind: 'five_hour' | 'weekly', used: number | null) =>
    built(
      env,
      {
        id: `${pool}:${kind}`,
        kind,
        usedPct: used,
        windowMinutes: kind === 'weekly' ? 10080 : 300,
        resetsInMin: 130,
      },
      { group: pool },
    );
  return viewOf('antigravity', {
    source: { id: 'language_server', official: false },
    plan: 'Plano de teste',
    readAt: env.now - 15,
    windows: [
      bucket('gemini', 'five_hour', 54),
      bucket('gemini', 'weekly', 29),
      bucket('group_2', 'five_hour', null),
      built(
        env,
        {
          id: 'group_2:cota_mensal_extra',
          kind: 'other',
          usedPct: null,
          windowMinutes: null,
          resetsInMin: null,
        },
        { group: 'group_2', label: 'Cota mensal extra' },
      ),
    ],
    notices: [notice('antigravity_pool_unknown', 'warn', { pool: 'group_2' })],
  });
}

/** Remote windows plus the local estimate, one of them idle, and a database that stayed shut. */
export function opencodeEmitted(env: Env): HarnessView {
  return viewOf('opencode', {
    source: { id: 'go_usage', official: false },
    readAt: env.now - 60,
    windows: [
      built(env, {
        id: 'rolling',
        kind: 'five_hour',
        usedPct: 0,
        windowMinutes: 300,
        resetsInMin: 300,
      }),
      built(env, {
        id: 'weekly',
        kind: 'weekly',
        usedPct: 12,
        windowMinutes: 10080,
        resetsInMin: 8100,
      }),
      built(env, {
        id: 'monthly',
        kind: 'monthly',
        usedPct: 21,
        windowMinutes: 43200,
        resetsInMin: 21000,
      }),
    ],
    facts: [
      fact('local_cost_5h', 0.42, 'usd'),
      fact('local_cost_7d', 3.18, 'usd'),
      fact('local_cost_30d', 11.9, 'usd'),
      fact('local_tokens_5h', 184_000, 'tokens'),
      fact('local_tokens_7d', 1_200_000, 'tokens'),
    ],
    notices: [
      notice('opencode_window_idle', 'info', { window: 'rolling' }),
      notice('opencode_local_estimate', 'info'),
      notice('opencode_db_unreadable', 'warn', { reason: 'locked' }),
    ],
    meta: { local_estimate: 'true' },
  });
}

/** The key has no cap of its own, so the team's budget is shown, with caps and per-model limits. */
export function litellmEmitted(env: Env): HarnessView {
  const cap = (id: string, used: number, limit: number, minutes: number | null, label: string) =>
    built(
      env,
      {
        id,
        kind: 'budget',
        usedPct: (used / limit) * 100,
        windowMinutes: minutes,
        resetsInMin: minutes === null ? null : 25,
        amount: usd(used, limit),
      },
      { label },
    );
  const model = (name: string, used: number, limit: number) => ({
    ...cap(`model:${name}`, used, limit, null, name),
    group: name,
  });
  return viewOf('litellm', {
    source: { id: 'key_info', official: true },
    readAt: env.now - 30,
    windows: [
      { ...cap('budget', 12.5, 50, 43200, 'squad-alfa'), resetsAt: env.now + 6 * DAY },
      cap('limit:1h', 3.1, 5, 60, '1h'),
      cap('limit:30d', 20, 80, 43200, '30d'),
      model('modelo-a', 40, 50),
      model('modelo-b', 2, 50),
    ],
    facts: [
      fact('rpm_limit', 60, 'requests'),
      fact('tpm_limit', 200_000, 'tokens'),
      fact('max_parallel_requests', 4, 'requests'),
      fact('expires', env.now + 30 * DAY, 'epoch_s'),
    ],
    notices: [
      notice('litellm_level_unreadable', 'warn', { level: 'user', reason: 'timeout' }),
      notice('litellm_windows_truncated', 'info', { dropped: '3' }),
    ],
    meta: { host: 'gateway.exemplo.dev', scope: 'key', level: 'team', alias: 'dev-pessoal' },
  });
}
