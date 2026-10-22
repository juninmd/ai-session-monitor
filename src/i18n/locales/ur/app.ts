// Urdu catalog: app (src/app, App.tsx, and the unit words src/i18n/format.ts uses).
export const appMessages = {
  // Duration units: "1h 53m", "45 min", "4d 11h", "4 s". Short, no plural forms.
  'duration.d': 'د',
  'duration.h': 'گھ',
  'duration.m': 'م',
  'duration.min': 'منٹ',
  'duration.s': 'سیکنڈ',
  // formatClock: after a verb ("resets at 14:35", "resets on 01/22 at 09:00").
  'clock.at': '{time} پر',
  'clock.on_at': '{date} کو {time} پر',
  // formatAmountValue and formatFactValue; {value} is the formatted number.
  'amount.credits.one': '{value} کریڈٹ',
  'amount.credits.other': '{value} کریڈٹس',
  'amount.requests.one': '{value} درخواست',
  'amount.requests.other': '{value} درخواستیں',
  'amount.resets.one': '{value} دستیاب',
  'amount.resets.other': '{value} دستیاب',
  'amount.tokens': '{value} ٹوکنز',
  'amount.req_short': '{value} درخواست',
  // Separator between items of a plain list ("78%, 80%").
  'list.sep': '، ',
  // Copying the sources summary to the clipboard.
  'copy.done': 'سورسز کا خلاصہ کلپ بورڈ پر کاپی ہو گیا۔',
  'copy.failed': 'خلاصہ کاپی نہیں ہو سکا۔',
} as const;
