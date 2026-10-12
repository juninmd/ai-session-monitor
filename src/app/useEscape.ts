import { useEffect, useRef } from 'react';

/**
 * Esc anywhere in the page, including with focus on the body after a click on empty space: a
 * handler on one element would miss those. The latest handler is used without resubscribing.
 */
export function useEscape(onEscape: () => void): void {
  const handler = useRef(onEscape);
  useEffect(() => {
    handler.current = onEscape;
  });

  useEffect(() => {
    const listener = (event: KeyboardEvent) => {
      if (event.key !== 'Escape' || event.defaultPrevented || event.isComposing) return;
      event.preventDefault();
      handler.current();
    };
    document.addEventListener('keydown', listener);
    return () => document.removeEventListener('keydown', listener);
  }, []);
}
