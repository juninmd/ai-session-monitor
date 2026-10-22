// ru catalog: the "Quota channels" surface (what each tool could still tell us).
export const channelsMessages = {
  'channels.page.title': 'Что ещё можно прочитать',
  'channels.page.intro':
    'Обзор от 16 окт. 2026 по документации и коду каждого инструмента. Для каждого канала указано, какой endpoint его отдаёт и как он вписывается в модель приложения: {window} с процентом, суммой и сбросом либо {fact} без лимита.',
  'channels.page.counts':
    'Новых каналов, доступных с уже принимаемыми приложением учётными данными: {now} · подтверждено как уже читаемых: {already} · вне правил проекта: {forbidden}.',
  'channels.candidates.title': 'Возможные источники',
  'channels.candidates.note':
    'Инструменты с официальным каналом данных об использовании и учётными данными, подходящими под текущую модель.',
  'channels.candidates.credential': 'учётные данные: {credential}',
  'channels.reach.already': 'Уже читается',
  'channels.reach.now': 'Можно читать сейчас',
  'channels.reach.forbidden': 'Есть, но противоречит правилам',
  'channels.reach_note.already': 'Уже работает.',
  'channels.reach_note.now':
    'Те же учётные данные и то же правило: запрос пойдёт через SafeClient и станет окном или фактом. Каждому неофициальному каналу всё равно нужна запись в docs/data-sources.md и значок «неофициальный» в интерфейсе.',
  'channels.reach_note.forbidden':
    'Нужен токен OAuth, cookie, сеанс браузера или мастер-ключ. Приложение этого не делает.',
  'channels.level.high': 'высокая',
  'channels.level.medium': 'средняя',
  'channels.level.low': 'низкая',
  'channels.confidence': 'Достоверность: {level}',
  'channels.form.window': 'окно',
  'channels.form.fact': 'факт',
  'channels.meta': 'Достоверность: {level} · источник: {source}',
  'channels.source.zai_counts': 'Ответ уже читаемого endpoint',
  'channels.source.community': 'Сообщество (ccusage)',
  'channels.claude_extra_usage.title': 'Кредиты использования (месячный бюджет)',
  'channels.claude_extra_usage.how':
    'GET api.anthropic.com/api/oauth/usage → {fields}, в центах. Этот endpoint уже вызывается как запасной вариант, когда CLI отсутствует или завершается с ошибкой; ключ есть в KNOWN_KEYS и сейчас отбрасывается.',
  'channels.claude_extra_usage.caveat':
    'Есть только на необязательном (неофициальном) пути OAuth; нужны явное согласие и значок.',
  'channels.openrouter_budgets.title': 'Бюджеты по областям, со сбросом',
  'channels.openrouter_budgets.how':
    'GET /api/v1/key → {fields} и data.effective_budget. Сейчас читается только пара limit/limit_remaining.',
  'channels.openrouter_free_daily.title': 'Дневные запросы к бесплатным моделям',
  'channels.openrouter_free_daily.how':
    'GET /api/v1/key → {fields}. Дневное окно по числу запросов со сбросом в начале суток UTC.',
  'channels.openrouter_byok.title': 'Расход BYOK и учитывается ли он в лимите',
  'channels.openrouter_byok.how':
    'GET /api/v1/key → byok_usage, byok_usage_daily/weekly/monthly, include_byok_in_limit. Объясняет, почему расход ключа не совпадает с расходом у провайдера.',
  'channels.zai_model_usage.title': 'Расход по моделям (24 ч)',
  'channels.zai_model_usage.how':
    'GET api.z.ai/api/monitor/usage/model-usage?startTime=&endTime= → всего токенов, всего вызовов и доля 5-часового окна. Обычный заголовок Authorization, как у уже читаемого endpoint квот.',
  'channels.zai_tool_usage.title': 'Использование инструментов MCP (24 ч)',
  'channels.zai_tool_usage.how':
    'GET api.z.ai/api/monitor/usage/tool-usage?startTime=&endTime= → web_search, web_reader, zread. Дополняет месячный TIME_LIMIT, который сейчас виден только в процентах.',
  'channels.zai_absolute_counts.title': 'Абсолютные значения квот',
  'channels.zai_absolute_counts.how':
    'Ответ /quota/limit уже содержит, помимо процента, итоги по токенам и MCP («123/1000»). Они отбрасываются: до интерфейса доходит только процент.',
  'channels.litellm_soft_budget.title': 'Бюджет для оповещения (soft_budget)',
  'channels.litellm_soft_budget.how':
    'GET /key/info и /team/info → soft_budget. Это не жёсткий лимит, а значение, при котором прокси предупреждает. Сейчас оно игнорируется, и предупреждение в трее не появляется.',
  'channels.litellm_temp_budget.title': 'Временное увеличение бюджета',
  'channels.litellm_temp_budget.how':
    'temp_budget_increase + temp_budget_expiry у ключа (и /team/member_update у команды). Денежное окно с датой окончания, которое сейчас не читается.',
  'channels.litellm_model_info.title': 'Цены по моделям',
  'channels.litellm_model_info.how':
    'GET /model/info → названия и input/output_cost_per_token. Позволяет клиенту самому переводить токены в деньги, а не полагаться только на прокси.',
  'channels.codex_token_counts.title': 'Токены сеанса в rollout',
  'channels.codex_token_counts.how':
    '~/.codex/sessions/**/rollout-*.jsonl, событие token_count → total_token_usage, last_token_usage, model_context_window. Вложенный объект rate_limits того же события мы уже читаем; токены отбрасываются.',
  'channels.codex_reset_detail.title': 'Подробности каждого заработанного сброса',
  'channels.codex_reset_detail.how':
    '{fields}. Сервер приложения это отдаёт, но сейчас мы читаем только availableCount; в issue #29618 строки пока не раскрываются.',
  'channels.caveat.new_host':
    'Новый хост и путь: перед чтением нужна отдельная запись в docs/data-sources.md.',
  'channels.claude_statusline.title': '5-часовое и недельное окна',
  'channels.claude_statusline.how':
    'rate_limits.five_hour и .seven_day (used_percentage, resets_at) в JSON statusLine, а также spend_limit в долларах.',
  'channels.codex_appserver.title': 'Основное и дополнительное окна, кредиты',
  'channels.codex_appserver.how':
    'account/rateLimits/read на сервере приложения: окна с названиями по windowDurationMins, тариф, credits.balance и availableCount заработанных сбросов.',
  'channels.litellm_budget.title': 'Бюджет ключа, пользователя или команды',
  'channels.litellm_budget.how':
    'GET /key|user|team/info: spend, max_budget, budget_duration, budget_reset_at, бюджеты по моделям и статические лимиты rpm/tpm.',
  'channels.openrouter_activity.title': 'Дневной расход по моделям',
  'channels.openrouter_activity.how':
    'GET /api/v1/activity. Документирован, но нужен ключ управления: ключ для инференса получает 403 (как и в случае /credits).',
  'channels.litellm_global_spend.title': 'Общий расход по моделям',
  'channels.litellm_global_spend.how':
    '/global/spend/models и /global/spend/report. Нужен мастер-ключ прокси, а не ключ пользователя.',
  'channels.opencode_console_budget.title': 'Бюджет и баланс Zen',
  'channels.opencode_console_budget.how':
    'Консоль /api/billing/status, /api/billing/account, /api/usage/cost-by-day. Они существуют, но доступны только после входа в консоль (OAuth), а не по вставленному ключу.',
  'channels.codex_wham.title': 'Лимиты по функциям и накопленные сбросы',
  'channels.codex_wham.how':
    'chatgpt.com/backend-api/wham/usage и /wham/rate-limit-reset-credits: недельный лимит ревью кода, именованные лимиты со своим окном, срок действия сбросов. Это закрытый бэкенд CLI со своим auth.json и заголовками.',
  'channels.antigravity_cloudcode.title': 'Квота и тариф через Cloud Code',
  'channels.antigravity_cloudcode.how':
    'POST daily-cloudcode-pa.googleapis.com/v1internal:retrieveUserQuotaSummary и :loadCodeAssist. Нужен токен OAuth, который сохраняет IDE; приложение его никогда не читает.',
  'channels.kimi_console.title': 'Подробности лимитов в консоли',
  'channels.kimi_console.how':
    'Консоль платформы показывает лимиты по моделям, но только в рамках сеанса браузера. Без нового хоста, cookie и скрейпинга — поэтому остаётся за рамками.',
  'channels.cursor_dashboard.title': 'Включённые пулы и расход по запросу',
  'channels.cursor_dashboard.how':
    'cursor.com/api/usage-summary и Connect-RPC GetClientVisibleCreditGrants. Нужны заголовки браузера, а на границе может вернуться 403.',
} as const satisfies Record<string, string>;
