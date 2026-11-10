// The two layers that open over the list: the lateral detail of one provider and the Ctrl+K
// palette. Each remembers what had the keyboard when it opened and hands it back on close.
import { useCallback, useEffect, useRef, useState } from 'react';

export interface Overlays {
  paletteOpen: boolean;
  detailOpen: boolean;
  openPalette: () => void;
  closePalette: () => void;
  /** Opens the detail and remembers the control that asked for it. */
  openDetail: () => void;
  /** Opens the detail with no opener to return to (a palette command). */
  showDetail: () => void;
  /** Closes the detail and returns the keyboard to its opener. */
  closeDetail: () => void;
  /** Closes the detail because the screen under it changed: focus stays where it is. */
  hideDetail: () => void;
}

function useLayer() {
  const [open, setOpen] = useState(false);
  const opener = useRef<Element | null>(null);
  const show = useCallback(() => {
    opener.current = document.activeElement;
    setOpen(true);
  }, []);
  const close = useCallback(() => {
    setOpen(false);
    if (opener.current instanceof HTMLElement) opener.current.focus();
  }, []);
  return { open, setOpen, show, close };
}

export function useOverlays(): Overlays {
  const palette = useLayer();
  const detail = useLayer();
  const { show: openPalette } = palette;
  const { setOpen: setDetailOpen } = detail;

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key.toLowerCase() !== 'k' || !(event.ctrlKey || event.metaKey)) return;
      event.preventDefault();
      openPalette();
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [openPalette]);

  const showDetail = useCallback(() => setDetailOpen(true), [setDetailOpen]);
  const hideDetail = useCallback(() => setDetailOpen(false), [setDetailOpen]);
  return {
    paletteOpen: palette.open,
    detailOpen: detail.open,
    openPalette,
    closePalette: palette.close,
    openDetail: detail.show,
    showDetail,
    closeDetail: detail.close,
    hideDetail,
  };
}
