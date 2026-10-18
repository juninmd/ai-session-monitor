// Shared mounting for the home tests: the whole flyout on fabricated data, plus the queries every
// surface needs. Kept out of the test files so each of them stays short.
import { screen, waitFor } from '@testing-library/react';
import type { UserEvent } from '@testing-library/user-event';
import { expect } from 'vitest';
import type { MockBackend } from '../mock';
import { type ScenarioName, startFor } from '../mock/scenarios';
import { renderApp } from '../test/helpers';

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
  settings?: (draft: ReturnType<typeof startFor>['settings']) => void;
}

export function mount(
  scenario: ScenarioName = 'demo',
  options: MountOptions = {},
): { backend: MockBackend; user: UserEvent } {
  const start = startFor(scenario);
  const settings = structuredClone(start.settings);
  options.settings?.(settings);
  return renderApp({
    scenario,
    settings,
    secrets: { ...start.secrets, ...options.secrets },
    latencyMs: 0,
  });
}

/** The cards of a list surface: buttons, marked by the tier or kind they carry. */
export async function rows(attribute: 'data-tier' | 'data-kind'): Promise<HTMLElement[]> {
  await waitFor(() => expect(document.querySelector(`button[${attribute}]`)).not.toBeNull());
  return [...document.querySelectorAll<HTMLElement>(`button[${attribute}]`)];
}

/** The lateral detail, which opens on a card rather than sitting on the page. */
export function panel(name: string) {
  return screen.getByRole('dialog', { name });
}

export function detail(): HTMLElement {
  const drawer = document.querySelector<HTMLElement>('.nx-drawer');
  if (drawer === null) throw new Error('the detail drawer is not open');
  return drawer;
}

/** Open the lateral detail for one provider, the way a click does. */
export async function openDetail(user: UserEvent, label: string) {
  const cards = await rows('data-tier');
  const card = cards.find((node) => node.textContent?.includes(label));
  if (card === undefined) throw new Error(`no card for ${label}`);
  await user.click(card);
}
