import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { App } from '../App';
import { ErrorBoundary } from '../components/shell/ErrorBoundary';
import { HARNESS_IDS } from '../lib/contract';
import { createMockBackend, scenarioFromSearch } from '../mock';
import { startFor } from '../mock/scenarios';
import { PreviewHost } from './PreviewHost';
import './preview.css';
import './grid.css';
import './details.css';
import './actions.css';
import './overview.css';

export function prototypeBackend(search = window.location.search) {
  const scenario = scenarioFromSearch(search);
  const start = startFor(scenario);
  const enabled = new Set(['claude', 'codex', 'antigravity', 'opencode', 'kimi', 'openrouter']);
  for (const id of HARNESS_IDS) start.settings[id].enabled = enabled.has(id);
  start.settings.general.showUsed = true;
  start.settings.general.alertThresholds = [90];
  return createMockBackend({ scenario, settings: start.settings, latencyMs: 120 });
}

export function startPrototype(container: HTMLElement): void {
  document.documentElement.dataset.prototype = 'grid';
  const backend = prototypeBackend();
  createRoot(container).render(
    <StrictMode>
      <ErrorBoundary>
        <PreviewHost backend={backend}>
          <App backend={backend} preview />
        </PreviewHost>
      </ErrorBoundary>
    </StrictMode>,
  );
}
