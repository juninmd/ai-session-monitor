import { groupName } from '../app/layout';
import { summaryOf } from '../app/tabModel';
import {
  durationFormat,
  gaugeLabel,
  leftText,
  modelWindow,
  paceLine,
  rowTitle,
  type ViewContext,
  type WindowModel,
} from '../app/windowModel';
import { formatDuration, formatPercent } from '../i18n/format';
import type { HarnessView, UsageWindow } from '../lib/contract';

/** Bound grid density without combining independent quotas or hiding the limiting window. */
export function gridQuotas(view: HarnessView, ctx: ViewContext) {
  const limiting = summaryOf(view, ctx);
  const groups = new Map<string | null, UsageWindow[]>();
  for (const window of view.windows) {
    groups.set(window.group, [...(groups.get(window.group) ?? []), window]);
  }
  const limitingWindow = view.windows.find((window) => window.id === limiting?.windowId);
  const group =
    limitingWindow?.group ?? (groups.has(null) ? null : (view.windows[0]?.group ?? null));
  const candidates = groups.get(group) ?? [];
  const chosen = candidates.slice(0, 3);
  if (limitingWindow && !chosen.includes(limitingWindow) && candidates.includes(limitingWindow)) {
    chosen[chosen.length - 1] = limitingWindow;
  }
  return {
    windows: candidates.filter((window) => chosen.includes(window)),
    group: group === null ? null : groupName(group),
    extra: view.windows.length - chosen.length,
  };
}

export function compactQuota(view: HarnessView, window: UsageWindow, ctx: ViewContext) {
  const model = modelWindow(view.id, window, ctx);
  const name = window.kind === 'other' ? model.title : rowTitle(window);
  const shown = model.expired || view.stale ? null : model.shown;
  return {
    name,
    shown,
    reading: model.expired
      ? 'Expirou'
      : view.stale
        ? 'Antigo'
        : shown === null
          ? '—'
          : formatPercent(shown),
    status: shown === null ? undefined : (model.status ?? undefined),
    spoken: view.stale ? `${name}: leitura desatualizada.` : gaugeLabel(model, name),
    renewal: model.expired || view.stale ? '—' : (leftText(model, ctx) ?? '—'),
    model,
  };
}

export function unofficialSource(view: HarnessView, ctx: ViewContext): boolean {
  return (
    view.source?.official === false ||
    (view.id === 'claude' &&
      (ctx.settings.claude.oauthUsage ||
        ['oauth_usage', 'cli_usage'].includes(view.meta.extra_source ?? '')))
  );
}

/** Short presentation of the shared pace rule; never project from a stale reading. */
export function compactForecast(view: HarnessView, model: WindowModel, ctx: ViewContext) {
  const line = view.stale ? null : paceLine(model, ctx);
  if (line === null) return null;
  const label =
    model.status === 'out'
      ? 'Esgotado'
      : model.pace.kind === 'risk'
        ? `Esgota em ${formatDuration(model.pace.runwayMin, durationFormat(ctx.settings), model.long)}`
        : model.pace.kind === 'fine'
          ? 'Dura até renovar'
          : 'Sem base para prever';
  return { ...line, label };
}
