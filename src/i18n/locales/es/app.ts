// es catalog: app (src/app, App.tsx, and the unit words src/i18n/format.ts uses).
export const appMessages = {
  // Duration units: "1h 53min", "45 min", "4d 11h", "4 s". Short, no plural forms.
  'duration.d': 'd',
  'duration.h': 'h',
  'duration.m': 'min',
  'duration.min': 'min',
  'duration.s': 's',
  // formatClock: after a verb ("se restablece a las 14:35", "el 22/01 a las 09:00").
  'clock.at': 'a las {time}',
  'clock.on_at': 'el {date} a las {time}',
  // formatAmountValue and formatFactValue; {value} is the formatted number.
  'amount.credits.one': '{value} crédito',
  'amount.credits.many': '{value} de créditos',
  'amount.credits.other': '{value} créditos',
  'amount.requests.one': '{value} solicitud',
  'amount.requests.many': '{value} de solicitudes',
  'amount.requests.other': '{value} solicitudes',
  'amount.resets.one': '{value} disponible',
  'amount.resets.many': '{value} disponibles',
  'amount.resets.other': '{value} disponibles',
  'amount.tokens': '{value} tokens',
  'amount.req_short': '{value} sol.',
  // Separator between items of a plain list ("78%, 80%").
  'list.sep': ', ',
  // Copying the sources summary to the clipboard.
  'copy.done': 'Resumen de fuentes copiado al portapapeles.',
  'copy.failed': 'No se pudo copiar el resumen.',
} as const;
