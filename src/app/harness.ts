import { t } from '../i18n';
import { HARNESS_IDS, type HarnessId } from '../lib/contract';

/** Tab order is the order of `HarnessId::ALL` in Rust. */
export const TAB_ORDER: readonly HarnessId[] = HARNESS_IDS;

export function harnessName(id: HarnessId): string {
  return t(`harness.${id}`);
}

/** Where a settings group lives: a harness or the general options. */
export type SettingsSection = HarnessId | 'general';
