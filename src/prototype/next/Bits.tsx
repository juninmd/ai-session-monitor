// Small shared visual pieces of the proposed interface. Everything here reuses the existing
// design tokens and the `.t-bar` bar from the current UI, so nothing is a second design system.
import { HarnessLogo } from '../../components/brand/HarnessLogo';
import type { HarnessId } from '../../lib/contract';
import type { Status } from '../../lib/usage';

export type Tone = Status | 'none';

/** The bar shipped with the tab strip: fill = the printed share, hatch = unknown. */
export function ShareBar({ pct, tone, label }: { pct: number | null; tone: Tone; label?: string }) {
  return (
    <span className={`t-bar nx-bar${pct === null ? ' off' : ''}`} data-status={tone}>
      {pct !== null && <i style={{ inlineSize: `${Math.min(100, Math.max(0, pct))}%` }} />}
      <span className="vh">{label ?? ''}</span>
    </span>
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
