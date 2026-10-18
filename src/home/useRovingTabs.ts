// Roving focus over one vertical list of providers: arrows wrap, Home and End jump to the ends,
// and only the selected row is a tab stop.
import { type KeyboardEvent, useCallback } from 'react';
import { focusRow } from '../app/homeIds';
import type { HarnessId } from '../lib/contract';

export function useRovingTabs(
  ids: readonly HarnessId[],
  selected: HarnessId,
  onSelect: (id: HarnessId) => void,
) {
  return useCallback(
    (event: KeyboardEvent<HTMLElement>) => {
      if (ids.length === 0) return;
      const index = Math.max(0, ids.indexOf(selected));
      let target: number;
      switch (event.key) {
        case 'ArrowDown':
        case 'ArrowRight':
          target = (index + 1) % ids.length;
          break;
        case 'ArrowUp':
        case 'ArrowLeft':
          target = (index - 1 + ids.length) % ids.length;
          break;
        case 'Home':
          target = 0;
          break;
        case 'End':
          target = ids.length - 1;
          break;
        default:
          return;
      }
      const id = ids[target];
      if (id === undefined) return;
      event.preventDefault();
      onSelect(id);
      focusRow(id);
    },
    [ids, selected, onSelect],
  );
}
