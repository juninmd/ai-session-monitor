// es catalog: the "Quota channels" surface (what each tool could still tell us).
export const channelsMessages = {
  'channels.page.title': 'Lo que aún podemos leer',
  'channels.page.intro':
    'Análisis del 16 de oct. de 2026 sobre la documentación y el código de cada herramienta. Cada canal indica el endpoint que lo entrega y cómo encaja en el modelo de la aplicación: una {window} con porcentaje, importe y restablecimiento, o un {fact} sin límite.',
  'channels.page.counts':
    '{now} canales nuevos al alcance de la credencial que la aplicación ya acepta · {already} confirmados como ya leídos · {forbidden} fuera de las reglas del proyecto.',
  'channels.candidates.title': 'Fuentes candidatas',
  'channels.candidates.note':
    'Herramientas con un canal de uso oficial y una credencial que encajaría en el modelo actual.',
  'channels.candidates.credential': 'credencial: {credential}',
  'channels.reach.already': 'Ya se lee',
  'channels.reach.now': 'Legible ahora',
  'channels.reach.forbidden': 'Existe, pero va contra las reglas',
  'channels.reach_note.already': 'En producción actualmente.',
  'channels.reach_note.now':
    'Misma credencial y misma regla: pasaría por SafeClient y se convertiría en una ventana o un dato. Cada canal no oficial aún necesita una entrada en docs/data-sources.md y la insignia «no oficial» en la interfaz.',
  'channels.reach_note.forbidden':
    'Requiere un token OAuth, una cookie, una sesión del navegador o una clave maestra. La aplicación no hace nada de eso.',
  'channels.level.high': 'alta',
  'channels.level.medium': 'media',
  'channels.level.low': 'baja',
  'channels.confidence': 'Confianza {level}',
  'channels.form.window': 'ventana',
  'channels.form.fact': 'dato',
  'channels.meta': 'Confianza {level} · fuente: {source}',
  'channels.source.zai_counts': 'Respuesta del endpoint que ya se lee',
  'channels.source.community': 'Comunidad (ccusage)',
  'channels.claude_extra_usage.title': 'Créditos de uso (presupuesto mensual)',
  'channels.claude_extra_usage.how':
    'GET api.anthropic.com/api/oauth/usage → {fields}, en centavos. El endpoint ya se llama como alternativa cuando falta la CLI o falla; la clave está en KNOWN_KEYS y hoy se descarta.',
  'channels.claude_extra_usage.caveat':
    'Solo existe en la vía OAuth opcional (no oficial); requiere activación voluntaria y una insignia.',
  'channels.openrouter_budgets.title': 'Presupuestos por alcance, con restablecimiento',
  'channels.openrouter_budgets.how':
    'GET /api/v1/key → {fields} y data.effective_budget. Hoy solo leemos el par limit/limit_remaining.',
  'channels.openrouter_free_daily.title': 'Solicitudes diarias en modelos gratuitos',
  'channels.openrouter_free_daily.how':
    'GET /api/v1/key → {fields}. Una ventana diaria basada en recuentos que se restablece en el día UTC.',
  'channels.openrouter_byok.title': 'Gasto BYOK y si cuenta para el límite',
  'channels.openrouter_byok.how':
    'GET /api/v1/key → byok_usage, byok_usage_daily/weekly/monthly, include_byok_in_limit. Explica por qué el gasto de la clave no coincide con el del proveedor.',
  'channels.zai_model_usage.title': 'Uso por modelo (24 h)',
  'channels.zai_model_usage.how':
    'GET api.z.ai/api/monitor/usage/model-usage?startTime=&endTime= → total de tokens, total de llamadas y la proporción de la ventana de 5 h. Encabezado Authorization simple, como el endpoint de cuota que ya se lee.',
  'channels.zai_tool_usage.title': 'Uso de herramientas MCP (24 h)',
  'channels.zai_tool_usage.how':
    'GET api.z.ai/api/monitor/usage/tool-usage?startTime=&endTime= → web_search, web_reader, zread. Complementa el TIME_LIMIT mensual, que hoy solo se muestra como porcentaje.',
  'channels.zai_absolute_counts.title': 'Recuentos absolutos de la cuota',
  'channels.zai_absolute_counts.how':
    'La respuesta de /quota/limit ya trae, además del porcentaje, los totales de tokens y MCP («123/1000»). Se descartan: a la interfaz solo llega el porcentaje.',
  'channels.litellm_soft_budget.title': 'Presupuesto de alerta (soft_budget)',
  'channels.litellm_soft_budget.how':
    'GET /key/info y /team/info → soft_budget. No es un límite estricto: es el valor en el que el proxy avisa. Hoy se ignora y el aviso nunca aparece en la bandeja.',
  'channels.litellm_temp_budget.title': 'Aumento temporal de presupuesto',
  'channels.litellm_temp_budget.how':
    'temp_budget_increase + temp_budget_expiry en la clave (y /team/member_update en el equipo). Una ventana de dinero con fecha de fin, que hoy no se lee.',
  'channels.litellm_model_info.title': 'Precios por modelo',
  'channels.litellm_model_info.how':
    'GET /model/info → nombres e input/output_cost_per_token. Permite que el cliente convierta tokens en dinero, en lugar de que solo lo haga el proxy.',
  'channels.codex_token_counts.title': 'Tokens de la sesión en el rollout',
  'channels.codex_token_counts.how':
    '~/.codex/sessions/**/rollout-*.jsonl, evento token_count → total_token_usage, last_token_usage, model_context_window. Ya leemos el subobjeto rate_limits del mismo evento; los tokens se descartan.',
  'channels.codex_reset_detail.title': 'Detalle de cada restablecimiento ganado',
  'channels.codex_reset_detail.how':
    '{fields}. El app-server lo expone, pero hoy solo leemos availableCount; la incidencia #29618 aún no expone las filas.',
  'channels.caveat.new_host':
    'Host y ruta nuevos: necesita su propia entrada en docs/data-sources.md antes de leerse.',
  'channels.claude_statusline.title': 'Ventanas de 5 h y semanal',
  'channels.claude_statusline.how':
    'rate_limits.five_hour y .seven_day (used_percentage, resets_at) en el JSON de statusLine, más spend_limit en dólares.',
  'channels.codex_appserver.title': 'Ventanas principal y secundaria, y créditos',
  'channels.codex_appserver.how':
    'account/rateLimits/read en el app-server: ventanas nombradas por windowDurationMins, plan, credits.balance y availableCount de restablecimientos ganados.',
  'channels.litellm_budget.title': 'Presupuesto de la clave, el usuario o el equipo',
  'channels.litellm_budget.how':
    'GET /key|user|team/info: spend, max_budget, budget_duration, budget_reset_at, presupuestos por modelo y límites rpm/tpm estáticos.',
  'channels.openrouter_activity.title': 'Gasto diario por modelo',
  'channels.openrouter_activity.how':
    'GET /api/v1/activity. Documentado, pero requiere una clave de administración: la clave de inferencia recibe un 403 (misma situación que /credits).',
  'channels.litellm_global_spend.title': 'Gasto global por modelo',
  'channels.litellm_global_spend.how':
    '/global/spend/models y /global/spend/report. Necesitan la clave maestra del proxy, no la clave del usuario.',
  'channels.opencode_console_budget.title': 'Presupuesto y saldo de Zen',
  'channels.opencode_console_budget.how':
    'Console /api/billing/status, /api/billing/account, /api/usage/cost-by-day. Existen, pero solo tras el inicio de sesión de la consola (OAuth), no con la clave pegada.',
  'channels.codex_wham.title': 'Límites por función y restablecimientos acumulados',
  'channels.codex_wham.how':
    'chatgpt.com/backend-api/wham/usage y /wham/rate-limit-reset-credits: límite semanal de revisión de código, límites con nombre y ventana propia, caducidad de restablecimientos. Es el backend privado de la CLI, con su propio auth.json y encabezados.',
  'channels.antigravity_cloudcode.title': 'Cuota y plan mediante Cloud Code',
  'channels.antigravity_cloudcode.how':
    'POST daily-cloudcode-pa.googleapis.com/v1internal:retrieveUserQuotaSummary y :loadCodeAssist. Necesita el token OAuth que guarda el IDE, que la aplicación nunca lee.',
  'channels.kimi_console.title': 'Detalle de límites en la consola',
  'channels.kimi_console.how':
    'La consola de la plataforma expone límites por modelo, pero solo por sesión del navegador. Sin host nuevo, sin cookie, sin scraping: se queda fuera.',
  'channels.cursor_dashboard.title': 'Reservas incluidas y gasto bajo demanda',
  'channels.cursor_dashboard.how':
    'cursor.com/api/usage-summary y el Connect-RPC GetClientVisibleCreditGrants. Necesitan encabezados de navegador y pueden devolver 403 en el borde.',
} as const satisfies Record<string, string>;
