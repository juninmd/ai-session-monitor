// The trend chart. It is the burndown the current interface already draws, at a size that can be
// read, plus the one thing the numbers cannot say at a glance: where the current burn lands if it
// keeps its rate. Every fact it draws is also printed as a sentence next to it.
import { useId } from 'react';
import type { Burndown } from '../app/burndown';
import type { ViewContext, WindowModel } from '../app/windowModel';
import { t } from '../i18n';
import { formatPercent } from '../i18n/format';
import { nowPoint, projectionLabel, projectionOf, type Segment } from './historyModel';

const W = 340;
const H = 168;
const LEFT = 34;
const RIGHT = 66;
const TOP = 12;
const BOTTOM = 26;
const GRID = [0, 0.5, 1] as const;

const px = (x: number) => LEFT + x * (W - LEFT - RIGHT);
const py = (y: number) => H - BOTTOM - y * (H - TOP - BOTTOM);
const path = (points: readonly Segment[]) =>
  points.map((p) => `${px(p.x).toFixed(1)},${py(p.y).toFixed(1)}`).join(' ');

export function TrendChart({
  burndown,
  model,
  ctx,
  title,
  live,
}: {
  burndown: Burndown;
  model: WindowModel;
  ctx: ViewContext;
  title: string;
  /** False for a stale reading: then there is no "now" to anchor a projection at. */
  live: boolean;
}) {
  const id = useId();
  const { current, past } = burndown;
  const now = live ? nowPoint(model) : null;
  const projection = now === null ? null : projectionOf(model, now);
  const end = current?.plot.at(-1) ?? null;
  const area =
    current === null || end === null
      ? ''
      : `${path(current.plot)} ${px(end.x).toFixed(1)},${py(0)} ${px(0)},${py(0)}`;

  return (
    <svg
      className="nx-chart"
      viewBox={`0 0 ${W} ${H}`}
      role="img"
      aria-labelledby={`${id}-t ${id}-d`}
      focusable="false"
    >
      <title id={`${id}-t`}>{t('history.chart_title', { title })}</title>
      <desc id={`${id}-d`}>
        {projection === null
          ? t('history.chart_desc')
          : `${t('history.chart_desc')} A projeção segue o ritmo atual até acabar em ${projectionLabel(projection.minutes, model, ctx)}.`}
      </desc>
      <defs>
        <linearGradient id={`${id}-fill`} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" className="nx-chart-stop-a" />
          <stop offset="100%" className="nx-chart-stop-b" />
        </linearGradient>
      </defs>
      {GRID.map((g) => (
        <g key={g}>
          <line className="nx-chart-grid" x1={LEFT} x2={W - RIGHT} y1={py(g)} y2={py(g)} />
          <text
            className="nx-chart-txt"
            x={LEFT - 5}
            y={py(g)}
            textAnchor="end"
            dominantBaseline="middle"
          >
            {formatPercent(g * 100)}
          </text>
        </g>
      ))}
      <text className="nx-chart-txt" x={LEFT} y={H - 7}>
        {t('history.axis_start')}
      </text>
      <text className="nx-chart-txt" x={W - RIGHT} y={H - 7} textAnchor="end">
        {t('history.axis_end')}
      </text>
      {past.map((curve) => (
        <polyline key={curve.resetsAt} className="nx-chart-past" points={path(curve.plot)} />
      ))}
      {current !== null && (
        <>
          <polygon className="nx-chart-area" points={area} fill={`url(#${id}-fill)`} />
          <polyline className="nx-chart-cur" points={path(current.plot)} />
        </>
      )}
      {projection !== null && now !== null && (
        <>
          <polyline
            className="nx-chart-proj"
            points={`${px(now.x).toFixed(1)},${py(now.y).toFixed(1)} ${px(projection.to.x).toFixed(1)},${py(projection.to.y).toFixed(1)}`}
          />
          <circle
            className="nx-chart-cross"
            cx={px(projection.to.x)}
            cy={py(projection.to.y)}
            r={2.5}
          />
        </>
      )}
      {now !== null && (
        <>
          <line className="nx-chart-now" x1={px(now.x)} x2={px(now.x)} y1={py(1)} y2={py(0)} />
          <circle className="nx-chart-dot" cx={px(now.x)} cy={py(now.y)} r={3.5} />
          <text className="nx-chart-lbl" x={px(now.x) + 8} y={py(now.y) + 4}>
            {formatPercent(model.shown)}
          </text>
        </>
      )}
      {projection !== null && (
        <text
          className="nx-chart-lbl nx-chart-lbl-proj"
          x={px(projection.to.x)}
          y={py(projection.to.y) + 15}
          textAnchor="middle"
        >
          {projectionLabel(projection.minutes, model, ctx)}
        </text>
      )}
    </svg>
  );
}
