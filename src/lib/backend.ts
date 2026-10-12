// The only door between the webview and the Rust side. `Backend` is implemented by the Tauri
// adapter below and by the dev mock (src/mock), so the UI never touches `invoke` directly.
// Every payload is parsed with the contract schemas; unknown shapes never reach components.
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { z } from 'zod';
import {
  COMMANDS,
  EVENTS,
  type ExternalLink,
  type HarnessError,
  HarnessErrorSchema,
  type HarnessId,
  type SecretKind,
  type SecretStatus,
  SecretStatusSchema,
  type Settings,
  SettingsSchema,
  type ShimStatus,
  ShimStatusSchema,
  type Snapshot,
  SnapshotSchema,
} from './contract';

export type Unsubscribe = () => void;

export interface Backend {
  getSnapshot(): Promise<Snapshot>;
  /** Starts a collection now (all harnesses, or one); the result arrives through onSnapshot. */
  refresh(harness?: HarnessId): Promise<void>;
  getSettings(): Promise<Settings>;
  /** Validates, persists and applies; resolves to the normalized settings. */
  saveSettings(settings: Settings): Promise<Settings>;
  secretStatus(): Promise<SecretStatus>;
  setSecret(kind: SecretKind, value: string): Promise<SecretStatus>;
  clearSecret(kind: SecretKind): Promise<SecretStatus>;
  claudeShimStatus(): Promise<ShimStatus>;
  installClaudeShim(): Promise<ShimStatus>;
  uninstallClaudeShim(): Promise<ShimStatus>;
  openLink(link: ExternalLink): Promise<void>;
  hideFlyout(): Promise<void>;
  /** Content height in CSS pixels; the backend clamps it to the work area. */
  setFlyoutHeight(height: number): Promise<void>;
  quit(): Promise<void>;
  /** `onInvalid` fires, without the payload, when an event fails the contract schema. */
  onSnapshot(callback: (snapshot: Snapshot) => void, onInvalid?: () => void): Promise<Unsubscribe>;
  onSettings(callback: (settings: Settings) => void): Promise<Unsubscribe>;
  onFlyoutShown(callback: () => void): Promise<Unsubscribe>;
}

/** A failed command. `error.code` is what the UI maps to a message and an action. */
export class BackendError extends Error {
  readonly error: HarnessError;

  constructor(error: HarnessError) {
    super(error.code);
    this.name = 'BackendError';
    this.error = error;
  }
}

function toBackendError(raw: unknown): BackendError {
  const parsed = HarnessErrorSchema.safeParse(raw);
  // Anything that is not our error shape is collapsed: raw rejection text is never surfaced.
  return new BackendError(
    parsed.success ? parsed.data : { code: 'internal', retryable: false, params: {} },
  );
}

async function call(command: string, args?: Record<string, unknown>): Promise<unknown> {
  try {
    return await invoke<unknown>(command, args);
  } catch (raw) {
    throw toBackendError(raw);
  }
}

async function callParsed<S extends z.ZodType>(
  schema: S,
  command: string,
  args?: Record<string, unknown>,
): Promise<z.infer<S>> {
  const parsed = schema.safeParse(await call(command, args));
  if (!parsed.success) {
    throw new BackendError({ code: 'protocol_error', retryable: false, params: { command } });
  }
  return parsed.data;
}

function subscribe<S extends z.ZodType>(
  event: string,
  schema: S,
  callback: (payload: z.infer<S>) => void,
  onInvalid?: () => void,
): Promise<Unsubscribe> {
  return listen<unknown>(event, (message) => {
    const parsed = schema.safeParse(message.payload);
    if (parsed.success) callback(parsed.data);
    else {
      console.error(`ignored malformed "${event}" event`, parsed.error.issues);
      onInvalid?.();
    }
  });
}

export function isTauriRuntime(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

export function createTauriBackend(): Backend {
  return {
    getSnapshot: () => callParsed(SnapshotSchema, COMMANDS.getSnapshot),
    refresh: async (harness) => {
      await call(COMMANDS.refresh, { harness });
    },
    getSettings: () => callParsed(SettingsSchema, COMMANDS.getSettings),
    saveSettings: (settings) => callParsed(SettingsSchema, COMMANDS.saveSettings, { settings }),
    secretStatus: () => callParsed(SecretStatusSchema, COMMANDS.secretStatus),
    setSecret: (kind, value) => callParsed(SecretStatusSchema, COMMANDS.setSecret, { kind, value }),
    clearSecret: (kind) => callParsed(SecretStatusSchema, COMMANDS.clearSecret, { kind }),
    claudeShimStatus: () => callParsed(ShimStatusSchema, COMMANDS.claudeShimStatus),
    installClaudeShim: () => callParsed(ShimStatusSchema, COMMANDS.installClaudeShim),
    uninstallClaudeShim: () => callParsed(ShimStatusSchema, COMMANDS.uninstallClaudeShim),
    openLink: async (link) => {
      await call(COMMANDS.openLink, { link });
    },
    hideFlyout: async () => {
      await call(COMMANDS.hideFlyout);
    },
    setFlyoutHeight: async (height) => {
      await call(COMMANDS.setFlyoutHeight, { height });
    },
    quit: async () => {
      await call(COMMANDS.quitApp);
    },
    onSnapshot: (callback, onInvalid) =>
      subscribe(EVENTS.snapshot, SnapshotSchema, callback, onInvalid),
    onSettings: (callback) => subscribe(EVENTS.settingsChanged, SettingsSchema, callback),
    onFlyoutShown: (callback) => listen(EVENTS.flyoutShown, () => callback()),
  };
}
