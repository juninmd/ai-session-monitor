import { useCallback, useEffect, useRef, useState } from 'react';
import type { Backend } from '../lib/backend';
import {
  defaultSettings,
  type HarnessError,
  type SecretKind,
  type SecretStatus,
  type Settings,
  type ShimStatus,
} from '../lib/contract';
import { toHarnessError } from './toHarnessError';
import { useSubscription } from './useSubscription';

export interface SettingsApi {
  settings: Settings;
  secrets: SecretStatus;
  shim: ShimStatus | null;
  /** False until the first get_settings resolves; forms wait for it. */
  loaded: boolean;
  loadError: HarnessError | null;
  /** The Credential Manager could not be read; everything but the keys keeps working. */
  secretsError: HarnessError | null;
  /** Serialized read-modify-write: patches apply in order against the latest saved value. */
  save: (patch: (current: Settings) => Settings) => Promise<Settings>;
  setSecret: (kind: SecretKind, value: string) => Promise<void>;
  clearSecret: (kind: SecretKind) => Promise<void>;
  refreshShim: () => Promise<void>;
  installShim: () => Promise<void>;
  uninstallShim: () => Promise<void>;
  reload: () => void;
}

const NO_SECRETS: SecretStatus = {
  litellm: false,
  opencode: false,
  kimi: false,
  zai: false,
  openrouter: false,
  deepseek: false,
  deepinfra: false,
  moonshot: false,
  novita: false,
  ollama: false,
  minimax: false,
};

export function useSettingsState(backend: Backend): SettingsApi {
  const [settings, setSettings] = useState<Settings>(defaultSettings);
  const [secrets, setSecrets] = useState<SecretStatus>(NO_SECRETS);
  const [shim, setShim] = useState<ShimStatus | null>(null);
  const [loaded, setLoaded] = useState(false);
  const [loadError, setLoadError] = useState<HarnessError | null>(null);
  const [secretsError, setSecretsError] = useState<HarnessError | null>(null);
  const latest = useRef<Settings>(defaultSettings);
  const hasStored = useRef(false);
  const queue = useRef<Promise<unknown>>(Promise.resolve());

  // The same value arriving twice (our own save plus its event) must not re-render the forms.
  const apply = useCallback((next: Settings) => {
    if (JSON.stringify(next) === JSON.stringify(latest.current)) return;
    latest.current = next;
    setSettings(next);
  }, []);

  // Any answer from the store proves it is readable, so an earlier failure of it is over.
  const takeSecrets = useCallback((status: SecretStatus) => {
    setSecrets(status);
    setSecretsError(null);
  }, []);

  const load = useCallback(() => {
    setLoadError(null);
    backend.getSettings().then(
      (initial) => {
        apply(initial);
        hasStored.current = true;
        setLoaded(true);
      },
      (failure: unknown) => setLoadError(toHarnessError(failure)),
    );
    // The settings never need the store, so its failure must not take the settings screen down.
    backend.secretStatus().then(takeSecrets, (failure: unknown) => {
      setSecrets(NO_SECRETS);
      setSecretsError(toHarnessError(failure));
    });
  }, [backend, apply, takeSecrets]);

  useEffect(load, [load]);

  const subscribe = useCallback(
    (callback: (next: Settings) => void) => backend.onSettings(callback),
    [backend],
  );
  useSubscription(subscribe, apply);

  // A save can drop a stored key (the backend ties it to its server), so the chip must follow.
  const refreshSecrets = useCallback(async () => {
    try {
      takeSecrets(await backend.secretStatus());
    } catch {
      // The save is done and reported as such; the chip keeps its last value until the next read.
    }
  }, [backend, takeSecrets]);

  const save = useCallback(
    (patch: (current: Settings) => Settings) => {
      const run = async () => {
        // A patch must never land on the placeholder defaults and overwrite the stored file.
        if (!hasStored.current) {
          apply(await backend.getSettings());
          hasStored.current = true;
          setLoaded(true);
        }
        try {
          const saved = await backend.saveSettings(patch(latest.current));
          apply(saved);
          return saved;
        } finally {
          // Also after a failure: the key can be gone while the write is what failed.
          await refreshSecrets();
        }
      };
      const result = queue.current.then(run, run);
      queue.current = result.catch(() => undefined);
      return result;
    },
    [backend, apply, refreshSecrets],
  );

  const setSecret = useCallback(
    async (kind: SecretKind, value: string) => {
      takeSecrets(await backend.setSecret(kind, value));
    },
    [backend, takeSecrets],
  );

  const clearSecret = useCallback(
    async (kind: SecretKind) => {
      takeSecrets(await backend.clearSecret(kind));
    },
    [backend, takeSecrets],
  );

  const refreshShim = useCallback(async () => {
    setShim(await backend.claudeShimStatus());
  }, [backend]);

  const installShim = useCallback(async () => {
    await save((s) => ({ ...s, claude: { ...s.claude, shimConsent: true } }));
    setShim(await backend.installClaudeShim());
  }, [backend, save]);

  const uninstallShim = useCallback(async () => {
    setShim(await backend.uninstallClaudeShim());
    await save((s) => ({ ...s, claude: { ...s.claude, shimConsent: false } }));
  }, [backend, save]);

  return {
    settings,
    secrets,
    shim,
    loaded,
    loadError,
    secretsError,
    save,
    setSecret,
    clearSecret,
    refreshShim,
    installShim,
    uninstallShim,
    reload: load,
  };
}
