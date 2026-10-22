import '@testing-library/jest-dom/vitest';
import { cleanup } from '@testing-library/react';
import { afterEach, beforeEach, vi } from 'vitest';
import { pinLocale } from '../i18n/locale';

// The suite reads Portuguese: the stored language is English, and the assertions predate it.
beforeEach(() => pinLocale('pt-BR'));

// Vitest globals are off, so Testing Library cannot register its own cleanup.
afterEach(cleanup);
// A test that froze the clock must not leak it into the next file's timers.
afterEach(() => vi.useRealTimers());

// jsdom has no layout engine: these are the browser APIs the UI touches. Tests that care about
// their behaviour (height reporting) stub their own.
class NoopResizeObserver implements ResizeObserver {
  observe(): void {}
  unobserve(): void {}
  disconnect(): void {}
}

if (typeof globalThis.ResizeObserver === 'undefined') {
  globalThis.ResizeObserver = NoopResizeObserver;
}

if (typeof Element.prototype.scrollIntoView !== 'function') {
  Element.prototype.scrollIntoView = () => {};
}
