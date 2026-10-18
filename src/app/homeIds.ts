// The DOM ids the flyout agrees on. The shell, the home surfaces and App all focus each other's
// elements through them, so a row, a name span, the drawer and the settings screen can never grab
// each other's id.
import type { HarnessId } from '../lib/contract';

/** The settings screen's own heading: what the shell focuses when a screen replaces a panel. */
export const SETTINGS_TITLE_ID = 'settings-title';

/** The drawer's close button: the first thing focus lands on when it opens. */
export const NX_DRAWER_CLOSE_ID = 'nx-drawer-close';

export function rowId(id: HarnessId): string {
  return `nx-tab-${id}`;
}

export function nxTabId(id: HarnessId): string {
  return rowId(id);
}

export function nxNameId(id: HarnessId): string {
  return `nx-name-${id}`;
}

/** The detail panel's own heading id: the row already owns `nx-name-<id>`. */
export function nxTitleId(id: HarnessId): string {
  return `nx-title-${id}`;
}

/** Move focus to one provider's row, as a keyboard move does. */
export function focusRow(id: HarnessId): void {
  document.getElementById(rowId(id))?.focus();
}

/**
 * Park focus on something that outlives the control that was just removed: the row when the grid
 * has it, else the open drawer, else the settings screen. Without this a replaced button drops the
 * keyboard on <body>.
 */
export function parkFocus(id: HarnessId): void {
  const row = document.getElementById(rowId(id));
  if (row !== null) {
    row.focus();
    return;
  }
  const drawer = document.getElementById(NX_DRAWER_CLOSE_ID);
  if (drawer !== null) {
    drawer.focus();
    return;
  }
  document.getElementById(SETTINGS_TITLE_ID)?.focus();
}
