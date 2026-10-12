// The slice of the Rust rules (Settings::normalize and the save-time checks) the mock imitates, so
// the forms see the same rejections they will get from the real backend.
import { type HarnessError, LIMITS, type Settings } from '../lib/contract';

const clamp = (n: number, range: { readonly min: number; readonly max: number }) =>
  Math.min(range.max, Math.max(range.min, Math.round(n)));

/** Settings::normalize for the alert list: 1..99, sorted, unique, the four lowest. */
export function normalizeThresholds(list: readonly number[]): number[] {
  const valid = list.filter((n) => Number.isInteger(n) && n >= 1 && n <= 99);
  return [...new Set(valid)].sort((a, b) => a - b).slice(0, 4);
}

export function normalizeSettings(settings: Settings): Settings {
  const { general, litellm } = settings;
  return {
    ...settings,
    general: {
      ...general,
      intervalSecs: clamp(general.intervalSecs, LIMITS.intervalSecs),
      warnAtPct: clamp(general.warnAtPct, LIMITS.warnAtPct),
      alertThresholds: normalizeThresholds(general.alertThresholds),
    },
    litellm: {
      ...litellm,
      pollIntervalSecs: clamp(litellm.pollIntervalSecs, LIMITS.litellmPollSecs),
      timeoutSecs: clamp(litellm.timeoutSecs, LIMITS.litellmTimeoutSecs),
    },
  };
}

const invalid = (code: 'invalid_url' | 'invalid_input', field: string): HarnessError => ({
  code,
  retryable: false,
  params: { field },
});

const isLoopback = (host: string) =>
  host === 'localhost' || host === '[::1]' || /^127(\.\d{1,3}){3}$/.test(host);

function urlProblem(raw: string): boolean {
  try {
    const url = new URL(raw);
    if (url.username || url.password || url.search || url.hash) return true;
    if (url.protocol === 'https:') return false;
    return !(url.protocol === 'http:' && isLoopback(url.hostname));
  } catch {
    return true;
  }
}

/** The first rejected field as the error the backend would return, or null when acceptable. */
export function validateSettings(settings: Settings): HarnessError | null {
  const { litellm } = settings;
  if (litellm.baseUrl !== null && litellm.baseUrl !== '' && urlProblem(litellm.baseUrl)) {
    return invalid('invalid_url', 'baseUrl');
  }
  if (litellm.scope !== 'key' && litellm.baseUrl !== null && !litellm.entityId) {
    return invalid('invalid_input', 'entityId');
  }
  if (!/^[A-Za-z0-9-]+$/.test(litellm.authHeader)) return invalid('invalid_input', 'authHeader');
  if (litellm.tls.mode === 'custom_ca' && litellm.tls.pemPath.trim() === '') {
    return invalid('invalid_input', 'pemPath');
  }
  if (litellm.tls.mode === 'insecure' && litellm.tls.host.trim() === '') {
    return invalid('invalid_input', 'host');
  }
  return null;
}
