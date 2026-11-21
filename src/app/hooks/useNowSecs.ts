import { useCallback, useEffect, useState } from 'react';

/** Countdowns and expiry advance on this cadence without needing a new snapshot. */
export const TICK_MS = 20_000;

const nowInSeconds = () => Math.floor(Date.now() / 1000);

export function useNowSecs(intervalMs: number = TICK_MS): { nowSecs: number; sync: () => void } {
  const [nowSecs, setNowSecs] = useState(nowInSeconds);
  const sync = useCallback(() => setNowSecs(nowInSeconds()), []);
  useEffect(() => {
    // A hidden flyout renders nothing anyone sees; skip the ticks and catch up on reveal.
    const tick = () => {
      if (!document.hidden) sync();
    };
    const id = setInterval(tick, intervalMs);
    document.addEventListener('visibilitychange', tick);
    return () => {
      clearInterval(id);
      document.removeEventListener('visibilitychange', tick);
    };
  }, [sync, intervalMs]);
  return { nowSecs, sync };
}
