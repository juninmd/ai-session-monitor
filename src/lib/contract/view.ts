// Mirror of crates/usage-model/src/view.rs. Every object is strict: a field added on the Rust
// side fails parsing here until this file is updated (see contract/snapshot.json).
import { z } from 'zod';

export const HARNESS_IDS = ['claude', 'codex', 'antigravity', 'opencode', 'litellm'] as const;
export const HarnessIdSchema = z.enum(HARNESS_IDS);
export type HarnessId = z.infer<typeof HarnessIdSchema>;

export const VIEW_STATES = [
  'disabled',
  'needs_consent',
  'needs_setup',
  'loading',
  'ready',
  'no_data',
  'error',
] as const;
export const ViewStateSchema = z.enum(VIEW_STATES);
export type ViewState = z.infer<typeof ViewStateSchema>;

export const WINDOW_KINDS = ['five_hour', 'weekly', 'monthly', 'daily', 'budget', 'other'] as const;
export const WindowKindSchema = z.enum(WINDOW_KINDS);
export type WindowKind = z.infer<typeof WindowKindSchema>;

export const AMOUNT_UNITS = ['usd', 'credits', 'requests', 'tokens'] as const;
export const AmountUnitSchema = z.enum(AMOUNT_UNITS);
export type AmountUnit = z.infer<typeof AmountUnitSchema>;

export const AmountSchema = z.strictObject({
  used: z.number(),
  limit: z.number().nullable(),
  unit: AmountUnitSchema,
});
export type Amount = z.infer<typeof AmountSchema>;

export const UsageWindowSchema = z.strictObject({
  id: z.string(),
  kind: WindowKindSchema,
  group: z.string().nullable(),
  label: z.string().nullable(),
  windowMinutes: z.number().int().nonnegative().nullable(),
  // Unknown stays null and must never be shown as 0 %. May exceed 100 when over budget.
  usedPct: z.number().nullable(),
  // Epoch seconds (UTC).
  resetsAt: z.number().int().nullable(),
  amount: AmountSchema.nullable(),
});
export type UsageWindow = z.infer<typeof UsageWindowSchema>;

export const FactSchema = z.strictObject({
  id: z.string(),
  number: z.number().nullable(),
  text: z.string().nullable(),
  unit: z.string().nullable(),
});
export type Fact = z.infer<typeof FactSchema>;

export const SEVERITIES = ['info', 'warn', 'error'] as const;
export const SeveritySchema = z.enum(SEVERITIES);
export type Severity = z.infer<typeof SeveritySchema>;

export const NoticeSchema = z.strictObject({
  code: z.string(),
  severity: SeveritySchema,
  params: z.record(z.string(), z.string()),
});
export type Notice = z.infer<typeof NoticeSchema>;

// Keep in sync with ErrorCode in view.rs; contract/error-codes.json pins both sides.
export const ERROR_CODES = [
  'internal',
  'timeout',
  'network',
  'bad_response',
  'response_too_large',
  'rate_limited',
  'unauthorized',
  'not_found',
  'server_error',
  'tls_error',
  'invalid_url',
  'invalid_input',
  'settings_unreadable',
  'install_failed',
  'unsupported_plan',
  'oauth_expired',
  'binary_not_found',
  'spawn_failed',
  'handshake_failed',
  'protocol_error',
  'not_signed_in',
  'not_running',
  'endpoint_changed',
] as const;
export const ErrorCodeSchema = z.enum(ERROR_CODES);
export type ErrorCode = z.infer<typeof ErrorCodeSchema>;

export const HarnessErrorSchema = z.strictObject({
  code: ErrorCodeSchema,
  retryable: z.boolean(),
  params: z.record(z.string(), z.string()),
});
export type HarnessError = z.infer<typeof HarnessErrorSchema>;

export const SourceSchema = z.strictObject({ id: z.string(), official: z.boolean() });
export type Source = z.infer<typeof SourceSchema>;

export const HarnessViewSchema = z.strictObject({
  id: HarnessIdSchema,
  state: ViewStateSchema,
  source: SourceSchema.nullable(),
  // Epoch seconds at which the data was produced, not when it was polled.
  readAt: z.number().int().nullable(),
  stale: z.boolean(),
  plan: z.string().nullable(),
  windows: z.array(UsageWindowSchema),
  facts: z.array(FactSchema),
  notices: z.array(NoticeSchema),
  error: HarnessErrorSchema.nullable(),
  meta: z.record(z.string(), z.string()),
});
export type HarnessView = z.infer<typeof HarnessViewSchema>;

export const SnapshotSchema = z.strictObject({
  generatedAt: z.number().int(),
  views: z.array(HarnessViewSchema),
});
export type Snapshot = z.infer<typeof SnapshotSchema>;
