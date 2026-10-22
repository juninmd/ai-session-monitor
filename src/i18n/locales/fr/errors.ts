// One title and one body for every ErrorCode (src/lib/contract/view.ts), plus the OpenCode
// ambiguity wording. src/i18n/catalog.test.ts fails when a code is missing.
export const errorMessages = {
  'error.internal.title': 'Une erreur est survenue',
  'error.internal.body':
    'Une erreur interne s’est produite pendant la lecture de cette source. Réessayez ; si le problème persiste, redémarrez le moniteur.',
  'error.timeout.title': 'La lecture a pris trop de temps',
  'error.timeout.body': 'La source n’a pas répondu à temps. Réessayez dans un instant.',
  'error.network.title': 'Source inaccessible',
  'error.network.body':
    'Impossible de communiquer avec le serveur. Vérifiez votre réseau et réessayez.',
  'error.bad_response.title': 'Réponse inattendue',
  'error.bad_response.body':
    'La source a renvoyé une réponse que le moniteur ne comprend pas. Réessayez plus tard.',
  'error.response_too_large.title': 'Réponse trop volumineuse',
  'error.response_too_large.body':
    'La réponse dépassait la taille acceptée et a été ignorée par sécurité.',
  'error.rate_limited.title': 'Trop de requêtes',
  'error.rate_limited.body':
    'La source a demandé une courte attente. Le moniteur réessaiera de lui-même, avec un intervalle plus long.',
  'error.unauthorized.title': 'Clé ou connexion refusée',
  'error.unauthorized.body':
    'Le serveur a refusé les identifiants. Vérifiez la clé et sa portée dans les paramètres.',
  'error.unauthorized_ambiguous.title': 'Impossible de vérifier la clé ou l’abonnement Go',
  'error.unauthorized_ambiguous.body':
    'Le service a refusé la requête. Cela arrive avec des clés valides, mais aussi sans abonnement Go. Vérifiez la clé et l’abonnement dans la console OpenCode.',
  'error.unauthorized_ambiguous.kimi.title': 'Impossible de vérifier la clé Kimi Code',
  'error.unauthorized_ambiguous.kimi.body':
    'Le service a refusé la requête. Cela arrive avec des clés expirées, mais aussi avec une clé de la plateforme ouverte Moonshot, qui est un autre produit. Vérifiez dans la console Kimi Code que la clé appartient bien au forfait.',
  'error.unauthorized_ambiguous.zai.title': 'Impossible de vérifier la clé z.ai',
  'error.unauthorized_ambiguous.zai.body':
    'Le service a refusé la requête. Cela arrive avec des clés valides, mais aussi avec des comptes sans le forfait GLM Coding. Vérifiez la clé et le forfait dans le tableau de bord z.ai.',
  'error.unauthorized_ambiguous.cursor.title': 'Impossible de vérifier la connexion à Cursor',
  'error.unauthorized_ambiguous.cursor.body':
    'Le service a refusé la requête. Cela arrive avec une session expirée ou périmée, mais aussi avec un compte sans forfait actif. Ouvrez Cursor pour renouveler la connexion et vérifiez le forfait dans le tableau de bord Cursor.',
  'error.unauthorized_ambiguous.copilot.title': 'Impossible de vérifier le jeton GitHub',
  'error.unauthorized_ambiguous.copilot.body':
    'GitHub a refusé la requête. Cela arrive avec un jeton expiré, un jeton à granularité fine ou d’application, et avec un compte sans Copilot. Vérifiez le jeton et votre abonnement Copilot dans les paramètres GitHub.',
  'error.not_found.title': 'Adresse introuvable',
  'error.not_found.body':
    'Le serveur ne reconnaît pas le chemin ou l’identifiant fourni. Vérifiez l’URL, la portée et l’identifiant.',
  'error.server_error.title': 'Erreur du serveur',
  'error.server_error.body': 'Le serveur de la source a échoué. Réessayez plus tard.',
  'error.tls_error.title': 'Échec du certificat TLS',
  'error.tls_error.body':
    'Le certificat du serveur n’a pas été accepté. Choisissez la bonne autorité de certification dans les paramètres.',
  'error.invalid_url.title': 'URL invalide',
  'error.invalid_url.body':
    'Utilisez https://. HTTP ne fonctionne que pour localhost, et l’URL ne peut contenir ni nom d’utilisateur, ni mot de passe, ni paramètres de requête, ni #.',
  'error.invalid_input.title': 'Valeur invalide',
  'error.invalid_input.body': 'Corrigez le champ indiqué et réessayez.',
  'error.settings_unreadable.title': 'Fichier de paramètres illisible',
  'error.settings_unreadable.body':
    'Impossible de lire settings.json. Rien n’a été modifié. Corrigez le fichier et réessayez.',
  'error.install_failed.title': 'Échec de l’installation',
  'error.install_failed.body':
    'Impossible d’installer la commande dans la ligne d’état de Claude Code. Rien n’a été modifié.',
  'error.unsupported_plan.title': 'Aucune limite à lire pour ce forfait',
  'error.unsupported_plan.body':
    'Le compte utilise un forfait ou une clé API qui ne signale pas de limites. Vous pouvez désactiver cette source.',
  'error.oauth_expired.title': 'Connexion à Claude Code expirée',
  'error.oauth_expired.body': 'Ouvrez Claude Code pour renouveler la connexion, puis réessayez.',
  'error.binary_not_found.title': 'Programme introuvable',
  'error.binary_not_found.body':
    'Le moniteur n’a pas trouvé l’exécutable. Installez le programme ou indiquez son chemin dans les paramètres.',
  'error.spawn_failed.title': 'Impossible de lancer le programme',
  'error.spawn_failed.body': 'Le processus n’a pas démarré. Vérifiez le chemin de l’exécutable.',
  'error.handshake_failed.title': 'Impossible de communiquer avec le programme',
  'error.handshake_failed.body':
    'Le programme a démarré mais n’a pas répondu comme prévu. Mettez-le à jour et réessayez.',
  'error.protocol_error.title': 'Format inattendu',
  'error.protocol_error.body':
    'Le format des données a changé. Mettez à jour le moniteur ou le programme.',
  'error.not_signed_in.title': 'Compte non connecté',
  'error.not_signed_in.body': 'Connectez-vous à votre compte dans le programme, puis réessayez.',
  'error.not_running.title': 'Programme fermé',
  'error.not_running.body':
    'Cette lecture nécessite que le programme soit ouvert. Ouvrez-le et réessayez.',
  'error.endpoint_changed.title': 'La source a changé de format',
  'error.endpoint_changed.body':
    'Le service a changé et cette lecture ne fonctionne plus comme avant. Le moniteur a peut-être besoin d’une mise à jour.',
  'error.http': 'Code HTTP : {httpStatus}.',
} as const satisfies Record<string, string>;
