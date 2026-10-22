// One title and one body for every ErrorCode (src/lib/contract/view.ts), plus the OpenCode
// ambiguity wording. src/i18n/catalog.test.ts fails when a code is missing.
export const errorMessages = {
  'error.internal.title': 'Algo salió mal',
  'error.internal.body':
    'Ocurrió un error interno al leer esta fuente. Inténtalo de nuevo; si sigue ocurriendo, reinicia el monitor.',
  'error.timeout.title': 'La lectura tardó demasiado',
  'error.timeout.body': 'La fuente no respondió a tiempo. Inténtalo de nuevo en un momento.',
  'error.network.title': 'No se puede acceder a la fuente',
  'error.network.body': 'No se pudo comunicar con el servidor. Revisa tu red e inténtalo de nuevo.',
  'error.bad_response.title': 'Respuesta inesperada',
  'error.bad_response.body':
    'La fuente respondió algo que el monitor no entiende. Inténtalo de nuevo más tarde.',
  'error.response_too_large.title': 'Respuesta demasiado grande',
  'error.response_too_large.body':
    'La respuesta superó el tamaño aceptado y se descartó por seguridad.',
  'error.rate_limited.title': 'Demasiadas solicitudes',
  'error.rate_limited.body':
    'La fuente pidió una breve espera. El monitor reintentará por su cuenta, con un intervalo mayor.',
  'error.unauthorized.title': 'Clave o inicio de sesión rechazados',
  'error.unauthorized.body':
    'El servidor rechazó las credenciales. Revisa la clave y su alcance en los ajustes.',
  'error.unauthorized_ambiguous.title': 'No se pudo verificar la clave ni la suscripción Go',
  'error.unauthorized_ambiguous.body':
    'El servicio rechazó la solicitud. Esto ocurre con claves válidas y también sin una suscripción Go. Revisa la clave y la suscripción en la consola de OpenCode.',
  'error.unauthorized_ambiguous.kimi.title': 'No se pudo verificar la clave de Kimi Code',
  'error.unauthorized_ambiguous.kimi.body':
    'El servicio rechazó la solicitud. Esto ocurre con claves caducadas y también con una clave de la plataforma abierta de Moonshot, que es otro producto. Comprueba en la consola de Kimi Code que la clave pertenece al plan.',
  'error.unauthorized_ambiguous.zai.title': 'No se pudo verificar la clave de z.ai',
  'error.unauthorized_ambiguous.zai.body':
    'El servicio rechazó la solicitud. Esto ocurre con claves válidas y también con cuentas sin el plan GLM Coding. Revisa la clave y el plan en el panel de z.ai.',
  'error.unauthorized_ambiguous.cursor.title': 'No se pudo verificar el inicio de sesión de Cursor',
  'error.unauthorized_ambiguous.cursor.body':
    'El servicio rechazó la solicitud. Esto ocurre con una sesión caducada o desactualizada y también con una cuenta sin un plan activo. Abre Cursor para renovar el inicio de sesión y revisa el plan en el panel de Cursor.',
  'error.unauthorized_ambiguous.copilot.title': 'No se pudo verificar el token de GitHub',
  'error.unauthorized_ambiguous.copilot.body':
    'GitHub rechazó la solicitud. Esto ocurre con un token caducado, un token de acceso detallado o de una aplicación, y con una cuenta sin Copilot. Revisa el token y tu suscripción de Copilot en los ajustes de GitHub.',
  'error.not_found.title': 'Dirección no encontrada',
  'error.not_found.body':
    'El servidor no reconoce la ruta o el ID indicados. Revisa la URL, el alcance y el ID.',
  'error.server_error.title': 'Error del servidor',
  'error.server_error.body': 'El servidor de la fuente falló. Inténtalo de nuevo más tarde.',
  'error.tls_error.title': 'Fallo del certificado TLS',
  'error.tls_error.body':
    'No se aceptó el certificado del servidor. Elige la CA correcta en los ajustes.',
  'error.invalid_url.title': 'URL no válida',
  'error.invalid_url.body':
    'Usa https://. HTTP solo funciona con localhost, y la URL no puede contener usuario, contraseña, parámetros de consulta ni #.',
  'error.invalid_input.title': 'Valor no válido',
  'error.invalid_input.body': 'Corrige el campo indicado e inténtalo de nuevo.',
  'error.settings_unreadable.title': 'No se puede leer el archivo de ajustes',
  'error.settings_unreadable.body':
    'No se pudo leer settings.json. No se cambió nada. Corrige el archivo e inténtalo de nuevo.',
  'error.install_failed.title': 'Falló la instalación',
  'error.install_failed.body':
    'No se pudo instalar el comando en la línea de estado de Claude Code. No se cambió nada.',
  'error.unsupported_plan.title': 'El plan no tiene límites que leer',
  'error.unsupported_plan.body':
    'La cuenta usa un plan o una clave de API que no informa límites. Puedes desactivar esta fuente.',
  'error.oauth_expired.title': 'El inicio de sesión de Claude Code caducó',
  'error.oauth_expired.body':
    'Abre Claude Code para renovar el inicio de sesión e inténtalo de nuevo.',
  'error.binary_not_found.title': 'Programa no encontrado',
  'error.binary_not_found.body':
    'El monitor no encontró el ejecutable. Instala el programa o indica su ruta en los ajustes.',
  'error.spawn_failed.title': 'No se pudo iniciar el programa',
  'error.spawn_failed.body': 'El proceso no se inició. Revisa la ruta del ejecutable.',
  'error.handshake_failed.title': 'No se pudo comunicar con el programa',
  'error.handshake_failed.body':
    'El programa se inició, pero no respondió como se esperaba. Actualízalo e inténtalo de nuevo.',
  'error.protocol_error.title': 'Formato inesperado',
  'error.protocol_error.body':
    'El formato de los datos ha cambiado. Actualiza el monitor o el programa.',
  'error.not_signed_in.title': 'Cuenta no conectada',
  'error.not_signed_in.body': 'Inicia sesión en tu cuenta desde el programa e inténtalo de nuevo.',
  'error.not_running.title': 'Programa cerrado',
  'error.not_running.body':
    'Esta lectura necesita que el programa esté abierto. Ábrelo e inténtalo de nuevo.',
  'error.endpoint_changed.title': 'La fuente cambió de formato',
  'error.endpoint_changed.body':
    'El servicio cambió y esta lectura ya no funciona como antes. Puede que el monitor necesite una actualización.',
  'error.http': 'Código HTTP: {httpStatus}.',
} as const satisfies Record<string, string>;
