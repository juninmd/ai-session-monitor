// The trend chart. It is the burndown the current interface already draws, at a size that can be
// read, plus the one thing the numbers cannot say at a glance: where the current burn lands if it
// keeps its rate. Every fact it draws is also printed as a sentence next to it; the line under it
// reads the running pass at the moment the pointer or the scrubber rests on.
import { type PointerEvent, useId, useState } from 'react';
import type { Burndown } from '../app/model/burndown';
import { durationFormat, type ViewContext, type WindowModel } from '../app/model/windowModel';
import { t } from '../i18n';
import { formatDuration, formatPercent } from '../i18n/format';
import { nowPoint, projectionLabel, projectionOf, type Segment } from './historyModel';
import { bandOf, clampMoment, domainOf, labelSpot, valueAt } from './trendChartModel';

const W = 340;
const H = 172;
const LEFT = 34;
const RIGHT = 14;
const TOP = 18;
const BOTTOM = 26;
const PLOT_W = W - LEFT - RIGHT;
const GRID = [0, 0.25, 0.5, 0.75, 1] as const;
const LABELLED = new Set<number>([0, 0.5, 1]);

const px = (x: number) => LEFT + x * PLOT_W;
const py = (y: number) => H - BOTTOM - y * (H - TOP - BOTTOM);
const at = (p: Segment) => `${px(p.x).toFixed(1)},${py(p.y).toFixed(1)}`;
const path = (points: readonly Segment[]) => points.map(at).join(' ');

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
  const [cursor, setCursor] = useState<number | null>(null);
  const { current, past } = burndown;
  const now = live ? nowPoint(model) : null;
  const projection = now === null ? null : projectionOf(model, now);
  const plot = current?.plot ?? [];
  const end = plot.at(-1) ?? null;
  const domain = domainOf(plot);
  const band = bandOf(past);
  const area =
    end === null
      ? ''
      : `${path(plot)} ${px(end.x).toFixed(1)},${py(0)} ${px(plot[0]?.x ?? 0)},${py(0)}`;
  const bandPoints = [
    ...band.map((point) => at({ x: point.x, y: point.high })),
    ...band.toReversed().map((point) => at({ x: point.x, y: point.low })),
  ].join(' ');

  // The moment being read: where the pointer or the scrubber rests, else the latest reading.
  const moment =
    cursor !== null && domain !== null ? clampMoment(domain, cursor) : (end?.x ?? null);
  const read = moment === null ? null : (valueAt(plot, moment) ?? end?.y ?? null);
  const readout =
    moment === null || read === null
      ? null
      : t('history.inspect.reading', {
          pct: formatPercent(read * 100),
          elapsed: formatDuration(
            (moment * burndown.lengthSecs) / 60,
            durationFormat(ctx.settings),
            model.long,
          ),
        });
  const follow = (event: PointerEvent<HTMLDivElement>) => {
    const frame = event.currentTarget.getBoundingClientRect();
    if (frame.width <= 0) return;
    setCursor((((event.clientX - frame.left) / frame.width) * W - LEFT) / PLOT_W);
  };
  const spot = now === null ? null : labelSpot(now, projection !== null);

  return (
    <div className="nx-chart-wrap">
      <div className="nx-chart-stage" onPointerMove={follow} onPointerLeave={() => setCursor(null)}>
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
              : `${t('history.chart_desc')} ${t('history.trend.projection_desc', { duration: projectionLabel(projection.minutes, model, ctx) })}`}
          </desc>
          <defs>
            <linearGradient id={`${id}-fill`} x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" className="nx-chart-stop-a" />
              <stop offset="100%" className="nx-chart-stop-b" />
            </linearGradient>
          </defs>
          {GRID.map((g) => (
            <g key={g}>
              <line
                className="nx-chart-grid"
                data-minor={!LABELLED.has(g) || undefined}
                x1={LEFT}
                x2={W - RIGHT}
                y1={py(g)}
                y2={py(g)}
              />
              {LABELLED.has(g) && (
                <text
                  className="nx-chart-txt"
                  x={LEFT - 6}
                  y={py(g)}
                  textAnchor="end"
                  dominantBaseline="middle"
                >
                  {formatPercent(g * 100)}
                </text>
              )}
            </g>
          ))}
          <text className="nx-chart-txt" x={LEFT} y={H - 7}>
            {t('history.axis_start')}
          </text>
          <text className="nx-chart-txt" x={W - RIGHT} y={H - 7} textAnchor="end">
            {t('history.axis_end')}
          </text>
          {band.length > 0 && <polygon className="nx-chart-band" points={bandPoints} />}
          {past.map((curve) => (
            <polyline key={curve.resetsAt} className="nx-chart-past" points={path(curve.plot)} />
          ))}
          {current !== null && (
            <>
              <polygon className="nx-chart-area" points={area} fill={`url(#${id}-fill)`} />
              <polyline className="nx-chart-cur" points={path(plot)} pathLength={1} />
            </>
          )}
          {projection !== null && now !== null && (
            <>
              <polyline className="nx-chart-proj" points={`${at(now)} ${at(projection.to)}`} />
              <circle
                className="nx-chart-cross"
                cx={px(projection.to.x)}
                cy={py(projection.to.y)}
                r={2.5}
              />
              <text
                className="nx-chart-lbl nx-chart-lbl-proj"
                x={Math.min(px(projection.to.x), W - RIGHT)}
                y={py(projection.to.y) - 7}
                textAnchor={projection.to.x > 0.85 ? 'end' : 'middle'}
              >
                {projectionLabel(projection.minutes, model, ctx)}
              </text>
            </>
          )}
          {now !== null && spot !== null && (
            <>
              <line className="nx-chart-now" x1={px(now.x)} x2={px(now.x)} y1={py(1)} y2={py(0)} />
              <circle className="nx-chart-dot" cx={px(now.x)} cy={py(now.y)} r={3.5} />
              <text
                className="nx-chart-lbl"
                x={px(now.x) + spot.dx}
                y={py(now.y) + spot.dy}
                textAnchor={spot.anchor}
              >
                {formatPercent(model.shown)}
              </text>
            </>
          )}
          {cursor !== null && moment !== null && read !== null && (
            <>
              <line
                className="nx-chart-cursor"
                x1={px(moment)}
                x2={px(moment)}
                y1={py(1)}
                y2={py(0)}
              />
              <circle className="nx-chart-cursor-dot" cx={px(moment)} cy={py(read)} r={3} />
            </>
          )}
        </svg>
        {domain !== null && (
          <input
            type="range"
            className="nx-chart-scrub"
            dir="ltr"
            min={0}
            max={100}
            step={1}
            value={Math.round((moment ?? domain.to) * 100)}
            aria-label={t('history.inspect.label')}
            aria-valuetext={readout ?? undefined}
            style={{
              insetInlineStart: `${(LEFT / W) * 100}%`,
              inlineSize: `${(PLOT_W / W) * 100}%`,
            }}
            onChange={(event) => setCursor(Number(event.currentTarget.value) / 100)}
            onBlur={() => setCursor(null)}
          />
        )}
      </div>
      {readout !== null && (
        <p className="nx-chart-readout" data-live={cursor !== null || undefined}>
          <i aria-hidden="true" />
          {readout}
        </p>
      )}
    </div>
  );
}
