// What the params of a failure add to its message: the `reason` a collector gives (closed
// vocabularies, worded per error code), the sentences for a redirect and a wait, and the names
// of the settings fields. A reason with a `.title` replaces the title of the code as well.
export const errorDetailMessages = {
  'error.install_failed.reason.status_line_too_large.body':
    'Votre commande de ligne d’état actuelle est trop volumineuse pour que le moniteur puisse l’enchaîner. Rien n’a été modifié.',
  'error.install_failed.reason.path_unsupported.body':
    'Le chemin du dossier de données contient des caractères que la commande de ligne d’état n’accepte pas. Rien n’a été modifié.',
  'error.install_failed.reason.serialize_failed.body':
    'Impossible de générer le nouveau settings.json. Rien n’a été modifié.',
  'error.install_failed.reason.state_failed.body':
    'Impossible d’enregistrer l’état de l’installation. settings.json n’a pas été modifié.',
  'error.install_failed.reason.concurrent_modification.body':
    'settings.json a changé pendant l’opération ; le moniteur l’a donc laissé tel quel. Réessayez.',
  'error.install_failed.reason.write_failed.body':
    'Impossible d’écrire settings.json. Le fichier n’a pas été modifié ; vérifiez qu’il n’est pas verrouillé et réessayez.',
  'error.install_failed.reason.shim_missing.body':
    'L’exécutable de la commande est introuvable à côté du moniteur. Réinstallez le moniteur.',
  'error.install_failed.reason.copy_failed.body':
    'Impossible de copier l’exécutable de la commande dans le dossier de données. settings.json n’a pas été modifié.',
  'error.install_failed.reason.backup_failed.body':
    'Impossible d’enregistrer la sauvegarde de settings.json, qui n’a donc pas été modifié.',
  // Only a removal is refused for this reason.
  'error.install_failed.reason.foreign_status_line.title': 'Impossible de supprimer la commande',
  'error.install_failed.reason.foreign_status_line.body':
    'La ligne d’état configurée n’est pas celle du moniteur (peut-être une copie modifiée à la main), donc rien n’a été supprimé.',

  // LiteLLM: the CA file the user chose, not the certificate of the server.
  'error.tls_error.reason.ca_missing.body':
    'Le fichier d’autorité de certification configuré est introuvable. Choisissez le bon fichier dans les paramètres.',
  'error.tls_error.reason.ca_too_large.body':
    'Le fichier d’autorité de certification est trop volumineux pour être lu. Choisissez un fichier .pem plus petit dans les paramètres.',
  'error.tls_error.reason.ca_unreadable.body':
    'Impossible de lire le fichier d’autorité de certification. Vérifiez le chemin et les autorisations dans les paramètres.',
  'error.tls_error.reason.ca_invalid.body':
    'Le fichier d’autorité de certification ne contient pas de certificat valide. Choisissez un fichier .pem valide dans les paramètres.',

  // LiteLLM: which part of the answer did not look like a key, user or team record.
  'error.endpoint_changed.reason.not_an_object.body':
    'La réponse du serveur n’a pas le format attendu. Vérifiez que l’URL pointe bien vers LiteLLM.',
  'error.endpoint_changed.reason.no_record.body':
    'La réponse ne contient aucun enregistrement de clé, d’utilisateur ou d’équipe. Vérifiez l’URL et la portée.',
  'error.endpoint_changed.reason.id_missing.body':
    'La réponse ne confirme pas l’identifiant interrogé. Vérifiez l’identifiant de l’utilisateur ou de l’équipe.',
  'error.endpoint_changed.reason.id_mismatch.body':
    'La réponse concerne un autre identifiant que celui fourni. Vérifiez l’identifiant de l’utilisateur ou de l’équipe.',

  // Antigravity: the IDE's local server is there, but there is nothing to call.
  'error.not_running.reason.no_port.title': 'Serveur local pas encore prêt',
  'error.not_running.reason.no_port.body':
    'Antigravity est ouvert, mais son serveur local n’expose pas encore d’adresse. Cela passe généralement en un instant : réessayez.',
  'error.unauthorized.reason.no_token.title': 'Jeton local indisponible',
  'error.unauthorized.reason.no_token.body':
    'Le moniteur n’a pas pu lire le jeton de session du serveur local de l’IDE. Redémarrez Antigravity et réessayez.',
  'error.timeout.reason.locked.body':
    'La base de données locale de Cursor est en cours d’utilisation. Réessayez dans un instant.',
  'error.timeout.reason.timeout.body':
    'La lecture de la base de données locale de Cursor a pris trop de temps. Réessayez dans un instant.',
  'error.endpoint_changed.reason.schema.body':
    'La base de données locale de Cursor n’a pas la structure attendue par le moniteur. Une mise à jour du moniteur peut être nécessaire.',
  'error.settings_unreadable.reason.permission.body':
    'Aucune autorisation pour ouvrir la base de données locale de Cursor. Vérifiez les autorisations du fichier.',
  'error.settings_unreadable.reason.not_a_database.body':
    'Le fichier Cursor trouvé n’est pas une base de données.',
  'error.settings_unreadable.reason.other.body':
    'Impossible de lire la base de données locale de Cursor.',
  'error.not_signed_in.reason.not_found.body':
    'Aucun jeton GitHub trouvé dans le dossier de l’extension Copilot. Connectez-vous à Copilot dans votre éditeur ou collez un jeton dans les paramètres.',
  'error.not_signed_in.reason.no_token.body':
    'Les fichiers de l’extension Copilot ne contiennent aucun jeton pour github.com. Connectez-vous à Copilot dans votre éditeur ou collez un jeton dans les paramètres.',
  'error.not_signed_in.reason.unreadable.body':
    'Impossible de lire le fichier de connexion de l’extension Copilot. Vérifiez les autorisations de son dossier.',

  'error.redirect':
    'Le serveur a tenté de rediriger la requête, et le moniteur ne suit pas les redirections.',
  'error.retry_after': 'Attente demandée par la source : {wait}.',
  'error.field_hint': 'Champ en cause : {field}.',
  'field.value': 'clé API',
  'field.baseUrl': 'URL du serveur',
  'field.entityId': 'identifiant de l’utilisateur ou de l’équipe',
  'field.authHeader': 'en-tête d’authentification',
  'field.pemPath': 'fichier d’autorité de certification (.pem)',
  'field.host': 'serveur sans vérification du certificat',
  'field.configDir': 'dossier de configuration de Claude Code',
  'field.binaryPath': 'chemin de l’exécutable Codex',
  'field.dataDir': 'dossier de données d’OpenCode',
  'field.shimConsent': 'consentement à l’installation de la commande',
  'field.limit': 'nombre de fenêtres précédentes',
} as const satisfies Record<string, string>;
