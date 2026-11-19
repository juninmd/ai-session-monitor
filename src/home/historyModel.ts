// What the history board says about a window: how this pass compares with the earlier ones, and
// where the current burn is heading. Pure, so the sentences and the chart come from one place.
//
// Compare readings at the same elapsed share of the quota window. The projection stays anchored
// at the live reading so the line the chart draws is the line `paceOf` decided.
import type { Burndown } from '../app/model/burndown';
import { durationFormat, type ViewContext, type WindowModel } from '../app/model/windowModel';
import { t } from '../i18n';
import { formatDuration, formatPercent } from '../i18n/format';
import { valueAt } from './trendChartModel';

export type TrendKind = 'more' | 'less' | 'similar' | 'first';

export interface Trend {
  kind: TrendKind;
  currentUsed: number | null;
  /** Median estimated usage at the same elapsed share, in percent. */
  pastMedian: number | null;
  pastCount: number;
  label: string;
}

export function trendOf(burndown: Burndown): Trend {
  const now = lastPoint(burndown);
  if (now === null) {
    return {
      currentUsed: null,
      pastMedian: null,
      pastCount: 0,
      kind: 'first',
      label: t('history.trend.none'),
    };
  }
  const past = burndown.past
    .flatMap((curve) => valueAt(curve.plot, now.x) ?? [])
    .sort((a, b) => a - b);
  const currentUsed = now.y * 100;
  const middle = Math.floor(past.length / 2);
  const upper = past[middle];
  if (upper === undefined) {
    return {
      currentUsed,
      pastMedian: null,
      pastCount: 0,
      kind: 'first',
      label: t('history.trend.no_comparison', { used: formatPercent(currentUsed) }),
    };
  }
  const lower = past.length % 2 === 0 ? (past[middle - 1] ?? upper) : upper;
  const pastMedian = (lower + upper) * 50;
  const delta = currentUsed - pastMedian;
  const kind: TrendKind = Math.abs(delta) < 5 ? 'similar' : delta > 0 ? 'more' : 'less';
  return {
    currentUsed,
    pastMedian,
    pastCount: past.length,
    kind,
    label: t('history.trend.same_stage', {
      used: formatPercent(currentUsed),
      median: formatPercent(pastMedian),
    }),
  };
}

export interface Segment {
  x: number;
  y: number;
}

/** The last reading of the current pass, which the chart marks as its end. */
export function lastPoint(burndown: Burndown): Segment | null {
  const plot = burndown.current?.plot ?? [];
  return plot.at(-1) ?? null;
}

/** Where the live reading sits: the share of the window already elapsed, and the used share. */
export function nowPoint(model: WindowModel): Segment | null {
  if (model.expired) return null;
  const { usedPct, windowMinutes } = model.window;
  if (usedPct === null || windowMinutes === null || windowMinutes <= 0) return null;
  if (model.leftMin === null) return null;
  const elapsed = Math.min(1, Math.max(0, (windowMinutes - model.leftMin) / windowMinutes));
  return { x: elapsed, y: Math.min(1, Math.max(0, usedPct / 100)) };
}

/**
 * Where the current burn lands if it keeps its rate: a straight line from the live reading to
 * 100 % used. Null when the quota reaches the renewal first, or when there is nothing to project
 * from — the warning comes from `paceOf`, this only draws it.
 */
export function projectionOf(
  model: WindowModel,
  now: Segment,
): { to: Segment; minutes: number } | null {
  if (model.pace.kind !== 'risk') return null;
  const lengthMin = model.window.windowMinutes ?? 0;
  if (lengthMin <= 0) return null;
  const x = now.x + model.pace.runwayMin / lengthMin;
  if (!Number.isFinite(x) || x >= 1) return null;
  return { to: { x, y: 1 }, minutes: model.pace.runwayMin };
}

/** "1h 12m" — how long the projection says is left before it runs out. */
export function projectionLabel(runwayMin: number, model: WindowModel, ctx: ViewContext): string {
  return formatDuration(runwayMin, durationFormat(ctx.settings), model.long);
}
