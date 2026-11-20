// Which surface the list shows, and the two ways to change it: the vertical tabs and Alt+1..3.
import { useCallback, useEffect, useState } from 'react';
import { SURFACES, type Surface } from './Fly';
import { surfaceFromKey } from './surfaceKey';

export interface SurfaceOptions {
  /** Leaving a form with unsaved edits asks first. */
  guarded: (action: () => void) => void;
  hideDetail: () => void;
  toTop: () => void;
  /** The palette owns the keyboard while it is open. */
  shortcutsOff: boolean;
}

export function useSurface({ guarded, hideDetail, toTop, shortcutsOff }: SurfaceOptions) {
  const [surface, setSurface] = useState<Surface>('triage');
  const changeSurface = useCallback(
    (next: Surface) => {
      guarded(() => {
        setSurface(next);
        hideDetail();
        toTop();
      });
    },
    [guarded, hideDetail, toTop],
  );
  // The digit maps to the surface's position in the vertical tab list.
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (shortcutsOff) return;
      const target = surfaceFromKey(event, SURFACES);
      if (target === undefined) return;
      event.preventDefault();
      changeSurface(target.id);
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [changeSurface, shortcutsOff]);
  /** Back to the triage from the top, with no guard: the caller already decided. */
  const showTriage = useCallback(() => {
    setSurface('triage');
    toTop();
  }, [toTop]);
  return { surface, changeSurface, showTriage };
}
