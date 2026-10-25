// Height protocol (docs/ipc.md): the webview owns its height. It reports the natural content
// height; the shell clamps to the work area and the panel body scrolls while header and tabs stay.
import { type RefObject, useEffect } from 'react';
import type { Backend } from '../lib/backend';

const DEBOUNCE_MS = 50;

/**
 * Natural height in CSS px, independent of the clamp: the chrome (root minus the scroller's
 * visible part) plus the full scrollable content.
 */
export function naturalHeight(root: HTMLElement, scroller: HTMLElement): number {
  const chrome = root.getBoundingClientRect().height - scroller.clientHeight;
  return Math.ceil(chrome + scroller.scrollHeight);
}

export function useFlyoutHeight(
  backend: Backend,
  rootRef: RefObject<HTMLElement | null>,
  scrollRef: RefObject<HTMLElement | null>,
  measure: (root: HTMLElement, scroller: HTMLElement) => number = naturalHeight,
  ready = true,
): void {
  useEffect(() => {
    if (!ready) return;
    const root = rootRef.current;
    const scroller = scrollRef.current;
    if (!root || !scroller) return;

    let timer: ReturnType<typeof setTimeout> | undefined;
    let reported = 0;
    const report = () => {
      timer = undefined;
      const height = measure(root, scroller);
      if (!(height > 0) || height === reported) return;
      reported = height;
      // A failed call leaves the window at its old size: forget it so the next change retries.
      backend.setFlyoutHeight(height).catch(() => {
        reported = 0;
      });
    };
    const schedule = () => {
      if (timer !== undefined) clearTimeout(timer);
      timer = setTimeout(report, DEBOUNCE_MS);
    };

    const observer = new ResizeObserver(schedule);
    observer.observe(root);
    // The content box keeps changing even while the root is clamped to the work area.
    if (scroller.firstElementChild) observer.observe(scroller.firstElementChild);
    schedule();

    return () => {
      observer.disconnect();
      if (timer !== undefined) clearTimeout(timer);
    };
  }, [backend, rootRef, scrollRef, measure, ready]);
}
