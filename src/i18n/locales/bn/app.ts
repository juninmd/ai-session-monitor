// bn catalog: app (src/app, App.tsx, and the unit words src/i18n/format.ts uses).
export const appMessages = {
  // Duration units: "1h 53m", "45 min", "4d 11h", "4 s". Short, no plural forms.
  'duration.d': 'দিন',
  'duration.h': 'ঘ',
  'duration.m': 'মি',
  'duration.min': 'মিনিট',
  'duration.s': 'সে',
  // formatClock: after a verb ("resets at 14:35", "resets on 01/22 at 09:00").
  'clock.at': '{time}-এ',
  'clock.on_at': '{date} তারিখে {time}-এ',
  // formatAmountValue and formatFactValue; {value} is the formatted number.
  'amount.credits.one': '{value} ক্রেডিট',
  'amount.credits.other': '{value} ক্রেডিট',
  'amount.requests.one': '{value}টি রিকোয়েস্ট',
  'amount.requests.other': '{value}টি রিকোয়েস্ট',
  'amount.resets.one': '{value}টি উপলব্ধ',
  'amount.resets.other': '{value}টি উপলব্ধ',
  'amount.tokens': '{value} টোকেন',
  'amount.req_short': '{value} রিকো.',
  // Separator between items of a plain list ("78%, 80%").
  'list.sep': ', ',
  // Copying the sources summary to the clipboard.
  'copy.done': 'সোর্সের সারাংশ ক্লিপবোর্ডে কপি হয়েছে।',
  'copy.failed': 'সারাংশ কপি করা যায়নি।',
} as const;
