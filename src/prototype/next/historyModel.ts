// What the history board says about a window: how this pass compares with the earlier ones, and
// where the current burn is heading. Pure, so the sentences and the chart come from one place.
//
// Two honesty rules live here. The comparison states two measured facts (this pass's peak so far,
// and the mean of the passes that closed) instead of calling a still-running pass above or below
// them, because an unfinished pass has necessarily peaked lower than a finished one. And the
// projection is anchored at the live reading, not at the last stored one, so the line the chart
// draws is the line `paceOf` decided.
import type { Burndown } from '../../app/burndown';
import { durationFormat, type ViewContext, type WindowModel } from '../../app/windowModel';
import { formatDuration, formatPercent } from '../../i18n/format';

export type TrendKind = 'more' | 'less' | 'similar' | 'first';

export interface Trend {
  kind: TrendKind;
  /** The peak of the pass still running, when it has one. */
  peakNow: number | null;
  /** Mean peak of the passes that already closed; null with none. */
  pastAverage: number | null;
  pastCount: number;
  label: string;
}

function mean(values: readonly number[]): number | null {
  if (values.length === 0) return null;
  return values.reduce((total, value) => total + value, 0) / values.length;
}

export function trendOf(burndown: Burndown): Trend {
  const peakNow = burndown.current?.peakPct ?? null;
  const past = burndown.past.map((curve) => curve.peakPct);
  const pastAverage = mean(past);
  const base = { peakNow, pastAverage, pastCount: past.length };
  if (peakNow === null) {
    return { ...base, kind: 'first', label: 'Sem leitura nesta janela.' };
  }
  if (pastAverage === null) {
    return {
      ...base,
      kind: 'first',
      label: `Pico de ${formatPercent(peakNow)} até agora, sem renovação anterior para comparar.`,
    };
  }
  const delta = peakNow - pastAverage;
  const kind: TrendKind = Math.abs(delta) < 5 ? 'similar' : delta > 0 ? 'more' : 'less';
  const closed =
    past.length === 1
      ? 'a renovação anterior fechou'
      : `as ${past.length} renovações anteriores fecharam`;
  return {
    ...base,
    kind,
    label: `Pico de ${formatPercent(peakNow)} até agora · ${closed} em média em ${formatPercent(pastAverage)}.`,
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
