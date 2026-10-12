import { useEffect, useState } from 'react';
import type { HarnessHistory, HarnessId } from '../lib/contract';
import { useBackend } from './backendContext';

export type HistoryState =
  | { status: 'loading' }
  | { status: 'failed' }
  | { status: 'ready'; history: HarnessHistory };

/**
 * Loads the stored epochs while the section is open. `refreshKey` (the view's read time) reloads
 * after each new reading; a failure is a state, never an error text: the webview shows no raw text.
 */
export function useHistory(harness: HarnessId, active: boolean, refreshKey: unknown): HistoryState {
  const backend = useBackend();
  const [state, setState] = useState<HistoryState>({ status: 'loading' });

  // biome-ignore lint/correctness/useExhaustiveDependencies: refreshKey only triggers a reload
  useEffect(() => {
    if (!active) return;
    let current = true;
    backend
      .getHistory(harness)
      .then((history) => current && setState({ status: 'ready', history }))
      .catch(() => current && setState({ status: 'failed' }));
    return () => {
      current = false;
    };
  }, [backend, harness, active, refreshKey]);

  return state;
}
