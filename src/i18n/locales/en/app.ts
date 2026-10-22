// English catalog: app (src/app, App.tsx, and the unit words src/i18n/format.ts uses).
export const appMessages = {
  // Duration units: "1h 53m", "45 min", "4d 11h", "4 s". Short, no plural forms.
  'duration.d': 'd',
  'duration.h': 'h',
  'duration.m': 'm',
  'duration.min': 'min',
  'duration.s': 's',
  // formatClock: after a verb ("resets at 14:35", "resets on 01/22 at 09:00").
  'clock.at': 'at {time}',
  'clock.on_at': 'on {date} at {time}',
  // formatAmountValue and formatFactValue; {value} is the formatted number.
  'amount.credits.one': '{value} credit',
  'amount.credits.other': '{value} credits',
  'amount.requests.one': '{value} request',
  'amount.requests.other': '{value} requests',
  'amount.resets.one': '{value} available',
  'amount.resets.other': '{value} available',
  'amount.tokens': '{value} tokens',
  'amount.req_short': '{value} req.',
  // Separator between items of a plain list ("78%, 80%").
  'list.sep': ', ',
  // Copying the sources summary to the clipboard.
  'copy.done': 'Sources summary copied to the clipboard.',
  'copy.failed': "Couldn't copy the summary.",
} as const;
