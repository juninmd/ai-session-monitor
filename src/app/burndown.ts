// Turns stored epochs into what the burndown draws and says: x is the share of the window that
// has elapsed, y the used percentage. Pure, so the chart and its text come from the same numbers.
import type { HistoryEpoch, HistoryPoint, UsageWindow } from '../lib/contract';

/** Providers re-state a reset a few seconds apart; windows are far longer than this. */
const SAME_EPOCH_SECS = 120;
const MIN_LENGTH_SECS = 3600;
const KIND_MINUTES: Partial<Record<UsageWindow['kind'], number>> = {
  five_hour: 300,
  daily: 1440,
  weekly: 10_080,
  monthly: 43_200,
};

export interface Curve {
  resetsAt: number;
  /** x and y in 0..1 (y is the used share). Oldest first. */
  plot: { x: number; y: number }[];
  peakPct: number;
  peakAt: number;
  readings: number;
}

export interface Burndown {
  lengthSecs: number;
  /** The pass that ends at the window's current reset; null when it has no reading yet. */
  current: Curve | null;
  /** Earlier passes, oldest first. */
  past: Curve[];
}

function lengthOf(window: UsageWindow, epochs: readonly HistoryEpoch[]): number {
  const minutes = window.windowMinutes ?? KIND_MINUTES[window.kind];
  if (minutes !== undefined && minutes > 0) return minutes * 60;
  const spans = epochs.map((epoch) => epoch.resetsAt - (epoch.points[0]?.t ?? epoch.resetsAt));
  return Math.max(MIN_LENGTH_SECS, ...spans);
}

function peakOf(points: readonly HistoryPoint[]): HistoryPoint | null {
  let peak: HistoryPoint | null = null;
  for (const point of points) if (peak === null || point.usedPct > peak.usedPct) peak = point;
  return peak;
}

function curveOf(epoch: HistoryEpoch, lengthSecs: number): Curve | null {
  const peak = peakOf(epoch.points);
  if (peak === null) return null;
  const start = epoch.resetsAt - lengthSecs;
  const unit = (value: number) => Math.min(1, Math.max(0, value));
  return {
    resetsAt: epoch.resetsAt,
    plot: epoch.points.map((p) => ({
      x: unit((p.t - start) / lengthSecs),
      y: unit(p.usedPct / 100),
    })),
    peakPct: peak.usedPct,
    peakAt: peak.t,
    readings: epoch.points.length,
  };
}

/** Null when the window has no stored epoch at all (the empty state). */
export function burndownOf(window: UsageWindow, epochs: readonly HistoryEpoch[]): Burndown | null {
  const mine = epochs.filter((epoch) => epoch.windowId === window.id && epoch.points.length > 0);
  if (mine.length === 0) return null;
  const lengthSecs = lengthOf(window, mine);
  const isCurrent = (epoch: HistoryEpoch) =>
    window.resetsAt !== null && Math.abs(epoch.resetsAt - window.resetsAt) <= SAME_EPOCH_SECS;
  const curves = (list: HistoryEpoch[]) =>
    list
      .sort((a, b) => a.resetsAt - b.resetsAt)
      .flatMap((epoch) => curveOf(epoch, lengthSecs) ?? []);
  return {
    lengthSecs,
    current: curves(mine.filter(isCurrent)).at(-1) ?? null,
    past: curves(mine.filter((epoch) => !isCurrent(epoch))),
  };
}
