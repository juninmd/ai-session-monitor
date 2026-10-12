// Which displayed values are estimates, from signals the contract already carries.
import type { HarnessId, HarnessView, UsageWindow } from '../lib/contract';

/**
 * Antigravity names a window's cadence from its reset time when an older IDE states none, and the
 * collector then leaves `windowMinutes` null (collector-antigravity view.rs). A stated cadence
 * always has its length, so a 5 h or weekly window without one is a guess.
 */
export function hasGuessedCadence(harness: HarnessId, window: UsageWindow): boolean {
  return (
    harness === 'antigravity' &&
    window.windowMinutes === null &&
    (window.kind === 'five_hour' || window.kind === 'weekly')
  );
}

/** Every `local_*` fact of OpenCode is a list-price estimate for this PC, never quota. */
export function isEstimatedFact(view: HarnessView, factId: string): boolean {
  return view.id === 'opencode' && factId.startsWith('local_');
}
