// Horizon: puts every provider on the same question — "do I reach the renewal, or is there a
// stretch with no quota?" Each bar runs from now to that window's reset, and the fill is the share
// of that stretch the remaining quota can cover. The rest of the track is time with no quota. The
// projections come from paceOf via WindowModel.pace; nothing here recomputes a status.
import { summaryOf } from '../app/tabModel';
import {
  durationFormat,
  modelWindow,
  type ViewContext,
  type WindowModel,
} from '../app/windowModel';
import { t } from '../i18n';
import { formatClock, formatDuration } from '../i18n/format';
import type { HarnessView, UsageWindow } from '../lib/contract';
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
  /** The row whose window renews first, when there is one. */
  soonest: HorizonRow | null;
}

function found(view: HarnessView, id: string | undefined): UsageWindow | null {
  return view.windows.find((window) => window.id === id) ?? null;
}

function horizonRow(view: HarnessView, ctx: ViewContext): HorizonRow {
  const base = { view, model: null, windowName: null, cover: null, leftMin: null, darkMin: 0 };
  if (view.state === 'loading') {
    return { ...base, kind: 'unknown', label: t('surface.reading') };
  }
  if (view.state === 'error' || view.stale) {
    return {
      ...base,
      kind: 'broken',
      label: view.stale ? t('horizon.label.stale') : t('horizon.label.unavailable'),
    };
  }
  const summary = summaryOf(view, ctx);
  const window = summary === null ? null : found(view, summary.windowId);
  if (window === null) return { ...base, kind: 'unknown', label: t('surface.no_reading') };
  const model = modelWindow(view.id, window, ctx);
  const name = quotaName(window);
  if (model.expired || model.leftMin === null) {
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
    case 'out':
      return {
        ...base,
        model,
        windowName: name,
        cover: 0,
        leftMin: model.leftMin,
        darkMin: model.leftMin,
        kind: 'out',
        label: t('horizon.label.out'),
      };
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

/** Rows ordered the way the eye should read them: the longest uncovered stretch first. */
export function horizonOf(views: readonly HarnessView[], ctx: ViewContext): Horizon {
  const rows = views
    .map((view) => horizonRow(view, ctx))
    .sort((a, b) => {
      const weight = (row: HorizonRow) => (row.kind === 'out' ? 2 : row.kind === 'short' ? 1 : 0);
      const byKind = weight(b) - weight(a);
      return byKind !== 0 ? byKind : b.darkMin - a.darkMin;
    });
  const dated = rows.filter((row) => row.leftMin !== null);
  const soonest =
    dated.length === 0
      ? null
      : dated.reduce((best, row) => ((row.leftMin ?? 0) < (best.leftMin ?? 0) ? row : best));
  return { rows, soonest };
}

/** "esgota às 18:40" for a row that will not reach its renewal; null when it will. */
export function exhaustionLabel(row: HorizonRow, ctx: ViewContext): string | null {
  if (row.kind === 'out') return t('horizon.exhausted');
  if (row.kind !== 'short' || row.model === null || row.model.pace.kind !== 'risk') return null;
  const at = Math.round(ctx.nowSecs + row.model.pace.runwayMin * 60);
  return t('horizon.exhausts', { at: formatClock(at, ctx.nowSecs).label });
}
