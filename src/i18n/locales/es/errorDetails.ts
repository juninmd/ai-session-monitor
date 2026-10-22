// What the params of a failure add to its message: the `reason` a collector gives (closed
// vocabularies, worded per error code), the sentences for a redirect and a wait, and the names
// of the settings fields. A reason with a `.title` replaces the title of the code as well.
export const errorDetailMessages = {
  'error.install_failed.reason.status_line_too_large.body':
    'El comando actual de tu línea de estado es demasiado grande para que el monitor lo encadene. No se cambió nada.',
  'error.install_failed.reason.path_unsupported.body':
    'La ruta de la carpeta de datos tiene caracteres que el comando de la línea de estado no admite. No se cambió nada.',
  'error.install_failed.reason.serialize_failed.body':
    'No se pudo generar el nuevo settings.json. No se cambió nada.',
  'error.install_failed.reason.state_failed.body':
    'No se pudo guardar el estado de la instalación. settings.json no se modificó.',
  'error.install_failed.reason.concurrent_modification.body':
    'settings.json cambió durante la operación, así que el monitor no lo tocó. Inténtalo de nuevo.',
  'error.install_failed.reason.write_failed.body':
    'No se pudo escribir settings.json. El archivo no se modificó; comprueba que no esté bloqueado e inténtalo de nuevo.',
  'error.install_failed.reason.shim_missing.body':
    'No se encontró el ejecutable del comando junto al monitor. Reinstala el monitor.',
  'error.install_failed.reason.copy_failed.body':
    'No se pudo copiar el ejecutable del comando a la carpeta de datos. settings.json no se modificó.',
  'error.install_failed.reason.backup_failed.body':
    'No se pudo guardar la copia de seguridad de settings.json, así que no se modificó.',
  // Only a removal is refused for this reason.
  'error.install_failed.reason.foreign_status_line.title': 'No se pudo quitar el comando',
  'error.install_failed.reason.foreign_status_line.body':
    'La línea de estado configurada no es la del monitor (quizá una copia editada a mano), así que no se quitó nada.',

  // LiteLLM: the CA file the user chose, not the certificate of the server.
  'error.tls_error.reason.ca_missing.body':
    'No se encontró el archivo de CA configurado. Elige el archivo correcto en los ajustes.',
  'error.tls_error.reason.ca_too_large.body':
    'El archivo de CA es demasiado grande para leerlo. Elige un archivo .pem más pequeño en los ajustes.',
  'error.tls_error.reason.ca_unreadable.body':
    'No se pudo leer el archivo de CA. Revisa la ruta y los permisos en los ajustes.',
  'error.tls_error.reason.ca_invalid.body':
    'El archivo de CA no contiene un certificado válido. Elige un archivo .pem válido en los ajustes.',

  // LiteLLM: which part of the answer did not look like a key, user or team record.
  'error.endpoint_changed.reason.not_an_object.body':
    'La respuesta del servidor no tiene el formato esperado. Comprueba que la URL apunte a LiteLLM.',
  'error.endpoint_changed.reason.no_record.body':
    'La respuesta no incluye ningún registro de clave, usuario o equipo. Revisa la URL y el alcance.',
  'error.endpoint_changed.reason.id_missing.body':
    'La respuesta no confirma el ID consultado. Revisa el ID de usuario o equipo.',
  'error.endpoint_changed.reason.id_mismatch.body':
    'La respuesta corresponde a un ID distinto del indicado. Revisa el ID de usuario o equipo.',

  // Antigravity: the IDE's local server is there, but there is nothing to call.
  'error.not_running.reason.no_port.title': 'Servidor local aún no disponible',
  'error.not_running.reason.no_port.body':
    'Antigravity está abierto, pero su servidor local aún no ofrece una dirección. Suele resolverse en un momento: inténtalo de nuevo.',
  'error.unauthorized.reason.no_token.title': 'Token local no disponible',
  'error.unauthorized.reason.no_token.body':
    'El monitor no pudo leer el token de sesión del servidor local del IDE. Reinicia Antigravity e inténtalo de nuevo.',

  'error.timeout.reason.locked.body':
    'La base de datos local de Cursor está en uso en este momento. Inténtalo de nuevo en un momento.',
  'error.timeout.reason.timeout.body':
    'La lectura de la base de datos local de Cursor tardó demasiado. Inténtalo de nuevo en un momento.',
  'error.endpoint_changed.reason.schema.body':
    'La base de datos local de Cursor tiene una estructura distinta de la que espera el monitor. Puede que el monitor necesite una actualización.',
  'error.settings_unreadable.reason.permission.body':
    'No hay permiso para abrir la base de datos local de Cursor. Revisa los permisos del archivo.',
  'error.settings_unreadable.reason.not_a_database.body':
    'El archivo de Cursor encontrado no es una base de datos.',
  'error.settings_unreadable.reason.other.body':
    'No se pudo leer la base de datos local de Cursor.',
  'error.not_signed_in.reason.not_found.body':
    'No se encontró ningún token de GitHub en la carpeta de la extensión de Copilot. Inicia sesión en Copilot en tu editor o pega un token en los ajustes.',
  'error.not_signed_in.reason.no_token.body':
    'Los archivos de la extensión de Copilot no contienen ningún token para github.com. Inicia sesión en Copilot en tu editor o pega un token en los ajustes.',
  'error.not_signed_in.reason.unreadable.body':
    'No se pudo leer el archivo de inicio de sesión de la extensión de Copilot. Revisa los permisos de su carpeta.',

  'error.redirect':
    'El servidor intentó redirigir la solicitud, y el monitor no sigue redirecciones.',
  'error.retry_after': 'Espera solicitada por la fuente: {wait}.',
  'error.field_hint': 'Campo con problema: {field}.',
  'field.value': 'clave de API',
  'field.baseUrl': 'URL del servidor',
  'field.entityId': 'ID de usuario o equipo',
  'field.authHeader': 'encabezado de autenticación',
  'field.pemPath': 'archivo de CA (.pem)',
  'field.host': 'servidor sin verificación de certificado',
  'field.configDir': 'carpeta de configuración de Claude Code',
  'field.binaryPath': 'ruta del ejecutable de Codex',
  'field.dataDir': 'carpeta de datos de OpenCode',
  'field.shimConsent': 'consentimiento para instalar el comando',
  'field.limit': 'número de ventanas anteriores',
} as const satisfies Record<string, string>;
