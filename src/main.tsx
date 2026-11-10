import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { App } from './App';
import { applySkin, loadSkin } from './app/skin';
import { ErrorBoundary } from './components/shell/ErrorBoundary';
import { loadCatalog } from './i18n/catalogs';
import { applyDocumentLocale, getLocale, recalledLocale, setLocale } from './i18n/locale';
import { type Backend, createTauriBackend, isTauriRuntime } from './lib/backend';
import './styles/index.css';

const MOCK_LATENCY_MS = 120;

async function selectBackend(): Promise<Backend> {
  if (isTauriRuntime()) return createTauriBackend();
  // Outside Tauri (`pnpm dev` in a browser) the UI runs on fabricated data; nothing is read.
  const { createMockBackend, scenarioFromSearch } = await import('./mock');
  return createMockBackend({
    scenario: scenarioFromSearch(window.location.search),
    latencyMs: MOCK_LATENCY_MS,
  });
}

async function start(): Promise<void> {
  const container = document.getElementById('root');
  if (!container) throw new Error('missing #root');
  // The browser host draws a page and a shadow around the flyout; the Tauri window does not.
  document.documentElement.dataset.host = isTauriRuntime() ? 'tauri' : 'browser';
  applySkin(loadSkin());
  // The remembered language's chunk loads while the backend is chosen; a missing chunk is not a
  // reason to stay closed, the texts fall back to English.
  const locale = recalledLocale() ?? getLocale();
  const [backend] = await Promise.all([selectBackend(), loadCatalog(locale).catch(() => {})]);
  setLocale(locale);
  applyDocumentLocale(getLocale());
  createRoot(container).render(
    <StrictMode>
      <ErrorBoundary>
        <App backend={backend} />
      </ErrorBoundary>
    </StrictMode>,
  );
}

start().catch((error: unknown) => {
  console.error('start failed', error instanceof Error ? error.name : 'unknown');
});
