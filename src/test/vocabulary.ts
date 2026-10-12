// What the collectors put on the wire, written out by hand and checked against their Rust sources
// (src/i18n/emitted.test.ts). When a collector starts to send a new notice code, fact id, reason
// or field name, add it here: the tests then fail until the UI can word it.
import { hasMessage } from '../i18n';
import { ERROR_CODES, type HarnessId } from '../lib/contract';

type Params = Readonly<Record<string, string>>;

const WINDOW_IDS = ['rolling', 'weekly', 'monthly'] as const;
const DB_REASONS = [
  'invalid_path',
  'not_found',
  'permission',
  'locked',
  'not_a_database',
  'schema',
  'timeout',
  'other',
] as const;
const reasons = (...values: string[]): Params[] => values.map((reason) => ({ reason }));

/** Every notice code, with the params it really carries (docs/ipc.md lists the meaning). */
export const NOTICE_SAMPLES: Readonly<Record<string, readonly Params[]>> = {
  claude_waiting_first_session: [{}],
  claude_no_rate_limits: [{}],
  claude_shim_outdated: [{}],
  claude_oauth_partial: reasons(...ERROR_CODES),
  codex_rollout_fallback: reasons(...ERROR_CODES),
  codex_no_limits: [{}],
  antigravity_pool_unknown: ['gemini', 'claude_gpt', 'group_0', 'premium_models'].map((pool) => ({
    pool,
  })),
  litellm_no_budget: [{}],
  litellm_level_unreadable: ['user', 'team'].flatMap((level) =>
    ERROR_CODES.map((reason) => ({ level, reason })),
  ),
  litellm_windows_truncated: ['1', '3', '16', '250'].map((dropped) => ({ dropped })),
  opencode_window_idle: WINDOW_IDS.map((window) => ({ window })),
  opencode_rate_limited: WINDOW_IDS.map((window) => ({ window })),
  opencode_no_key: [{}],
  opencode_local_estimate: [{}],
  opencode_db_unreadable: reasons(...DB_REASONS),
};

export const OPENCODE_DB_REASONS: readonly string[] = DB_REASONS;

/** Fact ids with their unit, per source. */
export const FACTS: Readonly<Record<HarnessId, readonly (readonly [id: string, unit: string])[]>> =
  {
    claude: [],
    codex: [['credits_balance', 'credits']],
    antigravity: [],
    opencode: [
      ['local_cost_5h', 'usd'],
      ['local_cost_7d', 'usd'],
      ['local_cost_30d', 'usd'],
      ['local_tokens_5h', 'tokens'],
      ['local_tokens_7d', 'tokens'],
    ],
    litellm: [
      ['rpm_limit', 'requests'],
      ['tpm_limit', 'tokens'],
      ['max_parallel_requests', 'requests'],
      ['expires', 'epoch_s'],
    ],
  };

/** The `reason` values of failures, per error code. */
export const ERROR_REASONS: Readonly<Record<string, readonly string[]>> = {
  install_failed: [
    'status_line_too_large',
    'path_unsupported',
    'serialize_failed',
    'state_failed',
    'concurrent_modification',
    'write_failed',
    'shim_missing',
    'copy_failed',
    'backup_failed',
    'foreign_status_line',
  ],
  tls_error: ['ca_missing', 'ca_too_large', 'ca_unreadable', 'ca_invalid'],
  endpoint_changed: ['not_an_object', 'no_record', 'id_missing', 'id_mismatch'],
  not_running: ['no_port'],
  unauthorized: ['no_token'],
};

/** The settings fields a rejected command can name (`params.field`). */
export const FIELDS = [
  'value',
  'baseUrl',
  'entityId',
  'authHeader',
  'pemPath',
  'host',
  'configDir',
  'binaryPath',
  'dataDir',
  'shimConsent',
  'limit',
] as const;

/** Window ids the titles depend on, and the source crate whose code must still produce them. */
export const TITLED_IDS: readonly (readonly [crate: string, literal: string])[] = [
  ['litellm', '"budget"'],
  ['litellm', 'limit:{'],
  ['codex', '"primary"'],
  ['codex', '"secondary"'],
  ['claude', '"spend_limit"'],
];

// Catalog keys are namespaced; these families never occur in prose ("settings.json" does).
const KEY_FAMILY =
  /\b(?:fact|facts|notice|window|meta|error|group|gauge|pace|source|field)\.[a-z0-9_.]+/g;
const DOTTED = /\b[a-z]+(?:\.[a-z0-9_]+)+/g;
const SNAKE_CASE = /\b[a-z][a-z0-9]*(?:_[a-z0-9]+)+\b/g;
const PLACEHOLDER = /[{}]|—/;

/** A catalog key (missing or not) or a snake_case id from the wire, printed where words belong. */
export function rawTokens(text: string): string[] {
  const missingKeys = text.match(KEY_FAMILY) ?? [];
  const printedKeys = (text.match(DOTTED) ?? []).filter((candidate) => hasMessage(candidate));
  return [...new Set([...missingKeys, ...printedKeys, ...(text.match(SNAKE_CASE) ?? [])])];
}

/** `rawTokens` plus an unfilled placeholder or the dash that stands for one (a sentence only). */
export function leaks(text: string): string[] {
  return PLACEHOLDER.test(text) ? [...rawTokens(text), 'placeholder'] : rawTokens(text);
}
