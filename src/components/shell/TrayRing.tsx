import type { Status } from '../../lib/usage';

const RADIUS = 6;
const CIRCUMFERENCE = 2 * Math.PI * RADIUS;

interface TrayRingProps {
  /** Available percentage of the lowest window, or null when nothing is readable. */
  pct: number | null;
  status: Status | null;
}

/** The tray icon's arc, repeated in the header: the arc is the available share. */
export function TrayRing({ pct, status }: TrayRingProps) {
  const known = pct !== null;
  const arc = known ? (Math.min(100, Math.max(0, pct)) / 100) * CIRCUMFERENCE : 0;
  return (
    <svg
      className="ring-ico"
      viewBox="0 0 16 16"
      fill="none"
      strokeWidth={2.5}
      aria-hidden="true"
      focusable="false"
      data-status={status ?? 'none'}
    >
      <circle
        className="ring-bg"
        cx={8}
        cy={8}
        r={RADIUS}
        strokeDasharray={known ? undefined : '2.5 2.5'}
      />
      {known && (
        <circle
          className="ring-fg"
          cx={8}
          cy={8}
          r={RADIUS}
          strokeLinecap="round"
          strokeDasharray={`${arc.toFixed(1)} ${CIRCUMFERENCE.toFixed(1)}`}
          transform="rotate(-90 8 8)"
        />
      )}
    </svg>
  );
}
