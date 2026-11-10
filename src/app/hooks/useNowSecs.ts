import { useCallback, useEffect, useState } from 'react';

/** Countdowns and expiry advance on this cadence without needing a new snapshot. */
export const TICK_MS = 20_000;

const nowInSeconds = () => Math.floor(Date.now() / 1000);

export function useNowSecs(intervalMs: number = TICK_MS): { nowSecs: number; sync: () => void } {
  const [nowSecs, setNowSecs] = useState(nowInSeconds);
  const sync = useCallback(() => setNowSecs(nowInSeconds()), []);
  useEffect(() => {
    const id = setInterval(sync, intervalMs);
    return () => clearInterval(id);
  }, [sync, intervalMs]);
  return { nowSecs, sync };
}
