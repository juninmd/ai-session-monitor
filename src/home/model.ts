// What the home surfaces monitor: the sources that are reachable right now.
import { isEnabled } from '../app/settingsPatch';
import type { ViewContext } from '../app/windowModel';
import type { HarnessView } from '../lib/contract';

/** Connected sources remain reachable through loading, stale data and temporary failures. */
export function activeViews(views: readonly HarnessView[], ctx: ViewContext): HarnessView[] {
  return views.filter(
    (view) =>
      isEnabled(ctx.settings, view.id) &&
      !['disabled', 'needs_consent', 'needs_setup'].includes(view.state),
  );
}
