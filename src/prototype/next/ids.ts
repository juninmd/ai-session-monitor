// DOM ids of the proposed interface. They are prefixed so the comparison view can put the current
// interface and the proposed one in the same document without colliding, and so the keyboard
// helper below can never grab the other side's element.
import type { HarnessId } from '../../lib/contract';

export const NX_PANEL_ID = 'nx-panel';

export function nxTabId(id: HarnessId): string {
  return `nx-tab-${id}`;
}

export function nxNameId(id: HarnessId): string {
  return `nx-name-${id}`;
}

/** The detail panel's own heading id: the tile already owns `nx-name-<id>`. */
export function nxTitleId(id: HarnessId): string {
  return `nx-title-${id}`;
}

export function focusRow(id: HarnessId): void {
  document.getElementById(nxTabId(id))?.focus();
}
