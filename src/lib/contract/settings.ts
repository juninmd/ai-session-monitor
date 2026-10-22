// Mirror of crates/usage-model/src/settings.rs. Non-secret configuration only: keys live in the
// Windows Credential Manager and are written through the set_secret command.
import { z } from 'zod';
import { HarnessIdSchema } from './view';

export const TimeFormatSchema = z.enum(['hm', 'colon', 'decimal']);
export type TimeFormat = z.infer<typeof TimeFormatSchema>;

export const WeekFormatSchema = z.enum(['hours', 'days_hours']);
export type WeekFormat = z.infer<typeof WeekFormatSchema>;

// BCP 47 tags, the same strings the Rust enum serializes to. English is the default.
export const LanguageSchema = z.enum([
  'en',
  'zh-CN',
  'hi',
  'es',
  'fr',
  'ar',
  'bn',
  'pt-BR',
  'ru',
  'ur',
]);
export type Language = z.infer<typeof LanguageSchema>;

export const GeneralSettingsSchema = z.strictObject({
  intervalSecs: z.number().int(),
  autostart: z.boolean(),
  autoUpdateCheck: z.boolean(),
  traySource: HarnessIdSchema.nullable(),
  pinned: z.array(HarnessIdSchema).max(14),
  warnAtPct: z.number().int(),
  notifyOnOut: z.boolean(),
  notifyOnReset: z.boolean(),
  notifyOnRisk: z.boolean(),
  alertThresholds: z.array(z.number().int().min(1).max(99)).max(4),
  adaptive: z.boolean(),
  historyEnabled: z.boolean(),
  timeFormat: TimeFormatSchema,
  weekFormat: WeekFormatSchema,
  showUsed: z.boolean(),
  compact: z.boolean(),
  firstRunDone: z.boolean(),
  language: LanguageSchema,
  localApi: z.boolean(),
  hideFromCapture: z.boolean(),
});
export type GeneralSettings = z.infer<typeof GeneralSettingsSchema>;

export const ClaudeSettingsSchema = z.strictObject({
  enabled: z.boolean(),
  shimConsent: z.boolean(),
  oauthUsage: z.boolean(),
  configDir: z.string().nullable(),
});
export type ClaudeSettings = z.infer<typeof ClaudeSettingsSchema>;

export const CodexSettingsSchema = z.strictObject({
  enabled: z.boolean(),
  consent: z.boolean(),
  binaryPath: z.string().nullable(),
});
export type CodexSettings = z.infer<typeof CodexSettingsSchema>;

export const AntigravitySettingsSchema = z.strictObject({
  enabled: z.boolean(),
  consent: z.boolean(),
});
export type AntigravitySettings = z.infer<typeof AntigravitySettingsSchema>;

export const CursorSettingsSchema = z.strictObject({
  enabled: z.boolean(),
  consent: z.boolean(),
});
export type CursorSettings = z.infer<typeof CursorSettingsSchema>;

export const CopilotSettingsSchema = z.strictObject({
  enabled: z.boolean(),
  localConsent: z.boolean(),
});
export type CopilotSettings = z.infer<typeof CopilotSettingsSchema>;

export const OpencodeSettingsSchema = z.strictObject({
  enabled: z.boolean(),
  localConsent: z.boolean(),
  dataDir: z.string().nullable(),
});
export type OpencodeSettings = z.infer<typeof OpencodeSettingsSchema>;

export const KimiSettingsSchema = z.strictObject({ enabled: z.boolean() });
export type KimiSettings = z.infer<typeof KimiSettingsSchema>;

export const ZaiSettingsSchema = z.strictObject({ enabled: z.boolean() });
export type ZaiSettings = z.infer<typeof ZaiSettingsSchema>;

export const OpenrouterSettingsSchema = z.strictObject({ enabled: z.boolean() });
export type OpenrouterSettings = z.infer<typeof OpenrouterSettingsSchema>;

export const DeepseekSettingsSchema = z.strictObject({ enabled: z.boolean() });
export type DeepseekSettings = z.infer<typeof DeepseekSettingsSchema>;

export const DeepinfraSettingsSchema = z.strictObject({ enabled: z.boolean() });
export type DeepinfraSettings = z.infer<typeof DeepinfraSettingsSchema>;

export const MoonshotSettingsSchema = z.strictObject({ enabled: z.boolean() });
export type MoonshotSettings = z.infer<typeof MoonshotSettingsSchema>;

export const NovitaSettingsSchema = z.strictObject({ enabled: z.boolean() });
export type NovitaSettings = z.infer<typeof NovitaSettingsSchema>;

export const OllamaSettingsSchema = z.strictObject({ enabled: z.boolean() });
export type OllamaSettings = z.infer<typeof OllamaSettingsSchema>;

export const MinimaxSettingsSchema = z.strictObject({ enabled: z.boolean() });
export type MinimaxSettings = z.infer<typeof MinimaxSettingsSchema>;

export const LITELLM_SCOPES = ['key', 'user', 'team'] as const;
export const LitellmScopeSchema = z.enum(LITELLM_SCOPES);
export type LitellmScope = z.infer<typeof LitellmScopeSchema>;

export const TlsModeSchema = z.discriminatedUnion('mode', [
  z.strictObject({ mode: z.literal('system') }),
  z.strictObject({ mode: z.literal('custom_ca'), pemPath: z.string() }),
  z.strictObject({ mode: z.literal('insecure'), host: z.string() }),
]);
export type TlsMode = z.infer<typeof TlsModeSchema>;

export const LitellmSettingsSchema = z.strictObject({
  enabled: z.boolean(),
  baseUrl: z.string().nullable(),
  scope: LitellmScopeSchema,
  entityId: z.string().nullable(),
  authHeader: z.string(),
  pollIntervalSecs: z.number().int(),
  timeoutSecs: z.number().int(),
  tls: TlsModeSchema,
});
export type LitellmSettings = z.infer<typeof LitellmSettingsSchema>;

export const SettingsSchema = z.strictObject({
  version: z.number().int(),
  general: GeneralSettingsSchema,
  claude: ClaudeSettingsSchema,
  codex: CodexSettingsSchema,
  antigravity: AntigravitySettingsSchema,
  cursor: CursorSettingsSchema,
  copilot: CopilotSettingsSchema,
  opencode: OpencodeSettingsSchema,
  kimi: KimiSettingsSchema,
  zai: ZaiSettingsSchema,
  openrouter: OpenrouterSettingsSchema,
  deepseek: DeepseekSettingsSchema,
  deepinfra: DeepinfraSettingsSchema,
  moonshot: MoonshotSettingsSchema,
  novita: NovitaSettingsSchema,
  ollama: OllamaSettingsSchema,
  minimax: MinimaxSettingsSchema,
  litellm: LitellmSettingsSchema,
});
export type Settings = z.infer<typeof SettingsSchema>;

// The loopback port of the optional local API (usage_model::settings::LOCAL_API_PORT).
export const LOCAL_API_PORT = 6737;

// Ranges the backend clamps to (Settings::normalize); the forms use them for their inputs.
export const LIMITS = {
  intervalSecs: { min: 30, max: 3600 },
  warnAtPct: { min: 5, max: 90 },
  litellmPollSecs: { min: 60, max: 3600 },
  litellmTimeoutSecs: { min: 1, max: 30 },
} as const;

// Must equal Settings::default() in Rust; contract/settings.json pins both.
export const defaultSettings: Settings = {
  version: 1,
  general: {
    intervalSecs: 60,
    autostart: false,
    autoUpdateCheck: true,
    traySource: null,
    pinned: [],
    warnAtPct: 25,
    notifyOnOut: false,
    notifyOnReset: false,
    notifyOnRisk: false,
    alertThresholds: [80, 95],
    adaptive: true,
    historyEnabled: true,
    timeFormat: 'hm',
    weekFormat: 'hours',
    showUsed: false,
    compact: false,
    firstRunDone: false,
    language: 'en',
    localApi: false,
    hideFromCapture: false,
  },
  claude: { enabled: true, shimConsent: false, oauthUsage: false, configDir: null },
  codex: { enabled: true, consent: false, binaryPath: null },
  antigravity: { enabled: true, consent: false },
  cursor: { enabled: true, consent: false },
  copilot: { enabled: true, localConsent: false },
  opencode: { enabled: true, localConsent: false, dataDir: null },
  kimi: { enabled: true },
  zai: { enabled: true },
  openrouter: { enabled: true },
  deepseek: { enabled: true },
  deepinfra: { enabled: true },
  moonshot: { enabled: true },
  novita: { enabled: true },
  ollama: { enabled: true },
  minimax: { enabled: true },
  litellm: {
    enabled: true,
    baseUrl: null,
    scope: 'key',
    entityId: null,
    authHeader: 'Authorization',
    pollIntervalSecs: 300,
    timeoutSecs: 10,
    tls: { mode: 'system' },
  },
};
