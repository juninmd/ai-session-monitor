import '@testing-library/jest-dom/vitest';
import { cleanup } from '@testing-library/react';
import { afterEach, beforeAll, beforeEach, vi } from 'vitest';
import { pinLocale } from '../i18n/locale';

// Production fetches a locale's chunk on demand; the suite switches languages synchronously.
// Imported here, not at the top: a test file's vi.mock of a catalog must reach it first.
beforeAll(async () => {
  const { loadAllCatalogs } = await import('../i18n/catalogs');
  await loadAllCatalogs();
});

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
