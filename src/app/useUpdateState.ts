import { createContext, useCallback, useRef, useState } from 'react';
import type { Backend } from '../lib/backend';
import type { UpdateStatus } from '../lib/contract';
import { useAsyncAction } from './useAsyncAction';
import { useSubscription } from './useSubscription';

export function useUpdateState(backend: Backend) {
  const [status, setStatus] = useState<UpdateStatus | null>(null);
  const revision = useRef(0);
  const apply = useCallback((next: UpdateStatus) => {
    revision.current += 1;
    setStatus(next);
  }, []);
  const action = useAsyncAction(async (kind: 'check' | 'download' | 'install' | 'load') => {
    const before = revision.current;
    const next = await (kind === 'check'
      ? backend.checkUpdate()
      : kind === 'download'
        ? backend.downloadUpdate()
        : kind === 'install'
          ? backend.installUpdate()
          : backend.getUpdateStatus());
    // An event emitted after the request started is newer than its response.
    if (before === revision.current) apply(next);
  });
  const run = action.run;
  // Subscribe before reading: events published during startup must not fall into a gap.
  const subscribe = useCallback(
    (callback: (next: UpdateStatus) => void) =>
      backend.onUpdateStatus(callback).finally(() => {
        void run('load');
      }),
    [backend, run],
  );
  useSubscription(subscribe, apply);
  return { status, action };
}

export type UpdateApi = ReturnType<typeof useUpdateState>;
export const UpdateContext = createContext<UpdateApi | null>(null);
