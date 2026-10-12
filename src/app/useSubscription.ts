import { useEffect, useRef } from 'react';
import type { Unsubscribe } from '../lib/backend';

/**
 * Subscribes to a backend event for the lifetime of the component. The handler may change on
 * every render without resubscribing, and a subscription that resolves after unmount is released.
 */
export function useSubscription<A extends unknown[]>(
  subscribe: (callback: (...args: A) => void) => Promise<Unsubscribe>,
  handler: (...args: A) => void,
  onError?: (error: unknown) => void,
): void {
  const handlerRef = useRef(handler);
  const errorRef = useRef(onError);
  useEffect(() => {
    handlerRef.current = handler;
    errorRef.current = onError;
  });

  useEffect(() => {
    let active = true;
    let stop: Unsubscribe | null = null;
    subscribe((...args) => {
      if (active) handlerRef.current(...args);
    }).then(
      (unsubscribe) => {
        if (active) stop = unsubscribe;
        else unsubscribe();
      },
      (error: unknown) => {
        if (active) errorRef.current?.(error);
      },
    );
    return () => {
      active = false;
      stop?.();
    };
  }, [subscribe]);
}
