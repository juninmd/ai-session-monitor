// Portuguese (Brazil) catalog: app (src/app, App.tsx, and the unit words src/i18n/format.ts uses).
export const appMessages = {
  // Duration units: "1h 53m", "45 min", "4d 11h", "4 s". Short, no plural forms.
  'duration.d': 'd',
  'duration.h': 'h',
  'duration.m': 'm',
  'duration.min': 'min',
  'duration.s': 's',
  // formatClock: after a verb ("reseta às 14:35", "reseta em 22/01 às 09:00").
  'clock.at': 'às {time}',
  'clock.on_at': 'em {date} às {time}',
  // formatAmountValue and formatFactValue; {value} is the formatted number.
  'amount.credits.one': '{value} crédito',
  'amount.credits.other': '{value} créditos',
  'amount.requests.one': '{value} requisição',
  'amount.requests.other': '{value} requisições',
  'amount.resets.one': '{value} disponível',
  'amount.resets.other': '{value} disponíveis',
  'amount.tokens': '{value} tokens',
  'amount.req_short': '{value} req.',
  // Separator between items of a plain list ("78%, 80%").
  'list.sep': ', ',
  // Copying the sources summary to the clipboard.
  'copy.done': 'Resumo das fontes copiado para a área de transferência.',
  'copy.failed': 'Não foi possível copiar o resumo.',
} as const;
