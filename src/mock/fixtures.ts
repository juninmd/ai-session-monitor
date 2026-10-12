// Fabricated sample data for the dev mock and the tests. Nothing here comes from a real account:
// the numbers are invented and the host is a made-up name that is never contacted.
import type {
  Fact,
  HarnessError,
  HarnessId,
  HarnessView,
  Notice,
  SecretStatus,
  Settings,
  UsageWindow,
  WindowKind,
} from '../lib/contract';

export type Mood = 'ok' | 'warn' | 'crit';

export interface Env {
  /** Epoch seconds every sample is anchored to: resets are fixed points, so countdowns advance. */
  now: number;
  settings: Settings;
  secrets: SecretStatus;
}

export interface WindowSpec {
  id: string;
  kind: WindowKind;
  usedPct: number | null;
  windowMinutes: number | null;
  resetsInMin: number | null;
  group?: string;
  amount?: UsageWindow['amount'];
}

export function windowOf(env: Env, spec: WindowSpec): UsageWindow {
  return {
    id: spec.id,
    kind: spec.kind,
    group: spec.group ?? null,
    label: null,
    windowMinutes: spec.windowMinutes,
    usedPct: spec.usedPct,
    resetsAt: spec.resetsInMin === null ? null : env.now + Math.round(spec.resetsInMin * 60),
    amount: spec.amount ?? null,
  };
}

export function viewOf(id: HarnessId, patch: Partial<HarnessView> = {}): HarnessView {
  return {
    id,
    state: 'ready',
    source: null,
    readAt: null,
    stale: false,
    plan: null,
    windows: [],
    facts: [],
    notices: [],
    error: null,
    nextAttemptAt: null,
    meta: {},
    ...patch,
  };
}

export const notice = (
  code: string,
  severity: Notice['severity'],
  params: Record<string, string> = {},
): Notice => ({
  code,
  severity,
  params,
});

export const fact = (id: string, number: number, unit: string): Fact => ({
  id,
  number,
  text: null,
  unit,
});

/** Old numbers with their age; `error` is the reason the last refresh failed. */
export function asStale(
  view: HarnessView,
  env: Env,
  ageMin: number,
  error?: HarnessError,
): HarnessView {
  return { ...view, stale: true, readAt: env.now - ageMin * 60, error: error ?? null };
}

export function failed(id: HarnessId, error: HarnessError): HarnessView {
  return viewOf(id, { state: 'error', error });
}

export const problem = (
  code: HarnessError['code'],
  params: Record<string, string> = {},
  retryable = false,
): HarnessError => ({ code, retryable, params });
