// Mirror of crates/usage-model/src/ipc.rs plus the command and event names registered by
// src-tauri. contract/ipc-commands.json pins the names on both sides.
import { z } from 'zod';

export const SecretKindSchema = z.enum(['litellm', 'opencode', 'kimi', 'zai']);
export type SecretKind = z.infer<typeof SecretKindSchema>;

export const SecretStatusSchema = z.strictObject({
  litellm: z.boolean(),
  opencode: z.boolean(),
  kimi: z.boolean(),
  zai: z.boolean(),
});
export type SecretStatus = z.infer<typeof SecretStatusSchema>;

// The UI never passes a URL: the backend maps each link to a fixed destination.
export const ExternalLinkSchema = z.enum([
  'opencode_go_console',
  'opencode_billing',
  'kimi_console',
  'zai_console',
  'project_readme',
]);
export type ExternalLink = z.infer<typeof ExternalLinkSchema>;

export const SHIM_STATES = [
  'not_installed',
  'installed',
  'foreign',
  'outdated',
  'unreadable',
] as const;
export const ShimStateSchema = z.enum(SHIM_STATES);
export type ShimState = z.infer<typeof ShimStateSchema>;

export const ShimStatusSchema = z.strictObject({
  state: ShimStateSchema,
  chained: z.boolean(),
  settingsPath: z.string().nullable(),
});
export type ShimStatus = z.infer<typeof ShimStatusSchema>;

export const COMMANDS = {
  getSnapshot: 'get_snapshot',
  refresh: 'refresh',
  getSettings: 'get_settings',
  saveSettings: 'save_settings',
  secretStatus: 'secret_status',
  setSecret: 'set_secret',
  clearSecret: 'clear_secret',
  claudeShimStatus: 'claude_shim_status',
  installClaudeShim: 'install_claude_shim',
  uninstallClaudeShim: 'uninstall_claude_shim',
  getHistory: 'get_history',
  clearHistory: 'clear_history',
  openLink: 'open_link',
  hideFlyout: 'hide_flyout',
  setFlyoutHeight: 'set_flyout_height',
  quitApp: 'quit_app',
} as const;

export const EVENTS = {
  snapshot: 'snapshot',
  settingsChanged: 'settings-changed',
  flyoutShown: 'flyout-shown',
} as const;
