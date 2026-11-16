// What the home surfaces monitor: the sources that are reachable right now.
import { isEnabled } from '../app/model/settingsPatch';
import type { ViewContext } from '../app/model/windowModel';
import type { HarnessView } from '../lib/contract';

/**
 * A source that was connected and lost its setup (Claude Code's status line command replaced or
 * removed) says so in a warning. It is a broken source, not one that was never set up.
 */
export function lostSetup(view: HarnessView): boolean {
  return view.state === 'needs_setup' && view.notices.some((notice) => notice.severity === 'warn');
}

/**
 * Connected sources remain reachable through loading, stale data and temporary failures, and one
 * that lost its setup stays in sight instead of vanishing without a word.
 */
export function activeViews(views: readonly HarnessView[], ctx: ViewContext): HarnessView[] {
  return views.filter(
    (view) =>
      isEnabled(ctx.settings, view.id) &&
      (lostSetup(view) || !['disabled', 'needs_consent', 'needs_setup'].includes(view.state)),
  );
}
