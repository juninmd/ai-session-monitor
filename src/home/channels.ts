// What each harness can still tell us about quota, budget and reset that the app does not read
// today. Every line names the command or endpoint and the fields, and states whether the app can
// reach it with the credential it already accepts. Findings verified against vendor docs and the
// mirrors docs/data-sources.md. Nothing here needs an OAuth token, a cookie or a dashboard.

import { t } from '../i18n';
import { blockedChannels } from './channels-blocked';
import { alreadyChannels, nowChannels } from './channels-read';

export type Reach =
  /** Already read today. */
  | 'already'
  /** Readable with the credential the app already accepts. */
  | 'now'
  /** Exists, but needs a token, cookie or dashboard the app refuses to touch. */
  | 'forbidden';

export type Shape = 'window' | 'fact';

export type Confidence = 'high' | 'medium' | 'low';

export interface Channel {
  id: string;
  harness: string;
  title: string;
  /** Which of the two shapes in the domain model it becomes. */
  shape: Shape;
  /** The command or endpoint and the fields that carry it. */
  how: string;
  confidence: Confidence;
  reach: Reach;
  source: string;
  /** Anything a real implementation must settle first (opt-in, badge, docs entry). */
  caveat?: string;
}

export const REACH_ORDER: readonly Reach[] = ['now', 'already', 'forbidden'];

export function reachLabel(reach: Reach): string {
  return t(`channels.reach.${reach}`);
}

export function reachNote(reach: Reach): string {
  return t(`channels.reach_note.${reach}`);
}

/** "alta" / "high": the level word alone, for a sentence that already says "confidence". */
export function confidenceLevel(level: Confidence): string {
  return t(`channels.level.${level}`);
}

export const REACH_ICON: Record<Reach, 'check' | 'plug' | 'ban'> = {
  already: 'check',
  now: 'plug',
  forbidden: 'ban',
};

/** The catalogue, in the order the surface prints it: actionable first, blocked last. */
export function channels(): readonly Channel[] {
  return [...nowChannels(), ...alreadyChannels(), ...blockedChannels()];
}
