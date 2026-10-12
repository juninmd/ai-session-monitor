import { useCallback, useState } from 'react';
import type { Backend } from '../lib/backend';
import type { HarnessError, Snapshot } from '../lib/contract';
import { toHarnessError } from './toHarnessError';
import { useSubscription } from './useSubscription';

export interface SnapshotState {
  snapshot: Snapshot | null;
  /** The initial read failed; no snapshot is available. */
  error: HarnessError | null;
  /** The event bridge could not be set up, so numbers may stop moving. */
  eventsFailed: boolean;
  reload: () => void;
}

export function useSnapshotState(backend: Backend): SnapshotState {
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
  const [error, setError] = useState<HarnessError | null>(null);
  const [eventsFailed, setEventsFailed] = useState(false);
  // Whichever of an event and a read finishes last, the newer generatedAt wins. generatedAt is
  // whole seconds, so on a tie the event wins: a read may be older than the event that beat it.
  const keepNewer = useCallback((next: Snapshot, isEvent: boolean) => {
    setSnapshot((prev) => {
      if (prev === null) return next;
      return (isEvent ? next.generatedAt >= prev.generatedAt : next.generatedAt > prev.generatedAt)
        ? next
        : prev;
    });
  }, []);

  const load = useCallback(() => {
    setError(null);
    backend.getSnapshot().then(
      (initial) => keepNewer(initial, false),
      (failure: unknown) => setError(toHarnessError(failure)),
    );
  }, [backend, keepNewer]);

  // Listener first, read second: the shell emits only when the views change and never replays,
  // so whatever it published before the listener existed has to be in the read that follows.
  // `finally` also reads when the bridge is down, so the numbers still show beside the banner.
  // A malformed event is a protocol break: the banner shows, and the error text when nothing was read.
  const onInvalid = useCallback(() => {
    setEventsFailed(true);
    setError({ code: 'protocol_error', retryable: false, params: { event: 'snapshot' } });
  }, []);
  const subscribe = useCallback(
    (callback: (next: Snapshot) => void) => backend.onSnapshot(callback, onInvalid).finally(load),
    [backend, load, onInvalid],
  );
  useSubscription(
    subscribe,
    (next) => {
      keepNewer(next, true);
      setError(null);
    },
    () => setEventsFailed(true),
  );

  return { snapshot, error, eventsFailed, reload: load };
}
