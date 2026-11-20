// How the triage lists providers: as cards (two per row) or as a list. Presentation only, like the
// skin: it stays in this webview and never reaches the settings contract.
import { useCallback, useState } from 'react';

export type ViewMode = 'cards' | 'list';

const KEY = 'aism.view';

/** Storage can be blocked or empty: the cards are the default. */
export function loadViewMode(): ViewMode {
  try {
    return window.localStorage.getItem(KEY) === 'list' ? 'list' : 'cards';
  } catch {
    return 'cards';
  }
}

export function useViewMode(): [ViewMode, (mode: ViewMode) => void] {
  const [mode, setMode] = useState<ViewMode>(loadViewMode);
  const change = useCallback((next: ViewMode) => {
    setMode(next);
    try {
      window.localStorage.setItem(KEY, next);
    } catch {
      // The choice still applies for this run; it just will not be remembered.
    }
  }, []);
  return [mode, change];
}
