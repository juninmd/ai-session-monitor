import { createContext } from 'react';
import type { ViewContext } from '../app/windowModel';
import type { HarnessView } from '../lib/contract';
import { isExpired } from '../lib/usage';
import type { MockBackend } from '../mock';

export type ResetOutcome = 'reset' | 'noCredit' | 'nothingToReset' | 'unavailable';
export const PreviewActionsContext = createContext<{
  lowPriorityOffered: boolean;
  consumeReset: () => Promise<ResetOutcome>;
} | null>(null);

/** A missing capability/count is unknown, including the read-only rollout fallback. */
export function resetCreditCount(view: HarnessView, ctx: ViewContext): number | null {
  if (
    view.id !== 'codex' ||
    view.state !== 'ready' ||
    view.stale ||
    view.source?.id !== 'app_server' ||
    !ctx.settings.codex.enabled ||
    !ctx.settings.codex.consent
  )
    return null;
  const count = view.facts.find((fact) => fact.id === 'reset_credits')?.number;
  return count !== undefined &&
    count !== null &&
    Number.isInteger(count) &&
    count >= 0 &&
    count <= 1_000_000
    ? count
    : null;
}

/** Fabricated response only. Real integration must consume once, then re-read server limits. */
export async function simulateCodexReset(
  backend: MockBackend,
  nowSecs: number,
): Promise<ResetOutcome> {
  const settings = await backend.getSettings();
  const snapshot = await backend.getSnapshot();
  const view = snapshot.views.find((candidate) => candidate.id === 'codex');
  if (!view) return 'unavailable';
  const count = resetCreditCount(view, { settings, nowSecs });
  if (count === null) return 'unavailable';
  if (count === 0) return 'noCredit';
  const eligible = view.windows.filter(
    (window) => window.usedPct !== null && window.usedPct > 0 && !isExpired(window, nowSecs),
  );
  if (eligible.length === 0) return 'nothingToReset';
  backend.override({
    ...view,
    readAt: nowSecs,
    facts: view.facts.map((fact) =>
      fact.id === 'reset_credits' ? { ...fact, number: count - 1 } : fact,
    ),
    windows: view.windows.map((window) =>
      eligible.includes(window)
        ? {
            ...window,
            usedPct: 0,
            amount: window.amount ? { ...window.amount, used: 0 } : null,
            resetsAt: window.windowMinutes ? nowSecs + window.windowMinutes * 60 : window.resetsAt,
          }
        : window,
    ),
  });
  return 'reset';
}
