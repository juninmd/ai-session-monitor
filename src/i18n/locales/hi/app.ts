// hi catalog: app (src/app, App.tsx, and the unit words src/i18n/format.ts uses).
export const appMessages = {
  // Duration units: "1h 53m", "45 min", "4d 11h", "4 s". Short, no plural forms.
  'duration.d': 'दिन',
  'duration.h': 'घं',
  'duration.m': 'मि',
  'duration.min': 'मिनट',
  'duration.s': 'सेक',
  // formatClock: after a verb ("resets at 14:35", "resets on 01/22 at 09:00").
  'clock.at': '{time} बजे',
  'clock.on_at': '{date} को {time} बजे',
  // formatAmountValue and formatFactValue; {value} is the formatted number.
  'amount.credits.one': '{value} क्रेडिट',
  'amount.credits.other': '{value} क्रेडिट',
  'amount.requests.one': '{value} अनुरोध',
  'amount.requests.other': '{value} अनुरोध',
  'amount.resets.one': '{value} उपलब्ध',
  'amount.resets.other': '{value} उपलब्ध',
  'amount.tokens': '{value} टोकन',
  'amount.req_short': '{value} अनु.',
  // Separator between items of a plain list ("78%, 80%").
  'list.sep': ', ',
  // Copying the sources summary to the clipboard.
  'copy.done': 'स्रोतों का सारांश क्लिपबोर्ड पर कॉपी हो गया।',
  'copy.failed': 'सारांश कॉपी नहीं हो सका।',
} as const;
