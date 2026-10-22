// zh-CN catalog: the "Quota channels" surface (what each tool could still tell us).
export const channelsMessages = {
  'channels.page.title': '还能读到什么',
  'channels.page.intro':
    '基于 2026 年 10 月 16 日对各工具文档和代码的调研。每个渠道都标明了提供数据的端点，以及它如何契合应用的模型：带百分比、金额和重置时间的{window}，或没有上限的{fact}。',
  'channels.page.counts':
    '{now} 个新渠道可用应用已接受的凭据读取 · {already} 个已确认正在读取 · {forbidden} 个不符合项目规则。',
  'channels.candidates.title': '候选数据源',
  'channels.candidates.note': '提供官方用量渠道、且凭据符合当前模型的工具。',
  'channels.candidates.credential': '凭据：{credential}',
  'channels.reach.already': '已在读取',
  'channels.reach.now': '现在可读取',
  'channels.reach.forbidden': '存在，但违反规则',
  'channels.reach_note.already': '目前已在正式使用。',
  'channels.reach_note.now':
    '相同的凭据、相同的规则：会通过 SafeClient 请求，并成为窗口或明细项。每个非官方渠道仍需在 docs/data-sources.md 中登记，并在界面上标注“非官方”徽章。',
  'channels.reach_note.forbidden': '需要 OAuth 令牌、Cookie、浏览器会话或主密钥。应用不会这样做。',
  'channels.level.high': '高',
  'channels.level.medium': '中',
  'channels.level.low': '低',
  'channels.confidence': '可信度{level}',
  'channels.form.window': '窗口',
  'channels.form.fact': '明细项',
  'channels.meta': '可信度{level} · 来源：{source}',
  'channels.source.zai_counts': '已在读取的端点的响应',
  'channels.source.community': '社区（ccusage）',
  'channels.claude_extra_usage.title': '用量积分（月度预算）',
  'channels.claude_extra_usage.how':
    'GET api.anthropic.com/api/oauth/usage → {fields}，单位为美分。当 CLI 缺失或失败时，该端点已作为备用被调用；该键在 KNOWN_KEYS 中，目前被丢弃。',
  'channels.claude_extra_usage.caveat':
    '仅存在于可选的（非官方）OAuth 途径；需要用户主动开启并标注徽章。',
  'channels.openrouter_budgets.title': '按范围划分的预算，含重置',
  'channels.openrouter_budgets.how':
    'GET /api/v1/key → {fields} 和 data.effective_budget。目前只读取 limit/limit_remaining 这一对。',
  'channels.openrouter_free_daily.title': '免费模型的每日请求数',
  'channels.openrouter_free_daily.how':
    'GET /api/v1/key → {fields}。按次数计的每日窗口，在 UTC 日切换时重置。',
  'channels.openrouter_byok.title': 'BYOK 支出，及其是否计入上限',
  'channels.openrouter_byok.how':
    'GET /api/v1/key → byok_usage、byok_usage_daily/weekly/monthly、include_byok_in_limit。可解释为什么密钥的支出与服务商的不一致。',
  'channels.zai_model_usage.title': '按模型的用量（24 小时）',
  'channels.zai_model_usage.how':
    'GET api.z.ai/api/monitor/usage/model-usage?startTime=&endTime= → 总 Token 数、总调用数，以及占 5 小时窗口的比例。使用普通的 Authorization 请求头，与已在读取的额度端点相同。',
  'channels.zai_tool_usage.title': 'MCP 工具用量（24 小时）',
  'channels.zai_tool_usage.how':
    'GET api.z.ai/api/monitor/usage/tool-usage?startTime=&endTime= → web_search、web_reader、zread。补充每月的 TIME_LIMIT，后者目前只以百分比显示。',
  'channels.zai_absolute_counts.title': '额度的绝对数量',
  'channels.zai_absolute_counts.how':
    '/quota/limit 的响应除百分比外，本来就带有 Token 和 MCP 的总量（“123/1000”）。这些数据被丢弃了：只有百分比会进入界面。',
  'channels.litellm_soft_budget.title': '提醒预算（soft_budget）',
  'channels.litellm_soft_budget.how':
    'GET /key/info 和 /team/info → soft_budget。不是硬性上限：而是代理发出警告的阈值。目前被忽略，托盘中也从不显示该警告。',
  'channels.litellm_temp_budget.title': '临时预算提升',
  'channels.litellm_temp_budget.how':
    '密钥上的 temp_budget_increase + temp_budget_expiry（团队则用 /team/member_update）。这是一个有截止日期的金额窗口，目前未读取。',
  'channels.litellm_model_info.title': '各模型的价格',
  'channels.litellm_model_info.how':
    'GET /model/info → 名称和 input/output_cost_per_token。让客户端自己把 Token 换算成金额，而不是只由代理来算。',
  'channels.codex_token_counts.title': 'rollout 中的会话 Token',
  'channels.codex_token_counts.how':
    '~/.codex/sessions/**/rollout-*.jsonl，token_count 事件 → total_token_usage、last_token_usage、model_context_window。同一事件中的 rate_limits 子对象我们已在读取；Token 数据被丢弃。',
  'channels.codex_reset_detail.title': '每次获得的重置的详情',
  'channels.codex_reset_detail.how':
    '{fields}。app-server 提供了这些数据，但目前我们只读取 availableCount；issue #29618 仍未公开明细行。',
  'channels.caveat.new_host': '新的主机和路径：读取之前，需先在 docs/data-sources.md 中单独登记。',
  'channels.claude_statusline.title': '5 小时和每周窗口',
  'channels.claude_statusline.how':
    'statusLine JSON 中的 rate_limits.five_hour 和 .seven_day（used_percentage、resets_at），以及以美元计的 spend_limit。',
  'channels.codex_appserver.title': '主窗口、次窗口和积分',
  'channels.codex_appserver.how':
    'app-server 上的 account/rateLimits/read：按 windowDurationMins 命名的窗口、套餐、credits.balance，以及已获得重置的 availableCount。',
  'channels.litellm_budget.title': '密钥、用户或团队的预算',
  'channels.litellm_budget.how':
    'GET /key|user|team/info：spend、max_budget、budget_duration、budget_reset_at、按模型的预算，以及固定的 rpm/tpm 限制。',
  'channels.openrouter_activity.title': '按模型的每日支出',
  'channels.openrouter_activity.how':
    'GET /api/v1/activity。有公开文档，但需要管理密钥：推理密钥会得到 403（与 /credits 的情况相同）。',
  'channels.litellm_global_spend.title': '按模型的全局支出',
  'channels.litellm_global_spend.how':
    '/global/spend/models 和 /global/spend/report。它们需要代理的主密钥，而不是用户的密钥。',
  'channels.opencode_console_budget.title': 'Zen 预算和余额',
  'channels.opencode_console_budget.how':
    '控制台的 /api/billing/status、/api/billing/account、/api/usage/cost-by-day。它们存在，但只能在控制台登录（OAuth）之后访问，粘贴的密钥无法使用。',
  'channels.codex_wham.title': '按功能的限额和已存的重置',
  'channels.codex_wham.how':
    'chatgpt.com/backend-api/wham/usage 和 /wham/rate-limit-reset-credits：每周代码审查限额、带独立窗口的命名限额、重置到期时间。这是 CLI 的私有后端，有自己的 auth.json 和请求头。',
  'channels.antigravity_cloudcode.title': '通过 Cloud Code 获取额度和套餐',
  'channels.antigravity_cloudcode.how':
    'POST daily-cloudcode-pa.googleapis.com/v1internal:retrieveUserQuotaSummary 和 :loadCodeAssist。需要 IDE 保存的 OAuth 令牌，而应用从不读取它。',
  'channels.kimi_console.title': '控制台中的速率详情',
  'channels.kimi_console.how':
    '平台控制台提供按模型的限额，但仅限于浏览器会话。没有新主机，没有 Cookie，也不抓取网页，因此不纳入。',
  'channels.cursor_dashboard.title': '包含的额度池和按需支出',
  'channels.cursor_dashboard.how':
    'cursor.com/api/usage-summary 和 Connect-RPC 的 GetClientVisibleCreditGrants。它们需要浏览器请求头，并可能在边缘返回 403。',
} as const satisfies Record<string, string>;
