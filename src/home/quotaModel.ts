import { groupName } from '../app/layout';
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
import { t } from '../i18n';
import { formatDuration, formatPercent } from '../i18n/format';
import type { HarnessView, UsageWindow } from '../lib/contract';

/** Every quota window of the view, grouped in first-seen order, so a card needs no click for detail. */
export function gridQuotas(view: HarnessView) {
  const groups = new Map<string | null, UsageWindow[]>();
  for (const window of view.windows) {
    groups.set(window.group, [...(groups.get(window.group) ?? []), window]);
  }
  return [...groups].map(([group, windows]) => ({
    group: group === null ? null : groupName(group),
    windows,
  }));
}

export function compactQuota(view: HarnessView, window: UsageWindow, ctx: ViewContext) {
  const model = modelWindow(view.id, window, ctx);
  const name = window.kind === 'other' ? model.title : rowTitle(window);
  const shown = model.expired || view.stale ? null : model.shown;
  return {
    name,
    shown,
    reading: model.expired
      ? t('quota.reading.expired')
      : view.stale
        ? t('quota.reading.stale')
        : shown === null
          ? '—'
          : formatPercent(shown),
    status: shown === null ? undefined : (model.status ?? undefined),
    spoken: view.stale ? t('quota.spoken_stale', { name }) : gaugeLabel(model, name),
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
      ? t('quota.forecast.out')
      : model.pace.kind === 'risk'
        ? t('quota.forecast.risk', {
            duration: formatDuration(
              model.pace.runwayMin,
              durationFormat(ctx.settings),
              model.long,
            ),
          })
        : model.pace.kind === 'fine'
          ? t('quota.forecast.fine')
          : t('quota.forecast.unknown');
  return { ...line, label };
}
