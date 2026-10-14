// In-memory Backend for `pnpm dev` in a browser and for tests. It behaves like the real one where
// the UI can tell: settings are normalized and validated, secrets are write-only flags, the shim
// has a small state machine, and every change emits a snapshot. No network, no files, no clock
// reads beyond the injected one.
import { type Backend, BackendError, type Unsubscribe } from '../lib/backend';
import type { UpdateStatus } from '../lib/contract';
import {
  type ExternalLink,
  HARNESS_IDS,
  type HarnessError,
  type HarnessHistory,
  type HarnessId,
  HarnessIdSchema,
  type HarnessView,
  MAX_EPOCHS_PER_WINDOW,
  type SecretKind,
  type SecretStatus,
  SecretStatusSchema,
  type Settings,
  SettingsSchema,
  type ShimStatus,
  ShimStatusSchema,
  type Snapshot,
  SnapshotSchema,
} from '../lib/contract';
import type { Env } from './fixtures';
import { demoHistory } from './history';
import { type ScenarioName, startFor, viewsFor } from './scenarios';
import { normalizeSettings, validateSettings } from './settingsRules';
import { MockUpdater } from './updater';

export interface MockOptions {
  scenario?: ScenarioName;
  /** Epoch seconds. Injected so tests can freeze or advance time. */
  now?: () => number;
  /** Simulated round trip for every call, in milliseconds. */
  latencyMs?: number;
  settings?: Settings;
  secrets?: SecretStatus;
}

type Listeners<T> = Set<(payload: T) => void>;

const reject = (code: HarnessError['code'], params: Record<string, string> = {}) =>
  new BackendError({ code, retryable: false, params });

export class MockBackend implements Backend {
  readonly updater = new MockUpdater();
  getUpdateStatus = () =>
    this.call('get_update_status', () => structuredClone(this.updater.status));
  checkUpdate = () => this.call('check_update', () => this.updater.check(this.clock()));
  downloadUpdate = () => this.call('download_update', () => this.updater.download());
  installUpdate = () => this.call('install_update', () => this.updater.install());
  onUpdateStatus = (callback: (status: UpdateStatus) => void) =>
    this.listen(this.updater.listeners, callback);
  /** Names of the commands received, oldest first. Tests read it; the UI never does. */
  readonly calls: string[] = [];
  readonly openedLinks: ExternalLink[] = [];
  lastFlyoutHeight: number | null = null;

  private scenario: ScenarioName;
  private readonly clock: () => number;
  private readonly latencyMs: number;
  private readonly anchor: number;
  private settings: Settings;
  private secrets: SecretStatus;
  private shim: ShimStatus;
  private firstSessionSeen: boolean;
  private historyCleared = false;
  /** Refreshes asked of Claude while it waits for a first session; the second one finds data. */
  private waitingRefreshes = 0;
  private readonly refreshedAt = new Map<HarnessId, number>();
  private readonly overrides = new Map<HarnessId, HarnessView>();
  private readonly snapshotListeners: Listeners<Snapshot> = new Set();
  private readonly settingsListeners: Listeners<Settings> = new Set();
  private readonly shownListeners: Listeners<undefined> = new Set();

  constructor(options: MockOptions = {}) {
    this.scenario = options.scenario ?? 'demo';
    this.clock = options.now ?? (() => Math.floor(Date.now() / 1000));
    this.latencyMs = options.latencyMs ?? 0;
    this.anchor = this.clock();
    const start = startFor(this.scenario);
    this.settings = options.settings ?? start.settings;
    this.secrets = options.secrets ?? start.secrets;
    this.shim = start.shim;
    this.firstSessionSeen = this.scenario !== 'fresh';
  }

  // -- test and dev controls -------------------------------------------------------------------

  /** Replaces one view (for cases a scenario does not cover) and tells the UI. */
  override(view: HarnessView): void {
    this.overrides.set(view.id, view);
    this.emitSnapshot();
  }

  emitSnapshot(): void {
    const snapshot = this.snapshot();
    for (const listener of this.snapshotListeners) listener(snapshot);
  }

  emitFlyoutShown(): void {
    for (const listener of this.shownListeners) listener(undefined);
  }

  // -- Backend ---------------------------------------------------------------------------------

  getSnapshot = () => this.call('get_snapshot', () => this.snapshot());

  refresh = (harness?: HarnessId) =>
    this.call('refresh', () => {
      const targets = harness ? [harness] : HARNESS_IDS;
      for (const id of targets) this.refreshedAt.set(id, this.clock());
      if (!this.firstSessionSeen && (!harness || harness === 'claude')) {
        this.waitingRefreshes += 1;
        this.firstSessionSeen = this.waitingRefreshes > 1;
      }
      queueMicrotask(() => this.emitSnapshot());
    });

  getSettings = () => this.call('get_settings', () => structuredClone(this.settings));

  saveSettings = (next: Settings) =>
    this.call('save_settings', () => {
      const parsed = SettingsSchema.parse(next);
      const problem = validateSettings(parsed);
      if (problem) throw new BackendError(problem);
      this.settings = normalizeSettings(parsed);
      const saved = structuredClone(this.settings);
      queueMicrotask(() => {
        for (const listener of this.settingsListeners) listener(saved);
        this.emitSnapshot();
      });
      return saved;
    });

  secretStatus = () => this.call('secret_status', () => ({ ...this.secrets }));

  setSecret = (kind: SecretKind, value: string) =>
    this.call('set_secret', () => {
      if (value.trim() === '') throw reject('invalid_input', { field: 'value' });
      this.secrets = SecretStatusSchema.parse({ ...this.secrets, [kind]: true });
      this.refreshedAt.set(kind, this.clock());
      queueMicrotask(() => this.emitSnapshot());
      return { ...this.secrets };
    });

  clearSecret = (kind: SecretKind) =>
    this.call('clear_secret', () => {
      this.secrets = SecretStatusSchema.parse({ ...this.secrets, [kind]: false });
      queueMicrotask(() => this.emitSnapshot());
      return { ...this.secrets };
    });

  claudeShimStatus = () => this.call('claude_shim_status', () => ({ ...this.shim }));

  installClaudeShim = () =>
    this.call('install_claude_shim', () => {
      if (!this.settings.claude.shimConsent)
        throw reject('invalid_input', { field: 'shimConsent' });
      this.shim = ShimStatusSchema.parse({ ...this.shim, state: 'installed' });
      queueMicrotask(() => this.emitSnapshot());
      return { ...this.shim };
    });

  uninstallClaudeShim = () =>
    this.call('uninstall_claude_shim', () => {
      this.shim = ShimStatusSchema.parse({ ...this.shim, state: 'not_installed', chained: false });
      queueMicrotask(() => this.emitSnapshot());
      return { ...this.shim };
    });

  getHistory = (harness: HarnessId, limit: number = MAX_EPOCHS_PER_WINDOW) =>
    this.call('get_history', (): HarnessHistory => {
      const id = HarnessIdSchema.safeParse(harness);
      if (!id.success) throw reject('invalid_input', { field: 'harness' });
      if (!Number.isInteger(limit) || limit < 1 || limit > MAX_EPOCHS_PER_WINDOW)
        throw reject('invalid_input', { field: 'limit' });
      const view = this.snapshot().views.find((candidate) => candidate.id === id.data);
      const empty = { harness: id.data, epochs: [] };
      const silent =
        this.scenario === 'fresh' || this.historyCleared || !this.settings.general.historyEnabled;
      if (view === undefined || view.state !== 'ready' || silent) return empty;
      return demoHistory(view, this.clock(), limit);
    });

  clearHistory = () =>
    this.call('clear_history', () => {
      this.historyCleared = true;
    });

  openLink = (link: ExternalLink) =>
    this.call('open_link', () => {
      this.openedLinks.push(link);
    });

  hideFlyout = () => this.call('hide_flyout', () => undefined);

  setFlyoutHeight = (height: number) =>
    this.call('set_flyout_height', () => {
      this.lastFlyoutHeight = height;
    });

  quit = () => this.call('quit_app', () => undefined);

  // The mock only emits valid snapshots, so onInvalid never fires; it is typed for spies and parity.
  onSnapshot = (callback: (snapshot: Snapshot) => void, _onInvalid?: () => void) =>
    this.listen(this.snapshotListeners, callback);
  onSettings = (callback: (settings: Settings) => void) =>
    this.listen(this.settingsListeners, callback);
  onFlyoutShown = (callback: () => void) => this.listen(this.shownListeners, () => callback());

  // -- internals -------------------------------------------------------------------------------

  private snapshot(): Snapshot {
    const env: Env = { now: this.anchor, settings: this.settings, secrets: this.secrets };
    const views = viewsFor(this.scenario, env, this.firstSessionSeen).map((view) => {
      const override = this.overrides.get(view.id);
      if (override) return override;
      const read = this.refreshedAt.get(view.id);
      const fresh = view.state === 'ready' && !view.stale && view.readAt !== null;
      return fresh && read !== undefined ? { ...view, readAt: read } : view;
    });
    return SnapshotSchema.parse({ generatedAt: this.clock(), views });
  }

  private async call<T>(name: string, work: () => T): Promise<T> {
    this.calls.push(name);
    if (this.latencyMs > 0) await new Promise((resolve) => setTimeout(resolve, this.latencyMs));
    return work();
  }

  private listen<T>(set: Listeners<T>, callback: (payload: T) => void): Promise<Unsubscribe> {
    set.add(callback);
    return Promise.resolve(() => {
      set.delete(callback);
    });
  }
}
