// Quota history: the "History" section of a panel and its switch in the settings.
export const historyMessages = {
  'history.title': 'Historial',
  'history.window': 'Ventana',
  'history.copy_csv': 'Copiar lecturas como CSV',
  'history.copied': 'Lecturas copiadas como CSV.',
  'history.copy_failed': 'No se pudieron copiar las lecturas.',
  'history.loading': 'Cargando historial',
  'history.failed': 'No se pudo cargar el historial en este momento.',
  'history.empty': 'Aún no hay historial. Las lecturas aparecen aquí mientras el monitor funciona.',
  'history.chart_title': 'Uso de {title} durante la ventana',
  'history.chart_desc':
    'Horizontal: desde el inicio de la ventana hasta el restablecimiento. Vertical: uso de 0 a 100%. La línea gruesa es la ventana actual y las tenues son ventanas anteriores.',
  'history.axis_start': 'inicio',
  'history.axis_end': 'restablecimiento',
  'history.label_current': 'actual {pct}',
  'history.label_past': 'anteriores',
  'history.peak_now': 'Pico de {pct} {when} en esta ventana.',
  'history.no_current': 'Aún no hay lecturas en esta ventana.',
  'history.past_one': 'Ventana anterior: pico de {peaks}.',
  'history.past_many': '{n} ventanas anteriores: picos de {peaks}.',
  'history.table_toggle': 'Ver como tabla',
  'history.table_caption': 'Picos de uso por ventana',
  'history.col_reset': 'Restablecimiento',
  'history.col_peak': 'Pico',
  'history.col_readings': 'Lecturas',
  'history.row_current': '{when} (actual)',

  'settings.history': 'Historial de uso',
  'settings.history_enabled': 'Guardar historial de uso',
  'settings.history_hint':
    'Guarda en este equipo solo los porcentajes y las horas de cada ventana, durante 35 días. Nada sale de aquí. Al desactivarlo se detiene el registro; usa el botón de abajo para borrar.',
  'settings.history_clear': 'Borrar historial',
  'settings.history_clear_confirm':
    '¿Borrar todo el historial guardado? Esta acción no se puede deshacer.',
  'settings.history_clear_yes': 'Borrar',
  'settings.history_clear_no': 'Cancelar',
  'settings.history_cleared': 'Historial borrado.',
} as const satisfies Record<string, string>;
