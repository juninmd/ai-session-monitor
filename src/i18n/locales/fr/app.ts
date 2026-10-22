// French catalog: app (src/app, App.tsx, and the unit words src/i18n/format.ts uses).
export const appMessages = {
  // Duration units: "1 h 53 min", "45 min", "4 j 11 h", "4 s". Short, no plural forms.
  'duration.d': 'j',
  'duration.h': 'h',
  'duration.m': 'min',
  'duration.min': 'min',
  'duration.s': 's',
  // formatClock: after a verb ("resets at 14:35", "resets on 01/22 at 09:00").
  'clock.at': 'à {time}',
  'clock.on_at': 'le {date} à {time}',
  // formatAmountValue and formatFactValue; {value} is the formatted number.
  'amount.credits.one': '{value} crédit',
  'amount.credits.many': '{value} de crédits',
  'amount.credits.other': '{value} crédits',
  'amount.requests.one': '{value} requête',
  'amount.requests.many': '{value} de requêtes',
  'amount.requests.other': '{value} requêtes',
  'amount.resets.one': '{value} disponible',
  'amount.resets.many': '{value} disponibles',
  'amount.resets.other': '{value} disponibles',
  'amount.tokens': '{value} jetons',
  'amount.req_short': '{value} req.',
  // Separator between items of a plain list ("78%, 80%").
  'list.sep': ', ',
  // Copying the sources summary to the clipboard.
  'copy.done': 'Résumé des sources copié dans le presse-papiers.',
  'copy.failed': 'Impossible de copier le résumé.',
} as const;
