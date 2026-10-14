// Shared test setup: a frozen clock, an in-memory backend and the provider stack panels need.
// Every value here is fabricated; nothing reads a real profile, store or network.
import { render } from '@testing-library/react';
import userEvent, { type UserEvent } from '@testing-library/user-event';
import type { ReactNode } from 'react';
import { vi } from 'vitest';
import { App } from '../App';
import { BackendContext } from '../app/backendContext';
import { DraftProvider } from '../app/draftContext';
import { type Navigation, NavigationContext } from '../app/navigation';
import { SettingsContext } from '../app/settingsContext';
import { useSettingsState } from '../app/useSettingsState';
import type { ViewContext } from '../app/windowModel';
import type { Backend } from '../lib/backend';
import type { HarnessView, Settings, UsageWindow } from '../lib/contract';
import { createMockBackend, type MockBackend, type MockOptions } from '../mock';
import { type Env, type WindowSpec, windowOf } from '../mock/fixtures';
import { startFor } from '../mock/scenarios';

/** 2027-01-15 08:00:00 UTC, far from any daylight-saving change. */
export const NOW = Date.UTC(2027, 0, 15, 8, 0, 0) / 1000;

type FakeMethod = 'setTimeout' | 'clearTimeout' | 'setInterval' | 'clearInterval';

/** Freezes `Date` at NOW; timers stay real unless asked for, so Testing Library keeps polling. */
export function freezeClock(timers: readonly FakeMethod[] = []): void {
  vi.useFakeTimers({ now: NOW * 1000, toFake: ['Date', ...timers] });
}

export function settingsWith(change: (draft: Settings) => void): Settings {
  const draft = structuredClone(startFor('healthy').settings);
  change(draft);
  return draft;
}

export function envOf(change: Partial<Env> = {}): Env {
  const start = startFor('healthy');
  return { now: NOW, settings: start.settings, secrets: start.secrets, ...change };
}

/** A five-hour window with 48% used and 113 minutes to go; override what a case is about. */
export function win(spec: Partial<WindowSpec> = {}, env: Env = envOf()): UsageWindow {
  return windowOf(env, {
    id: 'five_hour',
    kind: 'five_hour',
    usedPct: 48,
    windowMinutes: 300,
    resetsInMin: 113,
    ...spec,
  });
}

export function ctxOf(settings: Settings = startFor('healthy').settings): ViewContext {
  return { nowSecs: NOW, settings };
}

interface StackProps {
  backend: Backend;
  navigation: Navigation;
  children: ReactNode;
}

/** What App provides to its panels, without the shell around them. */
function Stack({ backend, navigation, children }: StackProps) {
  const settings = useSettingsState(backend);
  return (
    <BackendContext value={backend}>
      <SettingsContext value={settings}>
        <DraftProvider>
          <NavigationContext value={navigation}>{children}</NavigationContext>
        </DraftProvider>
      </SettingsContext>
    </BackendContext>
  );
}

export interface PanelHarness {
  backend: MockBackend;
  user: UserEvent;
  openSettings: ReturnType<typeof vi.fn<Navigation['openSettings']>>;
}

export interface PanelOptions extends MockOptions {
  /** Runs before the first render, for calls the mount itself makes (loading settings, the shim). */
  prepare?: (backend: MockBackend) => void;
}

export function renderPanel(ui: ReactNode, options: PanelOptions = {}): PanelHarness {
  const { prepare, ...mock } = options;
  const backend = createMockBackend({ scenario: 'healthy', now: () => NOW, ...mock });
  prepare?.(backend);
  const openSettings = vi.fn<Navigation['openSettings']>();
  const user = userEvent.setup();
  render(
    <Stack backend={backend} navigation={{ openSettings }}>
      {ui}
    </Stack>,
  );
  return { backend, user, openSettings };
}

export interface AppHarness {
  backend: MockBackend;
  user: UserEvent;
}

/**
 * The whole flyout on a mock scenario, with the clock frozen so ages and countdowns are exact.
 * Pass `timers` to fake the interval behind the 20 s tick as well.
 */
export function renderApp(
  options: PanelOptions = {},
  timers: readonly FakeMethod[] = [],
): AppHarness {
  freezeClock(timers);
  const { prepare, ...mock } = options;
  const backend = createMockBackend({ now: () => Math.floor(Date.now() / 1000), ...mock });
  prepare?.(backend);
  const user = userEvent.setup();
  render(<App backend={backend} />);
  return { backend, user };
}

/** The view a mock scenario serves for one harness. */
export async function viewOf(backend: MockBackend, id: HarnessView['id']): Promise<HarnessView> {
  const snapshot = await backend.getSnapshot();
  const view = snapshot.views.find((candidate) => candidate.id === id);
  if (!view) throw new Error(`no view for ${id}`);
  return view;
}
