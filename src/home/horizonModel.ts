// Horizon: puts every provider on the same question — "do I reach the renewal, or is there a
// stretch with no quota?" Each bar runs from now to that window's reset, and the fill is the share
// of that stretch the remaining quota can cover. The rest of the track is time with no quota. The
// projections come from paceOf via WindowModel.pace; nothing here recomputes a status.
import { summaryOf } from '../app/model/tabModel';
import {
  durationFormat,
  modelWindow,
  type ViewContext,
  type WindowModel,
} from '../app/model/windowModel';
import { t } from '../i18n';
import { formatClock, formatDuration } from '../i18n/format';
import type { HarnessId, HarnessView, UsageWindow } from '../lib/contract';
import { isExpired, paceOf } from '../lib/usage';
import { lostSetup } from './model';
import { quotaName } from './wording';

export type HorizonKind = 'lasts' | 'short' | 'out' | 'unknown' | 'broken';

export interface HorizonRow {
  view: HarnessView;
  model: WindowModel | null;
  windowName: string | null;
  /** 0..1 of the stretch to the reset the quota covers; 1 when it lasts, null when unknown. */
  cover: number | null;
  /** Minutes to the reset of that window; null without a reset time. */
  leftMin: number | null;
  /** Minutes of that stretch with no quota left; 0 when it lasts. */
  darkMin: number;
  kind: HorizonKind;
  /** Short line under the name. */
  label: string;
}

export interface Horizon {
  rows: HorizonRow[];
  renewals: Renewal[];
}

export interface Renewal {
  id: HarnessId;
  window: UsageWindow;
  resetsAt: number;
}

function runway(model: WindowModel | null): number {
  if (model?.status === 'out') return 0;
  return model?.pace.kind === 'risk' ? model.pace.runwayMin : Number.POSITIVE_INFINITY;
}

function horizonRow(view: HarnessView, ctx: ViewContext, model: WindowModel | null): HorizonRow {
  const base = { view, model: null, windowName: null, cover: null, leftMin: null, darkMin: 0 };
  if (view.state === 'loading') {
    return { ...base, kind: 'unknown', label: t('surface.reading') };
  }
  if (view.state === 'error' || view.stale || lostSetup(view)) {
    return {
      ...base,
      kind: 'broken',
      label: view.stale ? t('horizon.label.stale') : t('horizon.label.unavailable'),
    };
  }
  if (model === null) return { ...base, kind: 'unknown', label: t('surface.no_reading') };
  const name = quotaName(model.window);
  if (model.status === 'out') {
    return {
      ...base,
      model,
      windowName: name,
      cover: model.leftMin === null ? null : 0,
      leftMin: model.leftMin,
      darkMin: model.leftMin ?? 0,
      kind: 'out',
      label: t(model.leftMin === null ? 'window.out' : 'horizon.label.out'),
    };
  }
  if (model.leftMin === null) {
    return {
      ...base,
      model,
      windowName: name,
      kind: 'unknown',
      label: t('horizon.label.no_window'),
    };
  }
  const fmt = durationFormat(ctx.settings);
  const pace = model.pace;
  switch (pace.kind) {
    case 'risk': {
      const cover = model.leftMin <= 0 ? 0 : pace.runwayMin / model.leftMin;
      return {
        ...base,
        model,
        windowName: name,
        cover: Math.min(1, Math.max(0, cover)),
        leftMin: model.leftMin,
        darkMin: pace.earlyMin,
        kind: 'short',
        label: t('horizon.label.short', {
          duration: formatDuration(pace.earlyMin, fmt, model.long),
        }),
      };
    }
    case 'fine':
      return {
        ...base,
        model,
        windowName: name,
        cover: 1,
        leftMin: model.leftMin,
        kind: 'lasts',
        label: t('horizon.label.lasts', {
          duration: formatDuration(model.leftMin, fmt, model.long),
        }),
      };
    case 'early':
      return {
        ...base,
        model,
        windowName: name,
        leftMin: model.leftMin,
        kind: 'unknown',
        label: t('surface.too_early'),
      };
    case 'unknown':
    case 'out':
      return {
        ...base,
        model,
        windowName: name,
        leftMin: model.leftMin,
        kind: 'unknown',
        label: t('surface.no_basis'),
      };
  }
}

/** Blocked windows lead, followed by the closest projected exhaustion. */
export function horizonOf(views: readonly HarnessView[], ctx: ViewContext): Horizon {
  const renewals: Renewal[] = [];
  const rows = views.map((view) => {
    if (view.state !== 'ready' || view.stale) return horizonRow(view, ctx, null);
    const summary = summaryOf(view, ctx);
    let selected = view.windows.find((window) => window.id === summary?.windowId);
    let shortest = summary?.status === 'out' ? 0 : Number.POSITIVE_INFINITY;
    for (const window of view.windows) {
      if (isExpired(window, ctx.nowSecs)) continue;
      if (window.resetsAt !== null && Number.isFinite(new Date(window.resetsAt * 1000).getTime())) {
        renewals.push({ id: view.id, window, resetsAt: window.resetsAt });
      }
      const pace = paceOf(window, ctx.nowSecs);
      if (pace.kind === 'risk' && pace.runwayMin < shortest) {
        selected = window;
        shortest = pace.runwayMin;
      }
    }
    return horizonRow(view, ctx, selected ? modelWindow(view.id, selected, ctx) : null);
  });
  rows.sort((a, b) => {
    const first = runway(a.model);
    const second = runway(b.model);
    return first === second ? b.darkMin - a.darkMin : first - second;
  });
  return { rows, renewals: renewals.sort((a, b) => a.resetsAt - b.resetsAt) };
}

/** "esgota às 18:40" for a row that will not reach its renewal; null when it will. */
export function exhaustionLabel(row: HorizonRow, ctx: ViewContext): string | null {
  if (row.kind === 'out') return t('horizon.exhausted');
  if (row.kind !== 'short' || row.model === null || row.model.pace.kind !== 'risk') return null;
  const at = Math.round(ctx.nowSecs + row.model.pace.runwayMin * 60);
  return t('horizon.exhausts', { at: formatClock(at, ctx.nowSecs).label });
}
