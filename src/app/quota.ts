// "Available" versus "used" is wording only: every number stays the contract's used percentage.
import { hasMessage, type MessageKey, type MessageParams, t } from '../i18n';
import type { Settings } from '../lib/contract';

export const showsUsed = (settings: Settings): boolean => settings.general.showUsed;

/** The share to print: what is left, or its complement when the user reads "used". */
export function sharePct(availablePct: number, showUsed: boolean): number {
  return showUsed ? 100 - availablePct : availablePct;
}

/** `key` in its "available" wording, or `key_used` in its "used" twin. */
export function tq(key: MessageKey, showUsed: boolean, params?: MessageParams): string {
  const twin = `${key}_used`;
  return t(showUsed && hasMessage(twin) ? twin : key, params);
}
