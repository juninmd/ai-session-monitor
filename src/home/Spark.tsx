// A trend line for one window, built from the stored epochs the burndown already reads. It is
// decorative: every number it draws is also printed as text next to it, and it renders nothing
// while there are fewer than two readings to join. The caller decides when to ask for history.
import { burndownOf } from '../app/burndown';
import { useHistory } from '../app/useHistory';
import type { HarnessView, UsageWindow } from '../lib/contract';

function line(points: readonly { x: number; y: number }[]): string {
  return points
    .map((point) => `${(point.x * 100).toFixed(1)},${(100 - point.y * 100).toFixed(1)}`)
    .join(' ');
}

export function Spark({
  view,
  window,
  label,
}: {
  view: HarnessView;
  window: UsageWindow | null;
  label: string;
}) {
  const active = window !== null && view.readAt !== null;
  const state = useHistory(view.id, active, view.readAt);
  const burndown =
    state.status === 'ready' && window !== null ? burndownOf(window, state.history.epochs) : null;
  const points = burndown?.current?.plot ?? [];
  if (points.length < 2) return null;
  return (
    <span className="nx-spark" role="img" aria-label={label}>
      <svg viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
        <polyline
          points={line(points)}
          vectorEffect="non-scaling-stroke"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.5"
        />
      </svg>
    </span>
  );
}
