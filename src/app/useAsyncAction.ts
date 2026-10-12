import { useCallback, useEffect, useRef, useState } from 'react';
import type { HarnessError } from '../lib/contract';
import { toHarnessError } from './toHarnessError';

export interface AsyncAction<A extends unknown[]> {
  /** Resolves true on success; failures land in `error` instead of throwing. */
  run: (...args: A) => Promise<boolean>;
  pending: boolean;
  error: HarnessError | null;
  clear: () => void;
}

/** Wraps a backend call so buttons can show a busy state and the mapped error. */
export function useAsyncAction<A extends unknown[]>(
  action: (...args: A) => Promise<unknown>,
): AsyncAction<A> {
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<HarnessError | null>(null);
  const actionRef = useRef(action);
  useEffect(() => {
    actionRef.current = action;
  });

  const run = useCallback(async (...args: A) => {
    setPending(true);
    setError(null);
    try {
      await actionRef.current(...args);
      return true;
    } catch (failure) {
      setError(toHarnessError(failure));
      return false;
    } finally {
      setPending(false);
    }
  }, []);

  const clear = useCallback(() => setError(null), []);
  return { run, pending, error, clear };
}
