import type { Option } from '../components/forms/fields';
import { formatInterval } from '../i18n/format';

/**
 * Presets clipped to the backend's range (LIMITS), always including both ends and the current
 * value so a select never shows an option the backend would clamp away or hides the saved one.
 */
export function numericOptions(
  presets: readonly number[],
  limits: { readonly min: number; readonly max: number },
  current: number,
  label: (n: number) => string,
): Option<number>[] {
  const values = new Set<number>([limits.min, limits.max, current]);
  for (const n of presets) if (n >= limits.min && n <= limits.max) values.add(n);
  return [...values].sort((a, b) => a - b).map((value) => ({ value, label: label(value) }));
}

export const INTERVAL_PRESETS = [30, 60, 120, 300, 600, 900, 1800, 3600] as const;
export const LITELLM_POLL_PRESETS = [60, 120, 300, 600, 900, 1800, 3600] as const;
export const TIMEOUT_PRESETS = [1, 5, 10, 15, 20, 30] as const;
export const WARN_PRESETS = [5, 10, 15, 20, 25, 30, 40, 50, 60, 75, 90] as const;

export const intervalLabel = formatInterval;
export const secondsLabel = (n: number) => `${n} s`;
export const percentLabel = (n: number) => `${n}%`;
