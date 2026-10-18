// The shell behaviour around the home: the keyboard when the flyout comes back, Escape in its
// priority order (palette, settings, detail, hide) and the focus that returns after the settings
// close. App owns the state these read; the hook only subscribes and routes keys.
import { useCallback, useEffect, useRef } from 'react';
import { focusRow, NX_DRAWER_CLOSE_ID, SETTINGS_TITLE_ID } from '../app/homeIds';
import { useEscape } from '../app/useEscape';
import { useSubscription } from '../app/useSubscription';
import type { Backend } from '../lib/backend';
import type { HarnessId } from '../lib/contract';

export interface HomeShell {
  backend: Backend;
  /** The draft guard: leaving a form with unsaved edits asks first. */
  request: ((action: () => void) => void) | undefined;
  settingsOpen: boolean;
  selected: HarnessId;
  detailOpen: boolean;
  paletteOpen: boolean;
  closePalette: () => void;
  closeSettings: () => void;
  closeDetail: () => void;
  hide: () => void;
  /** Puts the clock back in step with the wall clock. */
  sync: () => void;
  refresh: () => void;
}

export function useHomeShell(input: HomeShell): void {
  const {
    backend,
    request,
    settingsOpen,
    selected,
    detailOpen,
    paletteOpen,
    closePalette,
    closeSettings,
    closeDetail,
    hide,
    sync,
    refresh,
  } = input;
  const subscribeShown = useCallback(
    (callback: () => void) => backend.onFlyoutShown(callback),
    [backend],
  );
  useSubscription(subscribeShown, () => {
    sync();
    // The flyout may have been hidden for minutes: give the keyboard back where it was.
    if (settingsOpen) document.getElementById(SETTINGS_TITLE_ID)?.focus();
    else if (detailOpen) document.getElementById(NX_DRAWER_CLOSE_ID)?.focus();
    else focusRow(selected);
    refresh();
  });

  useEscape(() => {
    if (paletteOpen) closePalette();
    else if (settingsOpen) {
      if (request) request(closeSettings);
      else closeSettings();
    } else if (detailOpen) closeDetail();
    else hide();
  });

  const wasSettings = useRef(false);
  useEffect(() => {
    if (wasSettings.current && !settingsOpen) focusRow(selected);
    wasSettings.current = settingsOpen;
  }, [settingsOpen, selected]);
}
