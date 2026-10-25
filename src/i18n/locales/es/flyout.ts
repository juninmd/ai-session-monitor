// es catalog: flyout (onboarding, command palette, provider detail, history board).
export const flyoutMessages = {
  // Onboarding wizard
  'onboarding.step.choose': 'Elegir',
  'onboarding.step.connect': 'Conectar',
  'onboarding.step.verify': 'Verificar',
  'onboarding.step.done': 'Listo',
  'onboarding.choose.title': '¿Qué quieres monitorear?',
  'onboarding.choose.note':
    'Los elementos marcados se encontraron en este equipo. No se lee nada hasta que continúes.',
  'onboarding.tag.responds': 'responde',
  'onboarding.tag.silent': 'sin lectura',
  'onboarding.tag.needs_key': 'necesita una clave',
  'onboarding.tag.needs_consent': 'necesita permiso',
  'onboarding.connect.title': 'Credenciales y permisos',
  'onboarding.connect.consent_note':
    'Esta fuente lee archivos o procesos de este PC. No se lee nada hasta que aceptes.',
  'onboarding.connect.allow': 'Permitir lectura',
  'onboarding.connect.details': 'Ver detalles',
  'onboarding.connect.litellm_note': 'Necesita la dirección del proxy, el alcance y la clave.',
  'onboarding.connect.multi_note':
    'Hay más de una forma de conectar: una clave o la lectura de la base de datos local.',
  'onboarding.connect.configure': 'Configurar',
  'onboarding.connect.nothing':
    'Nada que conectar: las fuentes elegidas ya tienen lo que necesitan.',
  'onboarding.verify.title': 'Qué respondió cada fuente',
  'onboarding.verify.reading': 'Leyendo…',
  'onboarding.verify.read_now': 'Leer ahora',
  'onboarding.verify.first.one': 'una cuota · la primera lleva {pct} usado',
  'onboarding.verify.first.many': '{count} de cuotas · la primera lleva {pct} usado',
  'onboarding.verify.first.other': '{count} cuotas · la primera lleva {pct} usado',
  'onboarding.verify.back_to_key': 'Volver y pegar la clave',
  'onboarding.done.title': 'Monitoreando {done} de {total}',
  'onboarding.done.body':
    'El panel se abre ordenado por urgencia: lo que está más cerca de agotarse aparece primero. Lo que no respondió queda en «Sin lectura», sin cifras inventadas.',
  'onboarding.done.go': 'Ir al panel',
  'onboarding.done.again': 'Leer de nuevo',
  'onboarding.back': 'Atrás',
  'onboarding.continue': 'Continuar con {count}',
  'onboarding.connected': 'Ya conecté',
  'onboarding.finish': 'Finalizar',
  'onboarding.outcome.loading': 'Leyendo…',
  'onboarding.outcome.disabled': 'Apagado',
  'onboarding.outcome.needs_consent': 'Esperando tu permiso',
  'onboarding.outcome.needs_setup': 'Falta la credencial',
  'onboarding.outcome.no_data': 'Aún sin datos',
  'onboarding.outcome.failed': 'La lectura falló',
  'onboarding.outcome.stale': 'Lectura antigua',
  'onboarding.outcome.stale_ago': 'Lectura antigua de hace {age}',
  'onboarding.outcome.no_windows': 'Lee importes, sin cuotas',
  'onboarding.outcome.quotas.one': '{count} cuota',
  'onboarding.outcome.quotas.many': '{count} de cuotas',
  'onboarding.outcome.quotas.other': '{count} cuotas',

  // Ctrl+K palette
  'palette.group.providers': 'Proveedores',
  'palette.group.actions': 'Acciones',
  'palette.group.settings': 'Ajustes',
  'palette.group.demo': 'Demo',
  'palette.refresh.label': 'Actualizar todas las fuentes',
  'palette.refresh.hint': 'leer de nuevo ahora',
  'palette.refresh.keywords': 'actualizar recargar refrescar refresh reload update',
  'palette.copy.label': 'Copiar resumen del estado',
  'palette.copy.hint.one': '{count} fuente como texto',
  'palette.copy.hint.many': '{count} de fuentes como texto',
  'palette.copy.hint.other': '{count} fuentes como texto',
  'palette.copy.keywords': 'portapapeles copiar estado informe clipboard copy status report',
  'palette.general.label': 'Ajustes generales',
  'palette.general.hint': 'pantalla, límites, bandeja',
  'palette.general.keywords': 'preferencias opciones configuración preferences options',
  'palette.configure.label': 'Configurar {name}',
  'palette.configure.hint_stale': 'lectura antigua',
  'palette.configure.hint_default': 'fuente y credenciales',
  'palette.configure.keywords':
    'configurar ajustar fuente credencial configure set up adjust source credential',
  'palette.scenario.label': 'Escenario: {name}',
  'palette.scenario.hint': 'datos ficticios',
  'palette.scenario.keywords': 'simulación demo datos mock demo data',
  'palette.close_aria': 'Cerrar el comando rápido',
  'palette.dialog_aria': 'Comando rápido',
  'palette.input_aria': 'Buscar comandos',
  'palette.placeholder': 'Ir a un proveedor, actualizar, copiar resumen…',
  'palette.empty': 'No se encontró nada',
  'palette.foot.navigate': 'navegar',
  'palette.foot.run': 'ejecutar',
  'palette.key.esc': 'Esc',
  'palette.key.enter': 'Intro',

  // Provider actions (preview only)
  'provider_actions.outcome.reset':
    'Restablecimiento simulado. Cuotas y saldo actualizados en la vista previa.',
  'provider_actions.outcome.noCredit':
    'No hay restablecimientos disponibles. El saldo cambió; actualiza la lectura.',
  'provider_actions.outcome.nothingToReset': 'No hay ninguna cuota que renovar en la vista previa.',
  'provider_actions.outcome.unavailable': 'La acción no está disponible para esta lectura.',
  'provider_actions.resets.one': '{count} restablecimiento disponible',
  'provider_actions.resets.many': '{count} de restablecimientos disponibles',
  'provider_actions.resets.other': '{count} restablecimientos disponibles',
  'provider_actions.use_reset': 'Usar restablecimiento',
  'provider_actions.confirm_aria': 'Confirmar el restablecimiento de la cuota',
  'provider_actions.confirm_body':
    '¿Usar 1 restablecimiento de Codex? Es definitivo. En esta vista previa, la acción es simulada.',
  'provider_actions.confirm': 'Confirmar restablecimiento',
  'provider_actions.cancel': 'Cancelar',
  'provider_actions.reset_failed': 'No se pudo simular el restablecimiento. Inténtalo de nuevo.',
  'provider_actions.low_priority_offer': 'Oferta de baja prioridad · vista previa',
  'provider_actions.copy_command': 'Copiar /low-priority',
  'provider_actions.low_priority_hint':
    'Usa {command} en la sesión interactiva que ofreció este modo.',
  'provider_actions.copied': 'Comando copiado. El modo aún debe activarse en Claude Code.',
  'provider_actions.copy_failed':
    'No se pudo copiar. Selecciona el comando de arriba y cópialo manualmente.',

  // History board and trend
  'history.board.no_readable':
    'Ninguna cuota de esta fuente conserva una lectura con hora de renovación.',
  'history.board.failed': 'No se pudo leer el historial.',
  'history.board.loading': 'Cargando las lecturas…',
  'history.board.empty': 'Aún no hay lecturas guardadas para esta cuota.',
  'history.trend.none': 'Ninguna lectura en esta ventana.',
  'history.trend.first':
    'Pico de {peak} hasta ahora, sin una renovación anterior con la que comparar.',
  'history.trend.compare.one':
    'Pico de {peak} hasta ahora · la renovación anterior cerró en {avg} de media.',
  'history.trend.compare.many':
    'Pico de {peak} hasta ahora · las {count} de renovaciones anteriores cerraron en {avg} de media.',
  'history.trend.compare.other':
    'Pico de {peak} hasta ahora · las {count} renovaciones anteriores cerraron en {avg} de media.',
  'history.trend.projection_desc':
    'La proyección sigue el ritmo actual hasta que se agote en {duration}.',

  // Quota sheet
  'quota.status.ok': 'holgada',
  'quota.status.warn': 'cerca del límite',
  'quota.status.crit': 'crítica',
  'quota.status.out': 'agotada',
  'quota.amount.unlimited': '{used} gastados, sin límite',
  'quota.amount.of': '{used} de {limit}',
  'quota.amount.free': '{free} libres',
  'quota.renew.restarted': 'reiniciada',
  'quota.renew.restarted_at': 'reiniciada a las {time}',
  'quota.renew.in': 'se renueva en {duration}',
  'quota.spoken_left': '{duration} hasta la renovación.',
  'quota.reading.expired': 'Caducada',
  'quota.reading.stale': 'Antigua',
  'quota.spoken_stale': '{name}: lectura desactualizada.',
  'quota.forecast.out': 'Agotada',
  'quota.forecast.risk': 'Se agota en {duration}',
  'quota.forecast.fine': 'Aguanta hasta la renovación',
  'quota.forecast.unknown': 'Datos insuficientes para prever',

  // Provider detail drawer
  'drawer.close': 'Cerrar los detalles',
  'drawer.refresh': 'Actualizar {name}',
  'drawer.pin': 'Fijar {name} arriba',
  'drawer.unpin': 'Dejar de fijar {name}',
  'drawer.settings': 'Configurar {name}',

  // Candidate sources (not wired yet)
  'candidates.amp.credential': 'CLI local (`amp`)',
  'candidates.amp.offer':
    '`amp usage` devuelve el saldo de créditos y el uso incluido. Mismo patrón de CLI fija que ya usan Claude y Antigravity.',
  'candidates.kiro.credential': 'La credencial propia de la CLI',
  'candidates.kiro.offer':
    'GetUsageLimits devuelve un único fondo de créditos con usado, límite, porcentaje y restablecimiento.',
  'candidates.cline.credential': 'Clave de API',
  'candidates.cline.offer':
    '`/users/me/plan/usage-limits`: ventanas de 5 h, semanal y mensual con porcentaje y resetsAt.',
  'candidates.windsurf.credential': 'Clave de servicio del equipo',
  'candidates.windsurf.offer':
    'GetTeamCreditBalance y GetUsageConfig mediante la API oficial. Solo plan Enterprise.',
  'candidates.command_code.credential': 'CLI local (`commandcode`)',
  'candidates.command_code.offer':
    '`/alpha/billing/credits` y `/subscriptions`: ventanas de gasto de 5 h y semanal, plan y créditos mensuales.',
} as const;
