// Channels that exist but that this app must not read: every one of them needs an OAuth token, a
// cookie, a browser session or a master key. They are listed so the decision is explicit rather
// than an omission.
import { t } from '../i18n';
import type { Channel } from './channels';

export function blockedChannels(): readonly Channel[] {
  return [
    {
      id: 'openrouter-activity',
      harness: 'OpenRouter',
      title: t('channels.openrouter_activity.title'),
      shape: 'fact',
      how: t('channels.openrouter_activity.how'),
      confidence: 'high',
      reach: 'forbidden',
      source: 'openrouter.ai/docs (activity)',
    },
    {
      id: 'litellm-global-spend',
      harness: 'LiteLLM',
      title: t('channels.litellm_global_spend.title'),
      shape: 'fact',
      how: t('channels.litellm_global_spend.how'),
      confidence: 'medium',
      reach: 'forbidden',
      source: 'LiteLLM Swagger',
    },
    {
      id: 'opencode-console-budget',
      harness: 'OpenCode',
      title: t('channels.opencode_console_budget.title'),
      shape: 'window',
      how: t('channels.opencode_console_budget.how'),
      confidence: 'medium',
      reach: 'forbidden',
      source: 'opencode-quota',
    },
    {
      id: 'codex-wham',
      harness: 'Codex',
      title: t('channels.codex_wham.title'),
      shape: 'window',
      how: t('channels.codex_wham.how'),
      confidence: 'medium',
      reach: 'forbidden',
      source: 'ai-usagebar vendor-endpoints',
    },
    {
      id: 'antigravity-cloudcode',
      harness: 'Antigravity',
      title: t('channels.antigravity_cloudcode.title'),
      shape: 'window',
      how: t('channels.antigravity_cloudcode.how'),
      confidence: 'medium',
      reach: 'forbidden',
      source: 'ai-usagebar',
    },
    {
      id: 'kimi-console',
      harness: 'Kimi Code',
      title: t('channels.kimi_console.title'),
      shape: 'fact',
      how: t('channels.kimi_console.how'),
      confidence: 'medium',
      reach: 'forbidden',
      source: 'kimi-cli #2150',
    },
    {
      id: 'cursor-dashboard',
      harness: 'Cursor',
      title: t('channels.cursor_dashboard.title'),
      shape: 'window',
      how: t('channels.cursor_dashboard.how'),
      confidence: 'medium',
      reach: 'forbidden',
      source: 'ai-usagebar',
    },
  ];
}
