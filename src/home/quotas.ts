// The quota sheet of one provider: every window it reports, with its pool, in the order the
// source gives them. Nothing is merged and nothing is capped: this is the audit view, so a quota
// that never fits a card still appears here.
import { groupName } from '../app/layout';
import { modelWindow, type ViewContext, type WindowModel } from '../app/windowModel';
import { hasMessage, t } from '../i18n';
import type { HarnessView, UsageWindow } from '../lib/contract';

export interface SheetGroup {
  /** Null for the windows that belong to no pool. */
  id: string | null;
  /** The pool's own heading, or null when the windows are independent. */
  title: string | null;
  /** Why the pool is separate, when the catalog explains it. */
  note: string | null;
  windows: WindowModel[];
}

export interface QuotaSheet {
  groups: SheetGroup[];
  /** Every window the view reports, including the ones with no reading. */
  total: number;
  /** Windows that carry a number, which is what the history can chart. */
  readable: number;
}

/** The pool's own explanation, the same one the current interface prints under the heading. */
function poolNote(id: string): string | null {
  const key = `group.note.${id}`;
  return hasMessage(key) ? t(key) : null;
}

export function sheetOf(view: HarnessView, ctx: ViewContext): QuotaSheet {
  const byGroup = new Map<string | null, WindowModel[]>();
  for (const window of view.windows) {
    const model = modelWindow(view.id, window, ctx);
    byGroup.set(window.group, [...(byGroup.get(window.group) ?? []), model]);
  }
  const groups: SheetGroup[] = [...byGroup.entries()].map(([id, windows]) => ({
    id,
    title: id === null ? null : groupName(id),
    note: id === null ? null : poolNote(id),
    windows,
  }));
  // Pools first: a grouped quota is only meaningful next to the pool it shares.
  groups.sort((a, b) => Number(a.id === null) - Number(b.id === null));
  return {
    groups,
    total: view.windows.length,
    readable: view.windows.filter(hasReading).length,
  };
}

export function hasReading(window: UsageWindow): boolean {
  return window.usedPct !== null && window.resetsAt !== null;
}

/** The windows a chart or a card can use: a reading with a reset to hang it on. */
export function readableWindows(view: HarnessView): UsageWindow[] {
  return view.windows.filter(hasReading);
}
