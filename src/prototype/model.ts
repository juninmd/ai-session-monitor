import { isEnabled } from '../app/settingsPatch';
import { summaryOf, tabChip } from '../app/tabModel';
import type { ViewContext } from '../app/windowModel';
import type { HarnessView } from '../lib/contract';

export const CRITICAL_USED_PCT = 90;

/** Connected sources remain reachable through loading, stale data and temporary failures. */
export function activeViews(views: readonly HarnessView[], ctx: ViewContext): HarnessView[] {
  return views.filter(
    (view) =>
      isEnabled(ctx.settings, view.id) &&
      !['disabled', 'needs_consent', 'needs_setup'].includes(view.state),
  );
}

export function criticalViews(views: readonly HarnessView[], ctx: ViewContext): HarnessView[] {
  return activeViews(views, ctx).filter((view) => {
    const summary = summaryOf(view, ctx);
    return summary !== null && 100 - summary.availablePct >= CRITICAL_USED_PCT;
  });
}

/** The overview always states usage, regardless of the detail panel's display preference. */
export function usageChip(view: HarnessView, ctx: ViewContext) {
  const usedContext = {
    ...ctx,
    settings: { ...ctx.settings, general: { ...ctx.settings.general, showUsed: true } },
  };
  const visible = isEnabled(ctx.settings, view.id) ? view : { ...view, state: 'disabled' as const };
  return tabChip(visible, summaryOf(visible, usedContext), usedContext);
}
