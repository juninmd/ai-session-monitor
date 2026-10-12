// Decides how a ready view's windows are laid out, from the data alone (no per-harness markup):
// the first ungrouped window is the hero, the rest are flat sections, grouped windows become
// pool cards (Antigravity) or one per-model list when every group has a single window (Claude).
import { hasMessage, t } from '../i18n';
import type { HarnessView, UsageWindow, WindowKind } from '../lib/contract';
import { modelWindow, rowTitle, type ViewContext, type WindowModel } from './windowModel';

export interface PoolRow {
  name: string;
  model: WindowModel;
}

export interface Pool {
  id: string;
  title: string;
  note: string | null;
  rows: PoolRow[];
}

export interface WindowLayout {
  hero: WindowModel | null;
  flats: WindowModel[];
  /** One list of single-window groups ("Semana por modelo"). */
  groupList: { title: string; rows: PoolRow[] } | null;
  pools: Pool[];
}

// Antigravity names a group it could not name `group_<index>`, counted from 0.
const UNNAMED_GROUP = /^group_(\d{1,3})$/;

export function groupName(group: string): string {
  const key = `group.${group}`;
  if (hasMessage(key)) return t(key);
  const index = UNNAMED_GROUP.exec(group)?.[1];
  if (index !== undefined) return t('group.numbered', { n: Number(index) + 1 });
  return group.charAt(0).toUpperCase() + group.slice(1).replaceAll('_', ' ');
}

function groupNote(group: string): string | null {
  const key = `group.note.${group}`;
  return hasMessage(key) ? t(key) : null;
}

function listTitle(kinds: readonly WindowKind[]): string {
  const first = kinds[0];
  const same = first !== undefined && kinds.every((k) => k === first);
  if (same && first === 'weekly') return t('window.by_group.weekly');
  if (same && first === 'monthly') return t('window.by_group.monthly');
  if (same && first === 'budget') return t('window.by_group.budget');
  return t('window.by_group.default');
}

function rowName(window: UsageWindow, group: string, single: boolean): string {
  if (window.label) return window.label;
  return single ? groupName(group) : rowTitle(window);
}

export function layoutWindows(view: HarnessView, ctx: ViewContext): WindowLayout {
  const models = view.windows.map((w) => modelWindow(view.id, w, ctx));
  const plain = models.filter((m) => m.window.group === null);
  const byGroup = new Map<string, WindowModel[]>();
  for (const m of models) {
    const group = m.window.group;
    if (group === null) continue;
    byGroup.set(group, [...(byGroup.get(group) ?? []), m]);
  }
  const groups = [...byGroup.entries()];
  const [hero = null, ...flats] = plain;

  if (groups.length > 0 && groups.every(([, list]) => list.length === 1)) {
    const rows = groups.flatMap(([group, list]) =>
      list.map((model) => ({ name: rowName(model.window, group, true), model })),
    );
    const kinds = rows.map((r) => r.model.window.kind);
    return { hero, flats, groupList: { title: listTitle(kinds), rows }, pools: [] };
  }

  const pools = groups.map(([group, list]) => ({
    id: group,
    title: groupName(group),
    note: groupNote(group),
    rows: list.map((model) => ({ name: rowName(model.window, group, false), model })),
  }));
  return { hero, flats, groupList: null, pools };
}
