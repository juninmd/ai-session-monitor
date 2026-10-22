// Portuguese (Brazil) catalog: the "Canais de cota" surface (what each tool could still tell us).
export const channelsMessages = {
  'channels.page.title': 'O que ainda dá para ler',
  'channels.page.intro':
    'Levantamento de 16/10/2026 nos documentos e no código de cada ferramenta. Cada canal diz qual endpoint o entrega e em que forma ele entra no modelo do app: uma {window} com percentual, valor e reset, ou um {fact} sem limite.',
  'channels.page.counts':
    '{now} canais novos ao alcance da credencial que o app já aceita · {already} confirmados como já lidos · {forbidden} fora das regras do projeto.',
  'channels.candidates.title': 'Fontes candidatas',
  'channels.candidates.note':
    'Ferramentas com canal de uso oficial e uma credencial que caberia no modelo atual.',
  'channels.candidates.credential': 'credencial: {credential}',
  'channels.reach.already': 'Já lemos',
  'channels.reach.now': 'Dá para ler agora',
  'channels.reach.forbidden': 'Existe, mas fora das regras',
  'channels.reach_note.already': 'Em produção hoje.',
  'channels.reach_note.now':
    'Mesma credencial e mesma regra: passaria por SafeClient e viraria janela ou fato. Cada canal não oficial ainda precisa de entrada em docs/data-sources.md e do selo "não oficial" na interface.',
  'channels.reach_note.forbidden':
    'Precisa de token OAuth, cookie, sessão de navegador ou chave-mestra. O app não faz nada disso.',
  'channels.level.high': 'alta',
  'channels.level.medium': 'média',
  'channels.level.low': 'baixa',
  'channels.confidence': 'confiança {level}',
  'channels.form.window': 'janela',
  'channels.form.fact': 'fato',
  'channels.meta': 'confiança {level} · fonte: {source}',
  'channels.source.zai_counts': 'Resposta do endpoint já lido',
  'channels.source.community': 'Comunidade (ccusage)',
  'channels.claude_extra_usage.title': 'Créditos de uso (orçamento mensal)',
  'channels.claude_extra_usage.how':
    'GET api.anthropic.com/api/oauth/usage → {fields}, em centavos. O endpoint já é chamado como fallback quando o CLI falta ou falha; a chave está em KNOWN_KEYS e é descartada hoje.',
  'channels.claude_extra_usage.caveat':
    'Só existe no caminho OAuth opcional (não oficial); exige opt-in e selo.',
  'channels.openrouter_budgets.title': 'Orçamentos por escopo, com reset',
  'channels.openrouter_budgets.how':
    'GET /api/v1/key → {fields} e data.effective_budget. Hoje só lemos o par limit/limit_remaining.',
  'channels.openrouter_free_daily.title': 'Requisições diárias de modelos gratuitos',
  'channels.openrouter_free_daily.how':
    'GET /api/v1/key → {fields}. Janela diária por contagem, com reset de dia UTC.',
  'channels.openrouter_byok.title': 'Gasto BYOK e sua inclusão no limite',
  'channels.openrouter_byok.how':
    'GET /api/v1/key → byok_usage, byok_usage_daily/weekly/monthly, include_byok_in_limit. Explica por que o gasto da chave não bate com o do provedor.',
  'channels.zai_model_usage.title': 'Uso por modelo (24 h)',
  'channels.zai_model_usage.how':
    'GET api.z.ai/api/monitor/usage/model-usage?startTime=&endTime= → total de tokens, total de chamadas e a fatia da janela de 5 h. Header Authorization simples, como no endpoint de quota já lido.',
  'channels.zai_tool_usage.title': 'Uso de ferramentas MCP (24 h)',
  'channels.zai_tool_usage.how':
    'GET api.z.ai/api/monitor/usage/tool-usage?startTime=&endTime= → web_search, web_reader, zread. Complementa o TIME_LIMIT mensal, que hoje aparece só como percentual.',
  'channels.zai_absolute_counts.title': 'Contagens absolutas da cota',
  'channels.zai_absolute_counts.how':
    'A resposta de /quota/limit já traz, além de percentage, os totais de tokens e de MCP ("123/1000"). São descartados: só o percentual chega à interface.',
  'channels.litellm_soft_budget.title': 'Orçamento de alerta (soft_budget)',
  'channels.litellm_soft_budget.how':
    'GET /key/info e /team/info → soft_budget. Não é limite rígido: é o valor em que o proxy avisa. Hoje é ignorado, e o aviso nunca aparece na bandeja.',
  'channels.litellm_temp_budget.title': 'Aumento temporário de orçamento',
  'channels.litellm_temp_budget.how':
    'temp_budget_increase + temp_budget_expiry na chave (e /team/member_update no time). É uma janela de dinheiro com data de fim, que hoje não é lida.',
  'channels.litellm_model_info.title': 'Preços por modelo',
  'channels.litellm_model_info.how':
    'GET /model/info → nomes e input/output_cost_per_token. Permite converter tokens em dinheiro no cliente, em vez de só no proxy.',
  'channels.codex_token_counts.title': 'Tokens da sessão no rollout',
  'channels.codex_token_counts.how':
    '~/.codex/sessions/**/rollout-*.jsonl, evento token_count → total_token_usage, last_token_usage, model_context_window. Já lemos o sub-objeto rate_limits do mesmo evento; os tokens são descartados.',
  'channels.codex_reset_detail.title': 'Detalhe de cada reset ganho',
  'channels.codex_reset_detail.how':
    '{fields}. O app-server expõe, mas hoje só lemos availableCount; a issue #29618 segue sem expor as linhas.',
  'channels.claude_statusline.title': 'Janelas de 5 h e semanal',
  'channels.claude_statusline.how':
    'rate_limits.five_hour e .seven_day (used_percentage, resets_at) no JSON do statusLine, mais spend_limit em dólares.',
  'channels.codex_appserver.title': 'Janelas primária, secundária e créditos',
  'channels.codex_appserver.how':
    'account/rateLimits/read no app-server: janelas nomeadas por windowDurationMins, plano, credits.balance e availableCount dos resets ganhos.',
  'channels.litellm_budget.title': 'Orçamento da chave, do usuário ou do time',
  'channels.litellm_budget.how':
    'GET /key|user|team/info: spend, max_budget, budget_duration, budget_reset_at, orçamentos por modelo e limites estáticos de rpm/tpm.',
  'channels.openrouter_activity.title': 'Gasto diário por modelo',
  'channels.openrouter_activity.how':
    'GET /api/v1/activity. Documentado, mas exige management key: a chave de inferência leva 403 (mesma situação do /credits).',
  'channels.litellm_global_spend.title': 'Gasto global por modelo',
  'channels.litellm_global_spend.how':
    '/global/spend/models e /global/spend/report. Exigem a master key do proxy, não a chave do usuário.',
  'channels.opencode_console_budget.title': 'Orçamento e saldo do Zen',
  'channels.opencode_console_budget.how':
    'Console /api/billing/status, /api/billing/account, /api/usage/cost-by-day. Existem, mas só atrás do login na Console (OAuth), não com a chave colada.',
  'channels.codex_wham.title': 'Limites por recurso e resets bancados',
  'channels.codex_wham.how':
    'chatgpt.com/backend-api/wham/usage e /wham/rate-limit-reset-credits: limite semanal de code review, limites nomeados com janela própria, expiração de resets. É o backend privado do CLI, com auth.json e cabeçalhos próprios.',
  'channels.antigravity_cloudcode.title': 'Quota e plano pela Cloud Code',
  'channels.antigravity_cloudcode.how':
    'POST daily-cloudcode-pa.googleapis.com/v1internal:retrieveUserQuotaSummary e :loadCodeAssist. Exige o token OAuth salvo pela IDE, que o app nunca lê.',
  'channels.kimi_console.title': 'Detalhe de frequência no console',
  'channels.kimi_console.how':
    'O console da plataforma expõe limites por modelo, mas só por sessão de navegador. Sem host novo, sem cookie, sem scraping — fica fora.',
  'channels.cursor_dashboard.title': 'Pools incluídos e gasto sob demanda',
  'channels.cursor_dashboard.how':
    'cursor.com/api/usage-summary e o Connect-RPC GetClientVisibleCreditGrants. Exigem cabeçalhos de navegador e podem devolver 403 na borda.',
  'channels.caveat.new_host':
    'Host e caminho novos: precisa de entrada própria em docs/data-sources.md antes de ser lido.',
} as const satisfies Record<string, string>;
