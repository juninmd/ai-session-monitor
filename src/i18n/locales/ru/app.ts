// ru catalog: app (src/app, App.tsx, and the unit words src/i18n/format.ts uses).
export const appMessages = {
  // Duration units: "1 ч 53 м", "45 мин", "4 д 11 ч", "4 с". Short, no plural forms.
  'duration.d': 'д',
  'duration.h': 'ч',
  'duration.m': 'м',
  'duration.min': 'мин',
  'duration.s': 'с',
  // formatClock: after a verb ("сброс в 14:35", "сброс 01/22 в 09:00").
  'clock.at': 'в {time}',
  'clock.on_at': '{date} в {time}',
  // formatAmountValue and formatFactValue; {value} is the formatted number.
  'amount.credits.one': '{value} кредит',
  'amount.credits.few': '{value} кредита',
  'amount.credits.many': '{value} кредитов',
  'amount.credits.other': '{value} кредита',
  'amount.requests.one': '{value} запрос',
  'amount.requests.few': '{value} запроса',
  'amount.requests.many': '{value} запросов',
  'amount.requests.other': '{value} запроса',
  'amount.resets.one': 'доступно: {value}',
  'amount.resets.few': 'доступно: {value}',
  'amount.resets.many': 'доступно: {value}',
  'amount.resets.other': 'доступно: {value}',
  'amount.tokens': 'токенов: {value}',
  'amount.req_short': 'запр.: {value}',
  // Separator between items of a plain list ("78%, 80%").
  'list.sep': ', ',
  // Copying the sources summary to the clipboard.
  'copy.done': 'Сводка по источникам скопирована в буфер обмена.',
  'copy.failed': 'Не удалось скопировать сводку.',
} as const;
