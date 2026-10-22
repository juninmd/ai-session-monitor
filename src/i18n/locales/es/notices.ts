// Notice codes from docs/ipc.md. An unknown code renders `notice.unknown` with the code in it,
// so a missing translation is visible instead of silent.
export const noticeMessages = {
  'notice.code.claude_waiting_first_session':
    'Comando instalado. Los límites llegan después de la primera respuesta en una sesión de Claude Code.',
  'notice.code.claude_no_rate_limits':
    'Claude Code no informa límites (facturación por clave de API o un plan sin límites).',
  'notice.code.claude_shim_outdated':
    'No se encontró el comando instalado en la línea de estado de Claude Code. Reinstálalo para retomar la lectura.',
  'notice.code.claude_oauth_partial':
    'El uso por modelo (claude /usage o la API de uso de la cuenta) está activado, pero falló ({reason}). Se muestran solo los datos de la línea de estado.',
  'notice.code.codex_rollout_fallback':
    'El app-server de codex no respondió ({reason}). Se lee la última sesión registrada; las cifras pueden ser antiguas.',
  'notice.code.codex_no_limits':
    'La cuenta de Codex no informa límites (inicio de sesión con clave de API).',
  'notice.code.antigravity_pool_unknown':
    'La cuota {pool} no devolvió datos. Se muestra como desconocida, no como cero.',
  'notice.code.litellm_no_budget': 'Esta clave, usuario o equipo no tiene límite de presupuesto.',
  'notice.code.litellm_level_unreadable':
    'No se pudo leer el presupuesto a nivel de {level} ({reason}). Puede haber un límite que no se ve aquí.',
  'notice.code.litellm_windows_truncated':
    'Hay más ventanas de presupuesto de las que caben aquí. Se omitieron las menos usadas: {dropped}.',
  'notice.code.opencode_window_idle':
    'La ventana {window} está en 0 %: solo empieza a contar con la próxima solicitud.',
  'notice.code.opencode_rate_limited':
    'La ventana {window} tiene el límite de frecuencia alcanzado.',
  'notice.code.opencode_no_key':
    'Se muestra solo la estimación local. Pega la clave de API de Go para ver las ventanas reales.',
  'notice.code.opencode_local_estimate':
    'Los valores locales son una estimación a precio de lista, solo de este PC. No son la cuota.',
  'notice.code.opencode_db_unreadable':
    'No se pudo leer la base de datos local de OpenCode: {reason}.',
  'notice.unknown': 'Aviso sin traducir: {code}',
  'notice.action.reinstall': 'Reinstalar',
  'notice.action.settings': 'Introducir la clave',
  'notice.param.window.rolling': 'de 5 horas',
  'notice.param.window.weekly': 'semanal',
  'notice.param.window.monthly': 'mensual',
  'notice.param.level.key': 'clave',
  'notice.param.level.user': 'usuario',
  'notice.param.level.team': 'equipo',
  'notice.param.dropped.one': '{n} ventana',
  'notice.param.dropped.many': '{n} de ventanas',
  'notice.param.dropped.other': '{n} ventanas',
  // The OpenCode database reasons are their own vocabulary: `not_found` there is a missing file.
  'notice.reason.opencode_db_unreadable.invalid_path':
    'la ruta de la carpeta de datos no es válida',
  'notice.reason.opencode_db_unreadable.not_found': 'no se encontró el archivo de la base de datos',
  'notice.reason.opencode_db_unreadable.permission':
    'sin permiso para abrir el archivo de la base de datos',
  'notice.reason.opencode_db_unreadable.locked': 'la base de datos está en uso por otro programa',
  'notice.reason.opencode_db_unreadable.not_a_database':
    'el archivo encontrado no es una base de datos',
  'notice.reason.opencode_db_unreadable.schema': 'el formato de la base de datos ha cambiado',
  'notice.reason.opencode_db_unreadable.timeout': 'la lectura de la base de datos tardó demasiado',
  'notice.reason.opencode_db_unreadable.other': 'motivo no identificado',
  'notice.severity.info': 'Info',
  'notice.severity.warn': 'Advertencia',
  'notice.severity.error': 'Error',
} as const satisfies Record<string, string>;
