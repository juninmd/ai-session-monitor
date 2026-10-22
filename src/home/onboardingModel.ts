// First-run wizard: what each source needs before it can show anything, and what a reading
// attempt actually produced. Pure helpers; the wizard only sequences them.
import { describeError } from '../app/errorText';
import { harnessName } from '../app/harness';
import type { ViewContext } from '../app/windowModel';
import type { KeyedHarness } from '../components/setup/KeyConnect';
import { t, tn } from '../i18n';
import { formatAge } from '../i18n/format';
import type { HarnessId, HarnessView, Settings } from '../lib/contract';

export type Need = 'nothing' | 'consent' | 'key';
export type OnboardStep = 'choose' | 'connect' | 'verify' | 'done';

export const STEP_ORDER: readonly OnboardStep[] = ['choose', 'connect', 'verify', 'done'];

export const stepLabel = (step: OnboardStep): string => t(`onboarding.step.${step}`);

/** The sources whose reading is unlocked by a pasted key alone. A switch keeps the type narrow. */
export function keyedOf(id: HarnessId): KeyedHarness | null {
  switch (id) {
    case 'kimi':
    case 'zai':
    case 'openrouter':
    case 'deepseek':
    case 'deepinfra':
    case 'moonshot':
    case 'novita':
    case 'ollama':
    case 'minimax':
      return id;
    default:
      return null;
  }
}

export interface Candidate {
  id: HarnessId;
  name: string;
  /** The tool looks present on this machine (a source we may be able to read). */
  detected: boolean;
  need: Need;
}

export function candidateOf(view: HarnessView): Candidate {
  const name = harnessName(view.id);
  switch (view.state) {
    case 'needs_setup':
      return { id: view.id, name, detected: false, need: 'key' };
    case 'needs_consent':
      return { id: view.id, name, detected: true, need: 'consent' };
    // While the first snapshot is in flight nothing has answered yet: claiming "já responde" here
    // would mark every source as reachable before any of them has been read.
    case 'loading':
    case 'disabled':
      return { id: view.id, name, detected: false, need: 'nothing' };
    default:
      return { id: view.id, name, detected: true, need: 'nothing' };
  }
}

export function candidates(views: readonly HarnessView[]): Candidate[] {
  return views.map(candidateOf);
}

/** Everything the wizard still has to ask for, for the chosen sources. */
export function pendingOf(
  chosen: readonly HarnessId[],
  views: readonly HarnessView[],
): { keys: HarnessId[]; consents: HarnessId[] } {
  const byId = new Map(views.map((view) => [view.id, view]));
  const keys: HarnessId[] = [];
  const consents: HarnessId[] = [];
  for (const id of chosen) {
    const view = byId.get(id);
    if (view === undefined) continue;
    if (view.state === 'needs_setup') keys.push(id);
    else if (view.state === 'needs_consent') consents.push(id);
  }
  return { keys, consents };
}

/** Turning on a source that asks first is a settings write, never a new mechanism. */
export function withConsent(settings: Settings, id: HarnessId): Settings {
  switch (id) {
    case 'claude':
      return { ...settings, claude: { ...settings.claude, shimConsent: true } };
    case 'codex':
      return { ...settings, codex: { ...settings.codex, consent: true } };
    case 'antigravity':
      return { ...settings, antigravity: { ...settings.antigravity, consent: true } };
    case 'cursor':
      return { ...settings, cursor: { ...settings.cursor, consent: true } };
    default:
      return settings;
  }
}

export interface Outcome {
  tone: 'ok' | 'warn' | 'error';
  label: string;
}

/** What one reading attempt produced, in the words of the flyout, never a raw error string. */
export function outcomeOf(view: HarnessView, ctx: ViewContext): Outcome {
  switch (view.state) {
    case 'loading':
      return { tone: 'warn', label: t('onboarding.outcome.loading') };
    case 'disabled':
      return { tone: 'warn', label: t('onboarding.outcome.disabled') };
    case 'needs_consent':
      return { tone: 'warn', label: t('onboarding.outcome.needs_consent') };
    case 'needs_setup':
      return { tone: 'warn', label: t('onboarding.outcome.needs_setup') };
    case 'no_data':
      return { tone: 'warn', label: t('onboarding.outcome.no_data') };
    case 'error':
      return {
        tone: 'error',
        label:
          view.error === null
            ? t('onboarding.outcome.failed')
            : describeError(view.error, view.id).title,
      };
    case 'ready': {
      if (view.stale) {
        const label =
          view.readAt === null
            ? t('onboarding.outcome.stale')
            : t('onboarding.outcome.stale_ago', { age: formatAge(ctx.nowSecs - view.readAt) });
        return { tone: 'warn', label };
      }
      if (view.windows.length === 0)
        return { tone: 'ok', label: t('onboarding.outcome.no_windows') };
      return { tone: 'ok', label: tn('onboarding.outcome.quotas', view.windows.length) };
    }
  }
}
