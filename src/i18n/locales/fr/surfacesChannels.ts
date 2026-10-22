// French catalog: the "Quota channels" surface (what each tool could still tell us).
export const channelsMessages = {
  'channels.page.title': 'Ce que l’on peut encore lire',
  'channels.page.intro':
    'Étude du 16 oct. 2026 sur la documentation et le code de chaque outil. Chaque canal indique le point d’accès qui le fournit et comment il s’intègre au modèle de l’application : une {window} avec pourcentage, montant et réinitialisation, ou un {fact} sans limite.',
  'channels.page.counts':
    '{now} nouveaux canaux à portée de l’identifiant que l’application accepte déjà · {already} confirmés comme déjà lus · {forbidden} hors des règles du projet.',
  'channels.candidates.title': 'Sources candidates',
  'channels.candidates.note':
    'Outils disposant d’un canal d’utilisation officiel et d’un identifiant compatible avec le modèle actuel.',
  'channels.candidates.credential': 'identifiant : {credential}',
  'channels.reach.already': 'Déjà lu',
  'channels.reach.now': 'Lisible maintenant',
  'channels.reach.forbidden': 'Existe, mais contraire aux règles',
  'channels.reach_note.already': 'En production aujourd’hui.',
  'channels.reach_note.now':
    'Même identifiant et même règle : il passerait par SafeClient et deviendrait une fenêtre ou un fait. Chaque canal non officiel nécessite encore une entrée dans docs/data-sources.md et le badge « non officiel » dans l’interface.',
  'channels.reach_note.forbidden':
    'Nécessite un jeton OAuth, un cookie, une session de navigateur ou une clé maîtresse. L’application ne fait rien de tout cela.',
  'channels.level.high': 'élevée',
  'channels.level.medium': 'moyenne',
  'channels.level.low': 'faible',
  'channels.confidence': 'confiance {level}',
  'channels.form.window': 'fenêtre',
  'channels.form.fact': 'fait',
  'channels.meta': 'confiance {level} · source : {source}',
  'channels.source.zai_counts': 'Réponse du point d’accès déjà lu',
  'channels.source.community': 'Communauté (ccusage)',
  'channels.claude_extra_usage.title': 'Crédits d’utilisation (budget mensuel)',
  'channels.claude_extra_usage.how':
    'GET api.anthropic.com/api/oauth/usage → {fields}, en centimes. Le point d’accès est déjà appelé en repli lorsque la CLI est absente ou échoue ; la clé figure dans KNOWN_KEYS et est écartée aujourd’hui.',
  'channels.claude_extra_usage.caveat':
    'N’existe que sur la voie OAuth facultative (non officielle) ; nécessite un consentement et un badge.',
  'channels.openrouter_budgets.title': 'Budgets par portée, avec réinitialisation',
  'channels.openrouter_budgets.how':
    'GET /api/v1/key → {fields} et data.effective_budget. Aujourd’hui, nous ne lisons que la paire limit/limit_remaining.',
  'channels.openrouter_free_daily.title': 'Requêtes quotidiennes sur les modèles gratuits',
  'channels.openrouter_free_daily.how':
    'GET /api/v1/key → {fields}. Une fenêtre quotidienne basée sur un compteur, réinitialisée au jour UTC.',
  'channels.openrouter_byok.title': 'Dépenses BYOK et prise en compte dans la limite',
  'channels.openrouter_byok.how':
    'GET /api/v1/key → byok_usage, byok_usage_daily/weekly/monthly, include_byok_in_limit. Explique pourquoi les dépenses de la clé ne correspondent pas à celles du fournisseur.',
  'channels.zai_model_usage.title': 'Utilisation par modèle (24 h)',
  'channels.zai_model_usage.how':
    'GET api.z.ai/api/monitor/usage/model-usage?startTime=&endTime= → total de jetons, total d’appels et part de la fenêtre de 5 h. En-tête Authorization simple, comme le point d’accès de quota déjà lu.',
  'channels.zai_tool_usage.title': 'Utilisation des outils MCP (24 h)',
  'channels.zai_tool_usage.how':
    'GET api.z.ai/api/monitor/usage/tool-usage?startTime=&endTime= → web_search, web_reader, zread. Complète le TIME_LIMIT mensuel, qui n’apparaît aujourd’hui que sous forme de pourcentage.',
  'channels.zai_absolute_counts.title': 'Compteurs de quota absolus',
  'channels.zai_absolute_counts.how':
    'La réponse de /quota/limit contient déjà, en plus du pourcentage, les totaux de jetons et de MCP (« 123/1000 »). Ils sont écartés : seul le pourcentage atteint l’interface.',
  'channels.litellm_soft_budget.title': 'Budget d’alerte (soft_budget)',
  'channels.litellm_soft_budget.how':
    'GET /key/info et /team/info → soft_budget. Ce n’est pas une limite stricte : c’est la valeur à partir de laquelle le proxy avertit. Aujourd’hui elle est ignorée, et l’avertissement n’apparaît jamais dans la zone de notification.',
  'channels.litellm_temp_budget.title': 'Augmentation temporaire du budget',
  'channels.litellm_temp_budget.how':
    'temp_budget_increase + temp_budget_expiry sur la clé (et /team/member_update pour l’équipe). Une fenêtre monétaire avec une date de fin, qui n’est pas lue aujourd’hui.',
  'channels.litellm_model_info.title': 'Prix par modèle',
  'channels.litellm_model_info.how':
    'GET /model/info → noms et input/output_cost_per_token. Permet au client de convertir les jetons en argent, au lieu de laisser le proxy seul le faire.',
  'channels.codex_token_counts.title': 'Jetons de session dans le rollout',
  'channels.codex_token_counts.how':
    '~/.codex/sessions/**/rollout-*.jsonl, événement token_count → total_token_usage, last_token_usage, model_context_window. Nous lisons déjà le sous-objet rate_limits du même événement ; les jetons sont écartés.',
  'channels.codex_reset_detail.title': 'Détail de chaque réinitialisation gagnée',
  'channels.codex_reset_detail.how':
    '{fields}. Le serveur d’application l’expose, mais nous ne lisons aujourd’hui que availableCount ; le ticket #29618 n’expose pas encore les lignes.',
  'channels.caveat.new_host':
    'Nouvel hôte et nouveau chemin : nécessite sa propre entrée dans docs/data-sources.md avant d’être lu.',
  'channels.claude_statusline.title': 'Fenêtres de 5 h et hebdomadaire',
  'channels.claude_statusline.how':
    'rate_limits.five_hour et .seven_day (used_percentage, resets_at) dans le JSON de statusLine, plus spend_limit en dollars.',
  'channels.codex_appserver.title': 'Fenêtres principale et secondaire, et crédits',
  'channels.codex_appserver.how':
    'account/rateLimits/read sur le serveur d’application : fenêtres nommées par windowDurationMins, forfait, credits.balance et availableCount des réinitialisations gagnées.',
  'channels.litellm_budget.title': 'Budget de la clé, de l’utilisateur ou de l’équipe',
  'channels.litellm_budget.how':
    'GET /key|user|team/info : spend, max_budget, budget_duration, budget_reset_at, budgets par modèle et limites rpm/tpm statiques.',
  'channels.openrouter_activity.title': 'Dépenses quotidiennes par modèle',
  'channels.openrouter_activity.how':
    'GET /api/v1/activity. Documenté, mais nécessite une clé de gestion : la clé d’inférence reçoit un 403 (même situation que /credits).',
  'channels.litellm_global_spend.title': 'Dépenses globales par modèle',
  'channels.litellm_global_spend.how':
    '/global/spend/models et /global/spend/report. Ils nécessitent la clé maîtresse du proxy, pas celle de l’utilisateur.',
  'channels.opencode_console_budget.title': 'Budget et solde Zen',
  'channels.opencode_console_budget.how':
    'Console /api/billing/status, /api/billing/account, /api/usage/cost-by-day. Ils existent, mais uniquement derrière la connexion à la console (OAuth), pas avec la clé collée.',
  'channels.codex_wham.title': 'Limites par fonctionnalité et réinitialisations en réserve',
  'channels.codex_wham.how':
    'chatgpt.com/backend-api/wham/usage et /wham/rate-limit-reset-credits : limite hebdomadaire de revue de code, limites nommées avec leur propre fenêtre, expiration des réinitialisations. C’est le backend privé de la CLI, avec son propre auth.json et ses propres en-têtes.',
  'channels.antigravity_cloudcode.title': 'Quota et forfait via Cloud Code',
  'channels.antigravity_cloudcode.how':
    'POST daily-cloudcode-pa.googleapis.com/v1internal:retrieveUserQuotaSummary et :loadCodeAssist. Nécessite le jeton OAuth que l’IDE enregistre, que l’application ne lit jamais.',
  'channels.kimi_console.title': 'Détail des limites dans la console',
  'channels.kimi_console.how':
    'La console de la plateforme expose des limites par modèle, mais uniquement par session de navigateur. Pas de nouvel hôte, pas de cookie, pas de scraping : cela reste exclu.',
  'channels.cursor_dashboard.title': 'Réserves incluses et dépenses à la demande',
  'channels.cursor_dashboard.how':
    'cursor.com/api/usage-summary et le GetClientVisibleCreditGrants en Connect-RPC. Ils nécessitent des en-têtes de navigateur et peuvent renvoyer un 403 en périphérie.',
} as const satisfies Record<string, string>;
