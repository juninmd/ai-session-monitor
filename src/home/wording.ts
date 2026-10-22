// Wording for the proposed surfaces (from the message catalog). Kept in one place so the same fact is never
// described two different ways in two lists.
import { groupName } from '../app/layout';
import {
  leftText,
  resetMoment,
  rowTitle,
  type ViewContext,
  type WindowModel,
  windowTitle,
} from '../app/windowModel';
import { t } from '../i18n';
import type { HarnessId, UsageWindow } from '../lib/contract';

/**
 * "Opus · Semana" for a quota that belongs to a pool or a per-model group, "5 horas" for a plain
 * one. The detail table names the group in its own header row; a single line of text has to say it.
 */
export function quotaName(window: UsageWindow): string {
  const base = rowTitle(window);
  return window.group === null ? base : `${groupName(window.group)} · ${base}`;
}

/**
 * The name a list gives one window. An independent quota keeps the name the source's own title
 * resolves to ("Semana, todos os modelos", "Sessão de 5 horas"); a quota that shares a pool is
 * named by the pool and the window, so three per-model weeks never read as the same row. A pool
 * whose name is the window's own ("Fable") is said once.
 */
export function windowLabel(harness: HarnessId, window: UsageWindow): string {
  if (window.group === null) return windowTitle(harness, window);
  const base = rowTitle(window);
  const pool = groupName(window.group);
  return base === pool ? base : `${pool} · ${base}`;
}

/** "renova em 3 h 10" / "volta às 18:40" / "sem horário de renovação". */
export function renewalText(model: WindowModel | null, ctx: ViewContext): string {
  if (model === null) return t('wording.no_reading');
  if (model.expired) {
    const when = resetMoment(model, ctx);
    return when === null ? t('wording.restarted') : t('wording.restarted_at', { time: when.text });
  }
  const left = leftText(model, ctx);
  if (left !== null) return t('wording.renews_in', { left });
  const when = resetMoment(model, ctx);
  if (when === null) return t('wording.no_renewal');
  return model.status === 'out'
    ? t('wording.back', { when: when.label })
    : t('wording.renews', { when: when.label });
}
