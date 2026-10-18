// What each harness can still tell us about quota, budget and reset that the app does not read
// today. Every line names the command or endpoint and the fields, and states whether the app can
// reach it with the credential it already accepts. Findings verified against vendor docs and the
// mirrors docs/data-sources.md. Nothing here needs an OAuth token, a cookie or a dashboard.

import { BLOCKED } from './channels-blocked';
import { ALREADY, NOW } from './channels-read';

export type Reach =
  /** Already read today. */
  | 'already'
  /** Readable with the credential the app already accepts. */
  | 'now'
  /** Exists, but needs a token, cookie or dashboard the app refuses to touch. */
  | 'forbidden';

export type Shape = 'window' | 'fact';

export interface Channel {
  id: string;
  harness: string;
  title: string;
  /** Which of the two shapes in the domain model it becomes. */
  shape: Shape;
  /** The command or endpoint and the fields that carry it. */
  how: string;
  confidence: 'Alta' | 'Média' | 'Baixa';
  reach: Reach;
  source: string;
  /** Anything a real implementation must settle first (opt-in, badge, docs entry). */
  caveat?: string;
}

export const REACH_ORDER: readonly Reach[] = ['now', 'already', 'forbidden'];

export const REACH_LABEL: Record<Reach, string> = {
  already: 'Já lemos',
  now: 'Dá para ler agora',
  forbidden: 'Existe, mas fora das regras',
};

export const REACH_NOTE: Record<Reach, string> = {
  already: 'Em produção hoje.',
  now: 'Mesma credencial e mesma regra: passaria por SafeClient e viraria janela ou fato. Cada canal não oficial ainda precisa de entrada em docs/data-sources.md e do selo "não oficial" na interface.',
  forbidden:
    'Precisa de token OAuth, cookie, sessão de navegador ou chave-mestra. O app não faz nada disso.',
};

export const REACH_ICON: Record<Reach, 'check' | 'plug' | 'ban'> = {
  already: 'check',
  now: 'plug',
  forbidden: 'ban',
};

/** The catalogue, in the order the surface prints it: actionable first, blocked last. */
export const CHANNELS: readonly Channel[] = [...NOW, ...ALREADY, ...BLOCKED];
