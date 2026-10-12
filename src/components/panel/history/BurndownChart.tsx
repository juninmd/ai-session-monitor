import { useId } from 'react';
import type { Burndown, Curve } from '../../../app/burndown';
import { t } from '../../../i18n';
import { formatPercent } from '../../../i18n/format';

const W = 320;
const H = 132;
const LEFT = 32;
const RIGHT = 56;
const TOP = 10;
const BOTTOM = 24;
const GRID = [0, 0.5, 1] as const;

const px = (x: number) => LEFT + x * (W - LEFT - RIGHT);
const py = (y: number) => H - BOTTOM - y * (H - TOP - BOTTOM);
const pointsOf = (curve: Curve) =>
  curve.plot.map((p) => `${px(p.x).toFixed(1)},${py(p.y).toFixed(1)}`).join(' ');

/**
 * Used share over the elapsed share of the window: one hue, the current pass heavy with a direct
 * label, the earlier ones thin. Meaning never rests on colour alone; the text alternative is the
 * title and description here plus the sentences and the table next to it.
 */
export function BurndownChart({ burndown, title }: { burndown: Burndown; title: string }) {
  const id = useId();
  const { current, past } = burndown;
  const last = current?.plot.at(-1);
  const newestPast = past.at(-1)?.plot.at(-1);
  return (
    <svg
      className="bd"
      viewBox={`0 0 ${W} ${H}`}
      role="img"
      aria-labelledby={`${id}-t ${id}-d`}
      focusable="false"
    >
      <title id={`${id}-t`}>{t('history.chart_title', { title })}</title>
      <desc id={`${id}-d`}>{t('history.chart_desc')}</desc>
      {GRID.map((g) => (
        <g key={g}>
          <line className="bd-grid" x1={LEFT} x2={W - RIGHT} y1={py(g)} y2={py(g)} />
          <text
            className="bd-txt"
            x={LEFT - 4}
            y={py(g)}
            textAnchor="end"
            dominantBaseline="middle"
          >
            {formatPercent(g * 100)}
          </text>
        </g>
      ))}
      <text className="bd-txt" x={LEFT} y={H - 6}>
        {t('history.axis_start')}
      </text>
      <text className="bd-txt" x={W - RIGHT} y={H - 6} textAnchor="end">
        {t('history.axis_end')}
      </text>
      {past.map((curve) => (
        <polyline key={curve.resetsAt} className="bd-past" points={pointsOf(curve)} />
      ))}
      {newestPast && (
        <text className="bd-txt" x={px(newestPast.x) + 4} y={py(newestPast.y) - 4}>
          {t('history.label_past')}
        </text>
      )}
      {current && <polyline className="bd-cur" points={pointsOf(current)} />}
      {current && last && (
        <>
          <circle className="bd-dot" cx={px(last.x)} cy={py(last.y)} r={3} />
          <text className="bd-lbl" x={px(last.x) + 6} y={py(last.y) + 3.5}>
            {t('history.label_current', { pct: formatPercent(current.peakPct) })}
          </text>
        </>
      )}
    </svg>
  );
}
