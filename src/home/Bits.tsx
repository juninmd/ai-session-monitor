// Small shared visual pieces of the proposed interface. Everything here reuses the existing
// design tokens and the shared `.t-bar` bar, so nothing is a second design system.
import { HarnessLogo } from '../components/brand/HarnessLogo';
import type { HarnessId } from '../lib/contract';
import type { Status } from '../lib/usage';

export type Tone = Status | 'none';

/** The bar the rows and cards share: fill = the printed share, hatch = unknown. */
export function ShareBar({ pct, tone, label }: { pct: number | null; tone: Tone; label?: string }) {
  return (
    <span className={`t-bar nx-bar${pct === null ? ' off' : ''}`} data-status={tone}>
      {pct !== null && <i style={{ inlineSize: `${Math.min(100, Math.max(0, pct))}%` }} />}
      <span className="vh">{label ?? ''}</span>
    </span>
  );
}

/** The reading as a ring around the card's number: the same share the bars print, drawn once. */
export function Ring({ pct, tone }: { pct: number | null; tone: Tone }) {
  return (
    <svg className="nx-ring" viewBox="0 0 20 20" aria-hidden="true" data-status={tone}>
      <circle className="nx-ring-track" cx="10" cy="10" r="8" pathLength="100" />
      {pct !== null && (
        <circle
          className="nx-ring-fill"
          cx="10"
          cy="10"
          r="8"
          pathLength="100"
          style={{ strokeDasharray: `${Math.min(100, Math.max(0, pct))} 100` }}
        />
      )}
    </svg>
  );
}

export function Logo({ id, size = 18 }: { id: HarnessId; size?: number }) {
  return <HarnessLogo id={id} size={size} />;
}

/** A real heading, so the grouping the surface is built on survives in the accessibility tree. */
export function GroupHead({ title, count }: { title: string; count: number }) {
  return (
    <h3 className="nx-section-head">
      {title}
      <span className="nx-count">{count}</span>
    </h3>
  );
}
