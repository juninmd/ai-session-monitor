// Tools with an official usage channel and a credential the app could accept as a new source.
// Candidates, not commitments: each one would need its own collector, card and data-sources entry.
import { type MessageKey, t } from '../i18n';

export type Confidence = 'high' | 'medium' | 'low';

export interface Candidate {
  id: string;
  name: string;
  credential: string;
  offer: string;
  /** How sure the wording of the source is; the level word comes from the catalog. */
  confidence: Confidence;
}

// The text fields are getters: the catalog is read when the list is rendered, never at import, so
// the list follows the active language.
function candidate(
  id: string,
  key: 'amp' | 'copilot' | 'kiro' | 'cline' | 'windsurf' | 'command_code',
  name: string,
  confidence: Confidence,
): Candidate {
  return {
    id,
    name,
    get credential() {
      return t(`candidates.${key}.credential` as MessageKey);
    },
    get offer() {
      return t(`candidates.${key}.offer` as MessageKey);
    },
    confidence,
  };
}

export const CANDIDATES: readonly Candidate[] = [
  candidate('amp', 'amp', 'Amp (Sourcegraph)', 'high'),
  candidate('copilot', 'copilot', 'GitHub Copilot', 'medium'),
  candidate('kiro', 'kiro', 'Kiro CLI', 'medium'),
  candidate('cline', 'cline', 'Cline', 'medium'),
  candidate('windsurf', 'windsurf', 'Windsurf / Devin', 'high'),
  candidate('command-code', 'command_code', 'Command Code', 'medium'),
];
