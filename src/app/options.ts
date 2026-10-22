import type { Option } from '../components/forms/fields';
import { t } from '../i18n';
import { formatInterval, formatList, formatSeconds } from '../i18n/format';

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

/** Select value for a threshold list: "80,95", or "off" for no alerts. */
export const thresholdKey = (list: readonly number[]) =>
  list.length === 0 ? 'off' : list.join(',');

export function parseThresholdKey(key: string): number[] {
  if (key === 'off') return [];
  return key.split(',').map(Number);
}

const THRESHOLD_PRESETS: readonly (readonly number[])[] = [[80, 95], [90]];

/** Off, the two presets, and the stored list when it is none of those ("Personalizado"). */
export function thresholdOptions(current: readonly number[]): Option<string>[] {
  const preset = (list: readonly number[]) => ({
    value: thresholdKey(list),
    label: formatList(list.map(percentLabel)),
  });
  const options: Option<string>[] = [
    { value: 'off', label: t('settings.alert_off') },
    ...THRESHOLD_PRESETS.map(preset),
  ];
  const key = thresholdKey(current);
  if (!options.some((option) => option.value === key)) {
    options.push({
      value: key,
      label: t('settings.alert_custom', { list: preset(current).label }),
    });
  }
  return options;
}

export const intervalLabel = formatInterval;
export const secondsLabel = formatSeconds;
export const percentLabel = (n: number) => `${n}%`;
