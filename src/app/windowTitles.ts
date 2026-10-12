// Titles for windows whose id says more than their kind. The ids are the ones each collector
// crate emits; a window nobody has a rule for falls back to the kind-based titles.
import { t } from '../i18n';
import { spokenDuration } from '../i18n/format';
import type { HarnessId, UsageWindow } from '../lib/contract';

const LIMIT_PREFIX = 'limit:';

const named = (label: string | null) => (label ? t('window.litellm.named', { name: label }) : null);

/** LiteLLM: the headline budget carries its record's alias as label, a concurrent cap its raw duration. */
function litellmTitle(window: UsageWindow): string | null {
  if (window.id === 'budget') return named(window.label);
  if (!window.id.startsWith(LIMIT_PREFIX)) return null;
  const minutes = window.windowMinutes;
  if (minutes !== null && minutes > 0) {
    return t('window.litellm.limit', { dur: spokenDuration(minutes) });
  }
  return named(window.label);
}

/** Codex: a length that matches no known cadence arrives as `primary` or `secondary`, kind other. */
function codexTitle(window: UsageWindow): string | null {
  if (window.kind !== 'other') return null;
  const minutes = window.windowMinutes;
  if (minutes !== null && minutes > 0) {
    return t('window.codex.other_for', { dur: spokenDuration(minutes) });
  }
  if (window.id === 'primary') return t('window.codex.primary');
  if (window.id === 'secondary') return t('window.codex.secondary');
  return null;
}

/** A title that beats the source's own label, or null to use the generic rules. */
export function specialTitle(harness: HarnessId, window: UsageWindow): string | null {
  switch (harness) {
    case 'litellm':
      return litellmTitle(window);
    case 'codex':
      return codexTitle(window);
    default:
      return null;
  }
}
