// French catalog: flyout (onboarding, command palette, provider detail, history board).
export const flyoutMessages = {
  // Onboarding wizard
  'onboarding.step.choose': 'Choisir',
  'onboarding.step.connect': 'Connecter',
  'onboarding.step.verify': 'Vérifier',
  'onboarding.step.done': 'Terminé',
  'onboarding.choose.title': 'Que voulez-vous suivre ?',
  'onboarding.choose.note':
    'Les éléments cochés ont été trouvés sur cette machine. Rien n’est lu avant que vous continuiez.',
  'onboarding.tag.responds': 'répond',
  'onboarding.tag.silent': 'aucun relevé',
  'onboarding.tag.needs_key': 'clé requise',
  'onboarding.tag.needs_consent': 'autorisation requise',
  'onboarding.connect.title': 'Identifiants et autorisations',
  'onboarding.connect.consent_note':
    'Cette source lit des fichiers ou des processus sur ce PC. Rien n’est lu tant que vous n’avez pas validé.',
  'onboarding.connect.allow': 'Autoriser la lecture',
  'onboarding.connect.details': 'Voir les détails',
  'onboarding.connect.litellm_note': 'Nécessite l’adresse du proxy, la portée et la clé.',
  'onboarding.connect.multi_note':
    'Il y a plusieurs façons de se connecter : une clé ou la lecture de la base de données locale.',
  'onboarding.connect.configure': 'Configurer',
  'onboarding.connect.nothing':
    'Rien à connecter : les sources choisies ont déjà tout ce qu’il leur faut.',
  'onboarding.verify.title': 'Ce que chaque source a répondu',
  'onboarding.verify.reading': 'Lecture…',
  'onboarding.verify.read_now': 'Lire maintenant',
  'onboarding.verify.first.one': '{count} quota · le premier est utilisé à {pct}',
  'onboarding.verify.first.many': '{count} de quotas · le premier est utilisé à {pct}',
  'onboarding.verify.first.other': '{count} quotas · le premier est utilisé à {pct}',
  'onboarding.verify.back_to_key': 'Revenir et coller la clé',
  'onboarding.done.title': 'Suivi de {done} sur {total}',
  'onboarding.done.body':
    'Le panneau s’ouvre trié par urgence : ce qui est le plus proche de l’épuisement vient en premier. Tout ce qui n’a pas répondu reste sous « Aucun relevé », sans chiffres inventés.',
  'onboarding.done.go': 'Aller au panneau',
  'onboarding.done.again': 'Relire',
  'onboarding.back': 'Retour',
  'onboarding.continue': 'Continuer avec {count}',
  'onboarding.connected': 'Je me suis connecté',
  'onboarding.finish': 'Terminer',
  'onboarding.outcome.loading': 'Lecture…',
  'onboarding.outcome.disabled': 'Inactif',
  'onboarding.outcome.needs_consent': 'En attente de votre autorisation',
  'onboarding.outcome.needs_setup': 'Identifiant manquant',
  'onboarding.outcome.no_data': 'Pas encore de données',
  'onboarding.outcome.failed': 'Échec de la lecture',
  'onboarding.outcome.stale': 'Relevé ancien',
  'onboarding.outcome.stale_ago': 'Relevé ancien, d’il y a {age}',
  'onboarding.outcome.no_windows': 'Lecture de valeurs, sans quotas',
  'onboarding.outcome.quotas.one': '{count} quota',
  'onboarding.outcome.quotas.many': '{count} de quotas',
  'onboarding.outcome.quotas.other': '{count} quotas',

  // Ctrl+K palette
  'palette.group.providers': 'Fournisseurs',
  'palette.group.actions': 'Actions',
  'palette.group.settings': 'Paramètres',
  'palette.group.demo': 'Démo',
  'palette.refresh.label': 'Actualiser toutes les sources',
  'palette.refresh.hint': 'relire maintenant',
  'palette.refresh.keywords': 'actualiser recharger rafraîchir mettre à jour refresh reload update',
  'palette.copy.label': 'Copier le résumé de l’état',
  'palette.copy.hint.one': '{count} source en texte',
  'palette.copy.hint.many': '{count} de sources en texte',
  'palette.copy.hint.other': '{count} sources en texte',
  'palette.copy.keywords': 'presse-papiers copier état rapport clipboard copy status report',
  'palette.general.label': 'Paramètres généraux',
  'palette.general.hint': 'affichage, limites, zone de notification',
  'palette.general.keywords': 'préférences options preferences',
  'palette.configure.label': 'Configurer {name}',
  'palette.configure.hint_stale': 'relevé ancien',
  'palette.configure.hint_default': 'source et identifiants',
  'palette.configure.keywords':
    'configurer régler source identifiant configure set up adjust credential',
  'palette.scenario.label': 'Scénario : {name}',
  'palette.scenario.hint': 'données fictives',
  'palette.scenario.keywords': 'simulation démo données mock demo data',
  'palette.close_aria': 'Fermer la commande rapide',
  'palette.dialog_aria': 'Commande rapide',
  'palette.input_aria': 'Rechercher des commandes',
  'palette.placeholder': 'Aller à un fournisseur, actualiser, copier le résumé…',
  'palette.empty': 'Aucun résultat',
  'palette.foot.navigate': 'naviguer',
  'palette.foot.run': 'exécuter',
  'palette.key.esc': 'Échap',
  'palette.key.enter': 'Entrée',

  // Provider actions (preview only)
  'provider_actions.outcome.reset':
    'Réinitialisation simulée. Quotas et solde mis à jour dans l’aperçu.',
  'provider_actions.outcome.noCredit':
    'Aucune réinitialisation disponible. Le solde a changé ; actualisez le relevé.',
  'provider_actions.outcome.nothingToReset': 'Il n’y a aucun quota à renouveler dans l’aperçu.',
  'provider_actions.outcome.unavailable': 'L’action n’est pas disponible pour ce relevé.',
  'provider_actions.resets.one': '{count} réinitialisation disponible',
  'provider_actions.resets.many': '{count} de réinitialisations disponibles',
  'provider_actions.resets.other': '{count} réinitialisations disponibles',
  'provider_actions.use_reset': 'Utiliser une réinitialisation',
  'provider_actions.confirm_aria': 'Confirmer la réinitialisation du quota',
  'provider_actions.confirm_body':
    'Utiliser 1 réinitialisation Codex ? Elle est définitive. Dans cet aperçu, l’action est simulée.',
  'provider_actions.confirm': 'Confirmer la réinitialisation',
  'provider_actions.cancel': 'Annuler',
  'provider_actions.reset_failed': 'Impossible de simuler la réinitialisation. Réessayez.',
  'provider_actions.low_priority_offer': 'Offre à basse priorité · aperçu',
  'provider_actions.copy_command': 'Copier /low-priority',
  'provider_actions.low_priority_hint':
    'Utilisez {command} dans la session interactive qui a proposé ce mode.',
  'provider_actions.copied': 'Commande copiée. Le mode doit encore être activé dans Claude Code.',
  'provider_actions.copy_failed':
    'Impossible de copier. Sélectionnez la commande ci-dessus et copiez-la manuellement.',

  // History board and trend
  'history.board.no_readable':
    'Aucun des quotas de cette source ne conserve de relevé avec une heure de renouvellement.',
  'history.board.failed': 'Impossible de lire l’historique.',
  'history.board.loading': 'Chargement des relevés…',
  'history.board.empty': 'Aucun relevé enregistré pour ce quota pour l’instant.',
  'history.trend.none': 'Aucun relevé dans cette fenêtre.',
  'history.trend.first': 'Pic de {peak} jusqu’ici, sans renouvellement antérieur auquel comparer.',
  'history.trend.compare.one':
    'Pic de {peak} jusqu’ici · le renouvellement précédent s’est terminé à {avg} en moyenne.',
  'history.trend.compare.many':
    'Pic de {peak} jusqu’ici · les {count} de renouvellements précédents se sont terminés à {avg} en moyenne.',
  'history.trend.compare.other':
    'Pic de {peak} jusqu’ici · les {count} renouvellements précédents se sont terminés à {avg} en moyenne.',
  'history.trend.projection_desc':
    'La projection suit le rythme actuel jusqu’à l’épuisement dans {duration}.',

  // Quota sheet
  'quota.status.ok': 'confortable',
  'quota.status.warn': 'proche de la limite',
  'quota.status.crit': 'critique',
  'quota.status.out': 'épuisé',
  'quota.amount.unlimited': '{used} dépensé, sans plafond',
  'quota.amount.of': '{used} sur {limit}',
  'quota.amount.free': '{free} gratuit',
  'quota.renew.restarted': 'réinitialisé',
  'quota.renew.restarted_at': 'réinitialisé à {time}',
  'quota.renew.in': 'renouvellement dans {duration}',
  'quota.spoken_left': '{duration} avant le renouvellement.',
  'quota.reading.expired': 'Expiré',
  'quota.reading.stale': 'Ancien',
  'quota.spoken_stale': '{name} : relevé périmé.',
  'quota.forecast.out': 'Épuisé',
  'quota.forecast.risk': 'Épuisé dans {duration}',
  'quota.forecast.fine': 'Tient jusqu’au renouvellement',
  'quota.forecast.unknown': 'Pas assez de données pour prévoir',

  // Provider detail drawer
  'drawer.close': 'Fermer les détails',
  'drawer.refresh': 'Actualiser {name}',
  'drawer.pin': 'Épingler {name} en haut',
  'drawer.unpin': 'Désépingler {name}',
  'drawer.settings': 'Configurer {name}',

  // Candidate sources (not wired yet)
  'candidates.amp.credential': 'CLI locale (`amp`)',
  'candidates.amp.offer':
    '`amp usage` renvoie le solde de crédits et l’utilisation incluse. Même schéma de CLI fixe que celui déjà utilisé par Claude et Antigravity.',
  'candidates.kiro.credential': 'L’identifiant propre à la CLI',
  'candidates.kiro.offer':
    'GetUsageLimits renvoie un seul pool de crédits avec utilisé, limite, pourcentage et réinitialisation.',
  'candidates.cline.credential': 'Clé API',
  'candidates.cline.offer':
    '`/users/me/plan/usage-limits` : fenêtres de 5 h, hebdomadaire et mensuelle avec pourcentage et resetsAt.',
  'candidates.windsurf.credential': 'Clé de service d’équipe',
  'candidates.windsurf.offer':
    'GetTeamCreditBalance et GetUsageConfig via l’API officielle. Forfait Enterprise uniquement.',
  'candidates.command_code.credential': 'CLI locale (`commandcode`)',
  'candidates.command_code.offer':
    '`/alpha/billing/credits` et `/subscriptions` : fenêtres de dépenses de 5 h et hebdomadaire, forfait et crédits mensuels.',
} as const;
