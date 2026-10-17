// Shared mounting for the proposed-surface tests: the whole stage on fabricated data, plus the two
// queries every surface needs. Kept out of the test files so each of them stays short.
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { expect } from 'vitest';
import type { Settings } from '../../lib/contract';
import { createMockBackend } from '../../mock';
import { type ScenarioName, startFor } from '../../mock/scenarios';
import { freezeClock, NOW } from '../../test/helpers';
import { NextStage } from './NextStage';

export const RANK: Record<string, number> = {
  blocked: 0,
  risk: 1,
  broken: 2,
  watch: 3,
  loading: 4,
  fine: 5,
  unknown: 6,
};

export interface MountOptions {
  secrets?: Record<string, boolean>;
  settings?: (draft: Settings) => void;
}

export function mount(scenario: ScenarioName = 'demo', options: MountOptions = {}) {
  freezeClock();
  const start = startFor(scenario);
  const settings = structuredClone(start.settings);
  options.settings?.(settings);
  const backend = createMockBackend({
    scenario,
    settings,
    secrets: { ...start.secrets, ...options.secrets },
    now: () => NOW,
    latencyMs: 0,
  });
  const user = userEvent.setup();
  render(<NextStage backend={backend} />);
  return { backend, user };
}

/** The rows of a list surface: buttons, marked by the tier or kind they carry. */
export async function rows(attribute: 'data-tier' | 'data-kind'): Promise<HTMLElement[]> {
  await waitFor(() => expect(document.querySelector(`button[${attribute}]`)).not.toBeNull());
  return [...document.querySelectorAll<HTMLElement>(`button[${attribute}]`)];
}

export function panel(name: string) {
  return screen.getByRole('region', { name });
}
