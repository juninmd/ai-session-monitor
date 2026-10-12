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
 * Fill = what is still AVAILABLE; the tick marks the share of the window that is left. An unknown
 * or already-reset window draws an empty hatched track, never a 0 % meter.
 */
export function Gauge({ model, ctx, size = 'md', title }: GaugeProps) {
  const label = gaugeLabel(model, title);
  const tickStyle =
    model.tick === null ? undefined : ({ '--x': `${model.tick.toFixed(1)}%` } as CSSProperties);
  return (
    <span className={size === 'sm' ? 'gauge sm' : 'gauge'} data-status={model.status ?? 'none'}>
      {model.available === null || model.expired ? (
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
          low={10}
          high={ctx.settings.general.warnAtPct}
          optimum={100}
          value={Math.round(model.available)}
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
