import type { HarnessId, HarnessView, Snapshot } from '../lib/contract';
import { TAB_ORDER } from './harness';

export function placeholderView(id: HarnessId): HarnessView {
  return {
    id,
    state: 'loading',
    source: null,
    readAt: null,
    stale: false,
    plan: null,
    windows: [],
    facts: [],
    notices: [],
    error: null,
    meta: {},
  };
}

/** One view per tab, in tab order; a harness the snapshot does not mention reads as loading. */
export function viewsInTabOrder(snapshot: Snapshot | null): HarnessView[] {
  return TAB_ORDER.map(
    (id) => snapshot?.views.find((view) => view.id === id) ?? placeholderView(id),
  );
}

/** The newest `readAt` of any view: when something was last actually read. */
export function latestReadAt(views: readonly HarnessView[]): number | null {
  let latest: number | null = null;
  for (const view of views) {
    if (view.readAt !== null && (latest === null || view.readAt > latest)) latest = view.readAt;
  }
  return latest;
}
