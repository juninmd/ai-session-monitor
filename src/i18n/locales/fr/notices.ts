// Notice codes from docs/ipc.md. An unknown code renders `notice.unknown` with the code in it,
// so a missing translation is visible instead of silent.
export const noticeMessages = {
  'notice.code.claude_waiting_first_session':
    'Commande installée. Les limites arrivent après la première réponse d’une session Claude Code.',
  'notice.code.claude_no_rate_limits':
    'Claude Code ne signale aucune limite (facturation par clé API ou forfait sans limites).',
  'notice.code.claude_shim_outdated':
    'La commande installée dans la ligne d’état de Claude Code est introuvable. Réinstallez-la pour reprendre la lecture.',
  'notice.code.claude_oauth_partial':
    'L’utilisation par modèle (claude /usage ou l’API d’utilisation du compte) est activée, mais elle a échoué ({reason}). Seules les données de la ligne d’état sont affichées.',
  'notice.code.codex_rollout_fallback':
    'Le serveur d’application codex n’a pas répondu ({reason}). Lecture de la dernière session enregistrée ; les chiffres peuvent être anciens.',
  'notice.code.codex_no_limits':
    'Le compte Codex ne signale aucune limite (connexion par clé API).',
  'notice.code.antigravity_pool_unknown':
    'Le quota {pool} n’a renvoyé aucune donnée. Il apparaît comme inconnu, et non comme nul.',
  'notice.code.litellm_no_budget':
    'Cette clé, cet utilisateur ou cette équipe n’a pas de plafond de budget.',
  'notice.code.litellm_level_unreadable':
    'Impossible de lire le budget de niveau {level} ({reason}). Il peut y avoir un plafond qui n’apparaît pas ici.',
  'notice.code.litellm_windows_truncated':
    'Il y a plus de fenêtres de budget que l’espace disponible. Les moins utilisées ont été omises : {dropped}.',
  'notice.code.opencode_window_idle':
    'La fenêtre {window} est à 0 % : elle ne commence à compter qu’à la prochaine requête.',
  'notice.code.opencode_rate_limited': 'La fenêtre {window} est limitée en débit.',
  'notice.code.opencode_no_key':
    'Seule l’estimation locale est affichée. Collez la clé API Go pour voir les vraies fenêtres.',
  'notice.code.opencode_local_estimate':
    'Les valeurs locales sont une estimation au tarif public, à partir de ce PC uniquement. Ce n’est pas le quota.',
  'notice.code.opencode_db_unreadable':
    'Impossible de lire la base de données OpenCode locale : {reason}.',
  'notice.code.copilot_reset_derived':
    'La date de réinitialisation est une estimation : GitHub documente que les requêtes premium sont réinitialisées le 1er de chaque mois, en UTC.',
  'notice.unknown': 'Notification non traduite : {code}',
  'notice.action.reinstall': 'Réinstaller',
  'notice.action.settings': 'Saisir la clé',
  'notice.param.window.rolling': 'de 5 heures',
  'notice.param.window.weekly': 'hebdomadaire',
  'notice.param.window.monthly': 'mensuelle',
  'notice.param.level.key': 'clé',
  'notice.param.level.user': 'utilisateur',
  'notice.param.level.team': 'équipe',
  'notice.param.dropped.one': '{n} fenêtre',
  'notice.param.dropped.many': '{n} de fenêtres',
  'notice.param.dropped.other': '{n} fenêtres',
  // The OpenCode database reasons are their own vocabulary: `not_found` there is a missing file.
  'notice.reason.opencode_db_unreadable.invalid_path':
    'le chemin du dossier de données est invalide',
  'notice.reason.opencode_db_unreadable.not_found': 'le fichier de base de données est introuvable',
  'notice.reason.opencode_db_unreadable.permission':
    'aucune autorisation d’ouvrir le fichier de base de données',
  'notice.reason.opencode_db_unreadable.locked':
    'la base de données est utilisée par un autre programme',
  'notice.reason.opencode_db_unreadable.not_a_database':
    'le fichier trouvé n’est pas une base de données',
  'notice.reason.opencode_db_unreadable.schema': 'le format de la base de données a changé',
  'notice.reason.opencode_db_unreadable.timeout':
    'la lecture de la base de données a pris trop de temps',
  'notice.reason.opencode_db_unreadable.other': 'raison non identifiée',
  'notice.severity.info': 'Info',
  'notice.severity.warn': 'Avertissement',
  'notice.severity.error': 'Erreur',
} as const satisfies Record<string, string>;
