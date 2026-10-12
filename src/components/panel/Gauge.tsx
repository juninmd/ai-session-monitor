import type { CSSProperties } from 'react';
import { gaugeLabel, gaugeTip, type ViewContext, type WindowModel } from '../../app/windowModel';

interface GaugeProps {
  model: WindowModel;
  ctx: ViewContext;
  size?: 'md' | 'sm';
  /** Accessible name when the window title alone is ambiguous (the same title in two pools). */
  title?: string;
}

/**
 * Fill = the share printed beside it (available, or used under `showUsed`); the tick marks the share of the window that is left. An unknown
 * or already-reset window draws an empty hatched track, never a 0 % meter.
 */
export function Gauge({ model, ctx, size = 'md', title }: GaugeProps) {
  const label = gaugeLabel(model, title);
  const { warnAtPct } = ctx.settings.general;
  const tickStyle =
    model.tick === null ? undefined : ({ '--x': `${model.tick.toFixed(1)}%` } as CSSProperties);
  return (
    <span className={size === 'sm' ? 'gauge sm' : 'gauge'} data-status={model.status ?? 'none'}>
      {model.shown === null || model.expired ? (
        <span
          className="gauge-void"
          data-kind={model.expired ? 'expired' : 'unknown'}
          role="img"
          aria-label={label}
        />
      ) : (
        <meter
          min={0}
          max={100}
          low={model.showUsed ? 100 - warnAtPct : 10}
          high={model.showUsed ? 90 : warnAtPct}
          optimum={model.showUsed ? 0 : 100}
          value={Math.round(model.shown)}
          aria-label={label}
        />
      )}
      {tickStyle && <i className="tick" style={tickStyle} />}
      <span className="tip" aria-hidden="true">
        {gaugeTip(model, ctx)}
      </span>
    </span>
  );
}
