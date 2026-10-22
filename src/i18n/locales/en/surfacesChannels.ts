// English catalog: the "Quota channels" surface (what each tool could still tell us).
export const channelsMessages = {
  'channels.page.title': 'What we can still read',
  'channels.page.intro':
    'Survey of Oct 16, 2026 across each tool’s docs and code. Each channel names the endpoint that delivers it and how it fits the app’s model: a {window} with a percentage, amount and reset, or a {fact} with no limit.',
  'channels.page.counts':
    '{now} new channels within reach of the credential the app already accepts · {already} confirmed as already read · {forbidden} outside the project’s rules.',
  'channels.candidates.title': 'Candidate sources',
  'channels.candidates.note':
    'Tools with an official usage channel and a credential that would fit the current model.',
  'channels.candidates.credential': 'credential: {credential}',
  'channels.reach.already': 'Already read',
  'channels.reach.now': 'Readable now',
  'channels.reach.forbidden': 'Exists, but against the rules',
  'channels.reach_note.already': 'In production today.',
  'channels.reach_note.now':
    'Same credential and same rule: it would go through SafeClient and become a window or a fact. Each unofficial channel still needs an entry in docs/data-sources.md and the "unofficial" badge in the interface.',
  'channels.reach_note.forbidden':
    'Needs an OAuth token, a cookie, a browser session or a master key. The app does none of that.',
  'channels.level.high': 'high',
  'channels.level.medium': 'medium',
  'channels.level.low': 'low',
  'channels.confidence': '{level} confidence',
  'channels.form.window': 'window',
  'channels.form.fact': 'fact',
  'channels.meta': '{level} confidence · source: {source}',
  'channels.source.zai_counts': 'Response of the endpoint already read',
  'channels.source.community': 'Community (ccusage)',
  'channels.claude_extra_usage.title': 'Usage credits (monthly budget)',
  'channels.claude_extra_usage.how':
    'GET api.anthropic.com/api/oauth/usage → {fields}, in cents. The endpoint is already called as a fallback when the CLI is missing or fails; the key is in KNOWN_KEYS and is discarded today.',
  'channels.claude_extra_usage.caveat':
    'Only exists on the optional (unofficial) OAuth path; needs opt-in and a badge.',
  'channels.openrouter_budgets.title': 'Budgets by scope, with reset',
  'channels.openrouter_budgets.how':
    'GET /api/v1/key → {fields} and data.effective_budget. Today we only read the limit/limit_remaining pair.',
  'channels.openrouter_free_daily.title': 'Daily requests on free models',
  'channels.openrouter_free_daily.how':
    'GET /api/v1/key → {fields}. A daily count-based window that resets on the UTC day.',
  'channels.openrouter_byok.title': 'BYOK spend and whether it counts toward the limit',
  'channels.openrouter_byok.how':
    'GET /api/v1/key → byok_usage, byok_usage_daily/weekly/monthly, include_byok_in_limit. Explains why the key’s spend does not match the provider’s.',
  'channels.zai_model_usage.title': 'Usage by model (24 h)',
  'channels.zai_model_usage.how':
    'GET api.z.ai/api/monitor/usage/model-usage?startTime=&endTime= → total tokens, total calls and the share of the 5 h window. Plain Authorization header, like the quota endpoint already read.',
  'channels.zai_tool_usage.title': 'MCP tool usage (24 h)',
  'channels.zai_tool_usage.how':
    'GET api.z.ai/api/monitor/usage/tool-usage?startTime=&endTime= → web_search, web_reader, zread. Complements the monthly TIME_LIMIT, which today only shows as a percentage.',
  'channels.zai_absolute_counts.title': 'Absolute quota counts',
  'channels.zai_absolute_counts.how':
    'The /quota/limit response already carries, besides percentage, the token and MCP totals ("123/1000"). They are discarded: only the percentage reaches the interface.',
  'channels.litellm_soft_budget.title': 'Alert budget (soft_budget)',
  'channels.litellm_soft_budget.how':
    'GET /key/info and /team/info → soft_budget. Not a hard limit: it is the value at which the proxy warns. Today it is ignored, and the warning never shows in the tray.',
  'channels.litellm_temp_budget.title': 'Temporary budget increase',
  'channels.litellm_temp_budget.how':
    'temp_budget_increase + temp_budget_expiry on the key (and /team/member_update on the team). A money window with an end date, which is not read today.',
  'channels.litellm_model_info.title': 'Prices per model',
  'channels.litellm_model_info.how':
    'GET /model/info → names and input/output_cost_per_token. Lets the client turn tokens into money, instead of only the proxy doing it.',
  'channels.codex_token_counts.title': 'Session tokens in the rollout',
  'channels.codex_token_counts.how':
    '~/.codex/sessions/**/rollout-*.jsonl, token_count event → total_token_usage, last_token_usage, model_context_window. We already read the rate_limits sub-object of the same event; the tokens are discarded.',
  'channels.codex_reset_detail.title': 'Detail of each earned reset',
  'channels.codex_reset_detail.how':
    '{fields}. The app-server exposes it, but today we only read availableCount; issue #29618 still does not expose the rows.',
  'channels.caveat.new_host':
    'New host and path: needs its own entry in docs/data-sources.md before it is read.',
  'channels.claude_statusline.title': '5 h and weekly windows',
  'channels.claude_statusline.how':
    'rate_limits.five_hour and .seven_day (used_percentage, resets_at) in the statusLine JSON, plus spend_limit in dollars.',
  'channels.codex_appserver.title': 'Primary and secondary windows, and credits',
  'channels.codex_appserver.how':
    'account/rateLimits/read on the app-server: windows named by windowDurationMins, plan, credits.balance and availableCount of earned resets.',
  'channels.litellm_budget.title': 'Budget of the key, user or team',
  'channels.litellm_budget.how':
    'GET /key|user|team/info: spend, max_budget, budget_duration, budget_reset_at, per-model budgets and static rpm/tpm limits.',
  'channels.openrouter_activity.title': 'Daily spend by model',
  'channels.openrouter_activity.how':
    'GET /api/v1/activity. Documented, but needs a management key: the inference key gets a 403 (same situation as /credits).',
  'channels.litellm_global_spend.title': 'Global spend by model',
  'channels.litellm_global_spend.how':
    '/global/spend/models and /global/spend/report. They need the proxy’s master key, not the user’s key.',
  'channels.opencode_console_budget.title': 'Zen budget and balance',
  'channels.opencode_console_budget.how':
    'Console /api/billing/status, /api/billing/account, /api/usage/cost-by-day. They exist, but only behind the Console login (OAuth), not with the pasted key.',
  'channels.codex_wham.title': 'Per-feature limits and banked resets',
  'channels.codex_wham.how':
    'chatgpt.com/backend-api/wham/usage and /wham/rate-limit-reset-credits: weekly code review limit, named limits with their own window, reset expiry. It is the CLI’s private backend, with its own auth.json and headers.',
  'channels.antigravity_cloudcode.title': 'Quota and plan via Cloud Code',
  'channels.antigravity_cloudcode.how':
    'POST daily-cloudcode-pa.googleapis.com/v1internal:retrieveUserQuotaSummary and :loadCodeAssist. Needs the OAuth token the IDE saves, which the app never reads.',
  'channels.kimi_console.title': 'Rate detail in the console',
  'channels.kimi_console.how':
    'The platform console exposes per-model limits, but only per browser session. No new host, no cookie, no scraping — it stays out.',
  'channels.cursor_dashboard.title': 'Included pools and on-demand spend',
  'channels.cursor_dashboard.how':
    'cursor.com/api/usage-summary and the Connect-RPC GetClientVisibleCreditGrants. They need browser headers and may return 403 at the edge.',
} as const satisfies Record<string, string>;
