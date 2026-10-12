// Fabricated burndown history for the dev mock: derived from the windows a scenario shows, with
// deterministic shapes (no randomness) so screenshots and tests are stable.
import {
  type HarnessHistory,
  type HarnessView,
  type HistoryEpoch,
  MAX_EPOCHS_PER_WINDOW,
  type UsageWindow,
} from '../lib/contract';

const DEFAULT_MINUTES: Record<UsageWindow['kind'], number> = {
  five_hour: 300,
  daily: 1440,
  weekly: 10_080,
  monthly: 43_200,
  budget: 43_200,
  other: 1440,
};
/** How much of the current peak each earlier window reached, newest first. */
const PAST_SHARE = [0.62, 0.91, 0.35, 0.78, 0.5, 0.85, 0.7];
/** Curve exponents: below 1 front-loaded use, above 1 a late rush. */
const SHAPES = [1, 0.7, 1.6, 1, 0.5, 1.3, 0.85, 1.1];
const POINTS = 28;

function epochOf(
  window: UsageWindow,
  age: number,
  nowSecs: number,
  lengthSecs: number,
): HistoryEpoch | null {
  if (window.usedPct === null || window.resetsAt === null) return null;
  const resetsAt = window.resetsAt - age * lengthSecs;
  const start = resetsAt - lengthSecs;
  const current = age === 0;
  const reach = current ? Math.min(1, Math.max(0.04, (nowSecs - start) / lengthSecs)) : 1;
  const peak = current ? window.usedPct : window.usedPct * (PAST_SHARE[age - 1] ?? 0.5);
  const shape = SHAPES[age % SHAPES.length] ?? 1;
  const points = Array.from({ length: POINTS }, (_, i) => {
    const x = i / (POINTS - 1);
    return {
      t: Math.round(start + x * reach * lengthSecs),
      usedPct: Math.round(Math.min(100, peak * x ** shape) * 10) / 10,
    };
  });
  return { windowId: window.id, resetsAt, points };
}

/** The current window plus up to seven earlier ones for every window of `view` with a reading. */
export function demoHistory(view: HarnessView, nowSecs: number, limit: number): HarnessHistory {
  const epochs: HistoryEpoch[] = [];
  for (const window of view.windows) {
    const lengthSecs = (window.windowMinutes ?? DEFAULT_MINUTES[window.kind]) * 60;
    const wanted = Math.min(limit, MAX_EPOCHS_PER_WINDOW);
    for (let age = wanted - 1; age >= 0; age -= 1) {
      const epoch = epochOf(window, age, nowSecs, lengthSecs);
      if (epoch !== null) epochs.push(epoch);
    }
  }
  return { harness: view.id, epochs };
}
