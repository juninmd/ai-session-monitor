// Browser-only entry for the proposed interface. Everything it shows is fabricated and held in
// memory; it never reads a profile, a credential or the network.
import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { ErrorBoundary } from '../../components/shell/ErrorBoundary';
import { HARNESS_IDS } from '../../lib/contract';
import { createMockBackend, scenarioFromSearch } from '../../mock';
import { startFor } from '../../mock/scenarios';
import { NextStage } from './NextStage';
import './next.css';

/** A machine that already has most tools, and a few waiting for a key, so the wizard has work. */
const ENABLED = ['claude', 'codex', 'antigravity', 'opencode', 'kimi', 'openrouter', 'deepseek'];
const MISSING_KEYS = ['kimi', 'openrouter'] as const;

export function nextBackend(search = window.location.search) {
  const scenario = scenarioFromSearch(search);
  const start = startFor(scenario);
  for (const id of HARNESS_IDS) start.settings[id].enabled = ENABLED.includes(id);
  start.settings.general.showUsed = true;
  start.settings.claude.oauthUsage = false;
  const secrets = { ...start.secrets };
  for (const id of MISSING_KEYS) secrets[id] = false;
  return createMockBackend({ scenario, settings: start.settings, secrets, latencyMs: 60 });
}

export function startNext(container: HTMLElement): void {
  document.documentElement.dataset.prototype = 'next';
  const backend = nextBackend();
  createRoot(container).render(
    <StrictMode>
      <ErrorBoundary>
        <NextStage backend={backend} />
      </ErrorBoundary>
    </StrictMode>,
  );
}
