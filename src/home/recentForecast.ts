import type { HistoryEpoch, UsageWindow } from '../lib/contract';

const LOOKBACK_SECS = 3600;
const MIN_INTERVAL_SECS = 300;
const MAX_AGE_SECS = 900;

export interface RecentForecast {
  earliestMin: number;
  /** Null when the slower observed rate reaches the reset or the usage was flat. */
  latestMin: number | null;
}

/** Project two observed rates from the latest sample in the current quota cycle. */
export function recentForecast(
  window: UsageWindow,
  epochs: readonly HistoryEpoch[],
  nowSecs: number,
): RecentForecast | null {
  const { resetsAt, windowMinutes, usedPct } = window;
  if (
    resetsAt === null ||
    resetsAt <= nowSecs ||
    windowMinutes === null ||
    windowMinutes <= 0 ||
    usedPct === null ||
    usedPct >= 100
  ) {
    return null;
  }
  const cycleStart = resetsAt - windowMinutes * 60;
  const epoch = epochs.find((item) => item.windowId === window.id && item.resetsAt === resetsAt);
  if (!epoch) return null;
  const points = epoch.points.filter(
    ({ t }) => t >= Math.max(cycleStart, nowSecs - LOOKBACK_SECS) && t <= nowSecs,
  );
  const start = points[0];
  const end = points.at(-1);
  if (!start || !end || nowSecs - end.t > MAX_AGE_SECS || Math.abs(end.usedPct - usedPct) > 0.1) {
    return null;
  }
  const midpoint = (start.t + end.t) / 2;
  const middle = points
    .filter(({ t }) => t - start.t >= MIN_INTERVAL_SECS && end.t - t >= MIN_INTERVAL_SECS)
    .sort((a, b) => Math.abs(a.t - midpoint) - Math.abs(b.t - midpoint))[0];
  if (
    !middle ||
    points.some((point, i) => i > 0 && point.usedPct < (points[i - 1]?.usedPct ?? 0))
  ) {
    return null;
  }
  const rates = [
    (middle.usedPct - start.usedPct) / (middle.t - start.t),
    (end.usedPct - middle.usedPct) / (end.t - middle.t),
  ];
  const fastest = Math.max(...rates);
  if (fastest === 0) return null;
  const exhaustionAt = (rate: number) => end.t + (100 - end.usedPct) / rate;
  const earliest = exhaustionAt(fastest);
  if (earliest <= nowSecs || earliest >= resetsAt) return null;
  const latest = exhaustionAt(Math.min(...rates));
  return {
    earliestMin: (earliest - nowSecs) / 60,
    latestMin: latest < resetsAt ? (latest - nowSecs) / 60 : null,
  };
}
