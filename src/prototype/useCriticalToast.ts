import { useEffect, useRef, useState } from 'react';
import type { ViewContext } from '../app/windowModel';
import type { HarnessId, HarnessView } from '../lib/contract';
import { isExpired } from '../lib/usage';
import { activeViews, CRITICAL_USED_PCT } from './model';

export interface CriticalToast {
  id: HarnessId;
  used: number;
  key: string;
}

/** Demonstrates crossing notifications in the browser. Real desktop toasts belong to Rust. */
export function useCriticalToast(views: readonly HarnessView[], ctx: ViewContext) {
  const epochs = useRef(new Map<string, boolean>());
  const [toasts, setToasts] = useState<CriticalToast[]>([]);
  useEffect(() => {
    const crossed: CriticalToast[] = [];
    for (const view of activeViews(views, ctx)) {
      if (view.state !== 'ready' || view.stale) continue;
      for (const window of view.windows) {
        if (window.usedPct === null || isExpired(window, ctx.nowSecs)) continue;
        const key = `${view.id}:${window.id}:${window.resetsAt}`;
        const before = epochs.current.get(key);
        const critical = window.usedPct >= CRITICAL_USED_PCT;
        epochs.current.set(key, before === true || critical);
        if (before === false && critical && ctx.settings.general.alertThresholds.includes(90))
          crossed.push({ id: view.id, used: window.usedPct, key });
      }
    }
    // Retain baselines through stale/missing reads; neither should repeat a notification.
    if (crossed.length > 0) setToasts((current) => [...current, ...crossed]);
  }, [views, ctx]);
  return {
    toasts,
    dismiss: (key: string) => setToasts((current) => current.filter((toast) => toast.key !== key)),
  };
}
