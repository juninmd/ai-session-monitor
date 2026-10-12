// TypeScript twin of crates/usage-model/src/status.rs. Both sides are pinned to
// contract/status-vectors.json, so the tab chips here and the tray ring in Rust never disagree.
import type { HarnessView, UsageWindow } from './contract';

// A window younger than this has no meaningful burn rate yet.
const EARLY_ELAPSED_MIN = 10;
// Below this much consumption the burn rate is noise.
const EARLY_USED_PCT = 3;
// Runway shorter than this while the window still has time left is critical.
const CRIT_RUNWAY_MIN = 30;
// Slack before "runs out before the reset" is reported.
const RISK_SLACK_MIN = 5;
const CRIT_AVAILABLE_PCT = 10;

export type Status = 'ok' | 'warn' | 'crit' | 'out';

export type Pace =
  | { kind: 'unknown' }
  | { kind: 'early' }
  | { kind: 'out' }
  | { kind: 'fine'; runwayMin: number }
  | { kind: 'risk'; runwayMin: number; earlyMin: number };

export interface Summary {
  windowId: string;
  availablePct: number;
  status: Status;
}

/** Percentage still available; null while unknown. */
export function availablePct(window: UsageWindow): number | null {
  return window.usedPct === null ? null : Math.min(100, Math.max(0, 100 - window.usedPct));
}

/** A window whose reset moment has passed no longer describes reality. */
export function isExpired(window: UsageWindow, nowSecs: number): boolean {
  return window.resetsAt !== null && window.resetsAt <= nowSecs;
}

/** Minutes until the reset, floored at zero; null when the source gave no reset time. */
export function minutesLeft(window: UsageWindow, nowSecs: number): number | null {
  return window.resetsAt === null ? null : Math.max(0, (window.resetsAt - nowSecs) / 60);
}

export function paceOf(window: UsageWindow, nowSecs: number): Pace {
  if (isExpired(window, nowSecs)) return { kind: 'unknown' };
  const avail = availablePct(window);
  const left = minutesLeft(window, nowSecs);
  if (avail === null || window.windowMinutes === null || left === null) return { kind: 'unknown' };
  if (avail <= 0) return { kind: 'out' };
  const elapsed = window.windowMinutes - left;
  const used = 100 - avail;
  if (elapsed < EARLY_ELAPSED_MIN || used < EARLY_USED_PCT) return { kind: 'early' };
  const runwayMin = avail / (used / elapsed);
  return runwayMin < left - RISK_SLACK_MIN
    ? { kind: 'risk', runwayMin, earlyMin: left - runwayMin }
    : { kind: 'fine', runwayMin };
}

/** Null for unknown or already-reset windows: they must not colour the UI or the tray. */
export function statusOf(window: UsageWindow, nowSecs: number, warnAtPct: number): Status | null {
  const avail = availablePct(window);
  if (avail === null || isExpired(window, nowSecs)) return null;
  if (avail <= 0) return 'out';
  const pace = paceOf(window, nowSecs);
  const critPace = pace.kind === 'risk' && pace.runwayMin < CRIT_RUNWAY_MIN;
  if (avail <= CRIT_AVAILABLE_PCT || critPace) return 'crit';
  if (avail <= warnAtPct || pace.kind === 'risk') return 'warn';
  return 'ok';
}

/** The most constrained usable window of a view; drives the tab chip. */
export function summarize(view: HarnessView, nowSecs: number, warnAtPct: number): Summary | null {
  if (view.state !== 'ready' || view.stale) return null;
  let best: Summary | null = null;
  let bestReset = Number.POSITIVE_INFINITY;
  for (const window of view.windows) {
    const status = statusOf(window, nowSecs, warnAtPct);
    const available = availablePct(window);
    if (status === null || available === null) continue;
    const reset = window.resetsAt ?? Number.POSITIVE_INFINITY;
    const lower = best === null || available < best.availablePct;
    const tiedButSooner = best !== null && available === best.availablePct && reset < bestReset;
    if (lower || tiedButSooner) {
      best = { windowId: window.id, availablePct: available, status };
      bestReset = reset;
    }
  }
  return best;
}
