import { t } from '../i18n';
import { formatAge } from '../i18n/format';
import type { HarnessView } from '../lib/contract';

/** Seconds until a rate-limit cool-down ends, or null when the source is not being held back. */
export function cooldownLeft(view: HarnessView, nowSecs: number): number | null {
  if (view.nextAttemptAt === null) return null;
  const left = view.nextAttemptAt - nowSecs;
  return left > 0 ? left : null;
}

/** Whole minutes from one minute up, so the text never says "0 min" or counts down by seconds. */
export function waitText(left: number): string {
  return formatAge(left >= 60 ? Math.ceil(left / 60) * 60 : left);
}

export function nextAttemptText(left: number): string {
  return t('cooldown.next', { wait: waitText(left) });
}
