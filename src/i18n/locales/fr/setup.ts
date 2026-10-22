// First-run connection surfaces: consent cards, secret fields, OpenCode and LiteLLM forms.
export const setupMessages = {
  'consent.skip': 'Plus tard',
  'consent.details': 'Ce qui sera lu',

  'claude.connect.title': 'Se connecter à Claude Code',
  'claude.connect.lead':
    'Le moniteur ajoute une petite commande à la ligne d’état de Claude Code. Elle n’enregistre que les pourcentages de limite (5 heures et semaine) dans un fichier local. Aucun identifiant n’est lu.',
  'claude.connect.detail':
    'Modifie le settings.json de Claude Code, avec une sauvegarde, et enchaîne votre ligne d’état actuelle. N’écrit que les limites dans %LOCALAPPDATA%. Aucun accès réseau, aucun jeton. Fonctionne avec les comptes Pro et Max. Les chiffres apparaissent après la première réponse d’une session et ne se mettent à jour que tant que Claude Code est ouvert.',
  'claude.connect.allow': 'Installer la commande',
  'claude.connect.tip':
    'Astuce : Windows masque les nouvelles icônes dans le menu de débordement. Faites glisser l’icône du moniteur vers la barre des tâches pour la garder toujours visible.',

  'codex.connect.title': 'Se connecter à Codex',
  'codex.connect.lead':
    'Le moniteur lance le serveur d’application codex installé ici et lui demande ce qu’il reste de la fenêtre de 5 heures et de la semaine.',
  'codex.connect.detail':
    'Exécute le programme codex sur votre machine, en lecture seule. Aucune donnée ne quitte cet ordinateur et l’application ne touche pas au fichier de connexion de Codex. OpenAI qualifie ce serveur d’application d’expérimental : le format peut changer d’une version à l’autre.',
  'codex.connect.allow': 'Autoriser la lecture',

  'antigravity.connect.title': 'Se connecter à Antigravity',
  'antigravity.connect.lead':
    'Le quota provient de l’agy (CLI) installé ou du serveur local de l’IDE. Cette méthode n’est pas officielle et peut cesser de fonctionner lors d’une mise à jour d’Antigravity.',
  'antigravity.connect.detail':
    'Avec votre autorisation, le moniteur peut exécuter l’agy installé uniquement pour lire /usage (sans consommer de jetons ni lire de données de connexion) et/ou lire la ligne de commande du serveur de l’IDE pour obtenir le jeton de session local, en ne communiquant qu’avec 127.0.0.1. Rien n’est écrit sur le disque ni envoyé sur Internet.',
  'antigravity.connect.allow': 'Autoriser la lecture locale',

  'shim.title': 'Commande de ligne d’état',
  'shim.state.not_installed': 'Non installée',
  'shim.state.installed': 'Installée',
  'shim.state.installed_chained': 'Installée, enchaînée avec votre ligne d’état',
  'shim.state.foreign': 'Une autre ligne d’état est utilisée',
  'shim.state.outdated': 'Obsolète : exécutable introuvable',
  'shim.state.unreadable': 'settings.json illisible',
  'shim.hint.foreign':
    'À l’installation, votre ligne d’état actuelle continue de fonctionner : le moniteur l’enchaîne.',
  'shim.hint.unreadable':
    'Le moniteur ne touche pas à un fichier qu’il ne peut pas lire. Corrigez settings.json et vérifiez à nouveau.',
  'shim.path': 'Fichier : {path}',
  'shim.install': 'Installer la commande',
  'shim.reinstall': 'Réinstaller la commande',
  'shim.uninstall': 'Supprimer la commande',
  'shim.recheck': 'Vérifier à nouveau',
  'shim.loading': 'Vérification de Claude Code',

  'secret.saved': 'Clé enregistrée',
  'secret.not_saved': 'Aucune clé enregistrée',
  'secret.placeholder': 'Collez la clé',
  'secret.placeholder_keep': 'Laissez vide pour conserver la clé enregistrée',
  'secret.hint':
    'Reste dans le Gestionnaire d’identifiants Windows et n’apparaît jamais dans l’interface.',
  'secret.save': 'Enregistrer la clé',
  'secret.clear': 'Supprimer la clé',
  'secret.empty': 'Collez la clé API.',
  'secret.saved_ok': 'Clé enregistrée.',
  'secret.cleared_ok': 'Clé supprimée.',

  'opencode.title': 'Se connecter à OpenCode Go',
  'opencode.lead':
    'Collez votre clé API OpenCode Go pour lire les vraies fenêtres (5 heures, semaine et mois). Sans la clé, vous ne verrez qu’une estimation locale de l’utilisation.',
  'opencode.key_label': 'Clé API OpenCode Go',
  'opencode.console': 'Ouvrir la console OpenCode Go',
  'opencode.billing': 'Voir la facturation Zen',
  'opencode.zen_note':
    'Le solde Zen (prépayé) n’a pas d’API : consultez le tableau de bord de facturation.',
  'opencode.local_label': 'Estimation locale (ce PC uniquement)',
  'opencode.local_hint':
    'Ouvre la base de données OpenCode en lecture seule et ne totalise que le coût et les jetons, au tarif public. Ce n’est pas le vrai quota.',
  'opencode.data_dir': 'Dossier de données d’OpenCode',
  'opencode.data_dir_hint': 'Laissez vide pour utiliser le dossier par défaut.',
  'opencode.save_settings': 'Enregistrer le dossier',

  'kimi.title': 'Se connecter à Kimi Code',
  'kimi.lead':
    'Collez votre clé API Kimi Code pour lire le quota du forfait : la fenêtre de 5 heures, la semaine et le total du mois, lorsque le forfait les signale.',
  'kimi.detail':
    'Lecture non officielle : l’API d’utilisation n’est pas documentée et peut changer sans préavis. La clé n’est envoyée qu’à api.kimi.com et reste dans le Gestionnaire d’identifiants Windows. Aucun cookie ni identifiant d’un autre outil n’est utilisé. Une clé de la plateforme ouverte Moonshot est un autre produit et ne fonctionne pas ici.',
  'kimi.key_label': 'Clé API Kimi Code',
  'kimi.console': 'Ouvrir la console Kimi Code',
  'kimi.note':
    'Lecture non officielle de l’API d’utilisation de Kimi Code. La semaine et le mois n’apparaissent que si le forfait les signale.',

  'zai.title': 'Se connecter à z.ai',
  'zai.lead':
    'Collez la clé API de votre forfait GLM Coding pour lire la fenêtre de 5 heures, la semaine et le quota mensuel des outils MCP.',
  'zai.detail':
    'Lecture non officielle : l’API de quota n’est pas documentée et peut changer sans préavis. La clé n’est envoyée qu’à api.z.ai et reste dans le Gestionnaire d’identifiants Windows. Aucun cookie ni identifiant d’un autre outil n’est utilisé. Les comptes BigModel (open.bigmodel.cn) ne sont pas lus.',
  'zai.key_label': 'Clé API z.ai',
  'zai.console': 'Ouvrir les clés z.ai',
  'zai.note':
    'Lecture non officielle de l’API de quota du forfait GLM Coding, sur le serveur mondial (api.z.ai).',

  'openrouter.title': 'Se connecter à OpenRouter',
  'openrouter.lead':
    'Collez votre clé API OpenRouter pour voir la limite de dépenses de la clé, l’utilisation et, si la clé le permet, le solde de crédits.',
  'openrouter.detail':
    'Lecture officielle, via l’API OpenRouter documentée. La clé n’est envoyée qu’à openrouter.ai et reste dans le Gestionnaire d’identifiants Windows. Le solde de crédits nécessite une clé de gestion ; avec une clé ordinaire, seuls la limite de la clé et l’utilisation apparaissent.',
  'openrouter.key_label': 'Clé API OpenRouter',
  'openrouter.console': 'Ouvrir les clés OpenRouter',
  'openrouter.note':
    'Lecture officielle de l’API OpenRouter. Sans limite sur la clé, il n’y a pas de pourcentage, seulement l’utilisation. Le solde n’apparaît qu’avec une clé de gestion.',

  'deepseek.title': 'Se connecter à DeepSeek',
  'deepseek.lead':
    'Collez votre clé API DeepSeek pour voir le solde du compte, dans chaque devise où il a du crédit.',
  'deepseek.detail':
    'Lecture officielle, via l’API DeepSeek documentée. La clé n’est envoyée qu’à api.deepseek.com et reste dans le Gestionnaire d’identifiants Windows. Le solde n’a ni limite ni heure de réinitialisation, donc pas de pourcentage ; les yuans et les dollars ne sont jamais additionnés.',
  'deepseek.key_label': 'Clé API DeepSeek',
  'deepseek.console': 'Ouvrir les clés DeepSeek',
  'deepseek.note':
    'Lecture officielle de l’API de solde DeepSeek. C’est de l’argent, pas un quota : il n’y a ni pourcentage ni réinitialisation.',

  'deepinfra.title': 'Se connecter à DeepInfra',
  'deepinfra.lead':
    'Collez votre clé API DeepInfra pour voir le solde, la limite de dépenses et les dépenses du mois.',
  'deepinfra.detail':
    'Lecture officielle, via l’API de facturation DeepInfra. La clé n’est envoyée qu’à api.deepinfra.com et reste dans le Gestionnaire d’identifiants Windows. Sans limite de dépenses, il n’y a pas de pourcentage, seulement des montants en dollars.',
  'deepinfra.key_label': 'Clé API DeepInfra',
  'deepinfra.console': 'Ouvrir les clés DeepInfra',
  'deepinfra.note':
    'Lecture officielle de l’API de facturation DeepInfra. Les valeurs sont lues en dollars ; sans limite de dépenses, il n’y a pas de pourcentage.',

  'moonshot.title': 'Se connecter à Moonshot',
  'moonshot.lead':
    'Collez votre clé API de la plateforme Moonshot (platform.moonshot.ai) pour voir le solde du compte.',
  'moonshot.detail':
    'Lecture officielle, via l’API de solde documentée de Moonshot. La clé n’est envoyée qu’à api.moonshot.ai et reste dans le Gestionnaire d’identifiants Windows. Les clés de la plateforme kimi.com sont distinctes et ne fonctionnent pas ici. Le solde est de l’argent : il n’y a ni pourcentage ni réinitialisation.',
  'moonshot.key_label': 'Clé API Moonshot',
  'moonshot.console': 'Ouvrir les clés Moonshot',
  'moonshot.note':
    'Lecture officielle de l’API de solde Moonshot, en dollars. Le solde en espèces peut devenir négatif lorsqu’un montant est dû.',

  'novita.title': 'Se connecter à Novita',
  'novita.lead':
    'Collez votre clé API Novita pour voir le solde du compte, la limite de crédit et les frais en attente.',
  'novita.detail':
    'Lecture officielle, via l’API de facturation documentée de Novita. La clé n’est envoyée qu’à api.novita.ai et reste dans le Gestionnaire d’identifiants Windows. Le solde est de l’argent : il n’y a ni pourcentage ni réinitialisation.',
  'novita.key_label': 'Clé API Novita',
  'novita.console': 'Ouvrir les clés Novita',
  'novita.note':
    'Lecture officielle de l’API de solde Novita, en dollars. Le solde peut devenir négatif lorsqu’un montant est dû.',

  'ollama.title': 'Se connecter à Ollama Cloud',
  'ollama.lead':
    'Collez une clé API Ollama Cloud pour voir les crédits inclus dans le forfait et le solde acheté.',
  'ollama.detail':
    'Lecture officielle, via l’API de solde Ollama Cloud documentée. Créez la clé sur ollama.com/settings/keys : la clé d’abonnement de la CLI n’est jamais lue. La clé n’est envoyée qu’à ollama.com et reste dans le Gestionnaire d’identifiants Windows. Le format de la réponse dépend du forfait (crédits mensuels ou fenêtres de session et de semaine).',
  'ollama.key_label': 'Clé API Ollama Cloud',
  'ollama.console': 'Ouvrir les clés Ollama Cloud',
  'ollama.note':
    'Lecture officielle de l’API de solde Ollama Cloud. Les forfaits actuels affichent des crédits en dollars ; les anciens affichent des pourcentages de session et de semaine.',

  'minimax.title': 'Se connecter à MiniMax',
  'minimax.lead':
    'Collez votre clé MiniMax Token Plan pour voir les quotas par modèle : l’intervalle en cours et la semaine.',
  'minimax.detail':
    'Lecture non officielle : la route de quota du Token Plan n’a pas de schéma publié et peut changer sans préavis. La clé n’est envoyée qu’à api.minimax.io (serveur mondial) et reste dans le Gestionnaire d’identifiants Windows. Comparez les chiffres avec la console MiniMax la première fois.',
  'minimax.key_label': 'Clé MiniMax Token Plan',
  'minimax.console': 'Ouvrir les clés MiniMax',
  'minimax.note':
    'Lecture non officielle du quota MiniMax Token Plan, par modèle. Un modèle sans quota dans le forfait n’apparaît pas.',

  'litellm.title': 'Se connecter à LiteLLM',
  'litellm.title_edit': 'Connexion LiteLLM',
  'litellm.url': 'URL du serveur',
  'litellm.url_placeholder': 'https://litellm.votreentreprise.com',
  'litellm.url_hint': 'HTTPS est obligatoire. HTTP n’est accepté que pour localhost.',
  'litellm.url_required': 'Saisissez l’URL du serveur.',
  'litellm.url_invalid': 'Saisissez une URL complète, comme https://litellm.votreentreprise.com.',
  'litellm.url_http': 'Utilisez https://. HTTP ne fonctionne que pour localhost.',
  'litellm.url_parts':
    'Retirez le nom d’utilisateur, le mot de passe, les paramètres de requête et le # de l’URL.',
  'litellm.key': 'Clé API',
  'litellm.key_required': 'Collez la clé API LiteLLM.',
  'litellm.key_rebind':
    'La clé enregistrée ne s’applique qu’à la connexion actuelle. Comme elle a changé, collez à nouveau la clé.',
  'litellm.scope': 'Ce qu’il faut mesurer',
  'litellm.scope.key': 'Cette clé',
  'litellm.scope.user': 'Utilisateur',
  'litellm.scope.team': 'Équipe',
  'litellm.entity.user': 'ID de l’utilisateur',
  'litellm.entity.team': 'ID de l’équipe',
  'litellm.entity_required': 'Saisissez l’ID.',
  'litellm.advanced': 'Avancé',
  'litellm.auth_header': 'En-tête d’authentification',
  'litellm.auth_header_hint':
    'Ne modifiez que si votre proxy utilise son propre en-tête pour la clé.',
  'litellm.auth_header_invalid': 'Utilisez uniquement des lettres, des chiffres et des tirets.',
  'litellm.poll': 'Actualiser toutes les',
  'litellm.timeout': 'Délai d’attente',
  'litellm.tls': 'Certificat TLS',
  'litellm.tls.system': 'Celui de Windows',
  'litellm.tls.custom_ca': 'AC personnalisée',
  'litellm.tls.insecure': 'Ne pas vérifier',
  'litellm.pem': 'Fichier d’AC (.pem)',
  'litellm.pem_placeholder': 'C:\\certs\\ca.pem',
  'litellm.pem_required': 'Saisissez le chemin du fichier .pem.',
  'litellm.insecure_host': 'Serveur sans vérification',
  'litellm.insecure_host_hint': 'Le certificat n’est ignoré que pour ce serveur.',
  'litellm.insecure_host_required': 'Saisissez le serveur.',
  'litellm.insecure_warning':
    'La connexion ne sera pas vérifiée et la clé pourrait être interceptée. À n’utiliser que sur un réseau de confiance.',
  'litellm.insecure_confirm':
    'Je comprends le risque et je souhaite ignorer le certificat de ce serveur.',
  'litellm.insecure_confirm_required': 'Confirmez pour continuer sans vérifier le certificat.',
  'litellm.save': 'Enregistrer et se connecter',
  'litellm.saving': 'Enregistrement…',
  'litellm.saved': 'Paramètres enregistrés. Lecture du serveur…',
  'litellm.edit': 'Modifier la connexion',
  'litellm.delay_note':
    'Les dépenses et la réinitialisation peuvent avoir jusqu’à environ 10 min de retard côté serveur. Le plafond peut porter sur l’utilisateur ou l’équipe, et pas seulement sur la clé.',
} as const satisfies Record<string, string>;
