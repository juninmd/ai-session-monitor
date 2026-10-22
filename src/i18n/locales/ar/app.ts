// ar catalog: app (src/app, App.tsx, and the unit words src/i18n/format.ts uses).
export const appMessages = {
  // Duration units: "1h 53m", "45 min", "4d 11h", "4 s". Short, no plural forms.
  'duration.d': 'ي',
  'duration.h': 'س',
  'duration.m': 'د',
  'duration.min': 'دقيقة',
  'duration.s': 'ث',
  // formatClock: after a verb ("resets at 14:35", "resets on 01/22 at 09:00").
  'clock.at': 'عند {time}',
  'clock.on_at': 'في {date} عند {time}',
  // formatAmountValue and formatFactValue; {value} is the formatted number.
  'amount.credits.zero': '{value} رصيد',
  'amount.credits.one': '{value} رصيد',
  'amount.credits.two': '{value} رصيدان',
  'amount.credits.few': '{value} أرصدة',
  'amount.credits.many': '{value} رصيدًا',
  'amount.credits.other': '{value} رصيد',
  'amount.requests.zero': '{value} طلب',
  'amount.requests.one': '{value} طلب',
  'amount.requests.two': '{value} طلبان',
  'amount.requests.few': '{value} طلبات',
  'amount.requests.many': '{value} طلبًا',
  'amount.requests.other': '{value} طلب',
  'amount.resets.zero': '{value} متاحة',
  'amount.resets.one': '{value} متاحة',
  'amount.resets.two': '{value} متاحتان',
  'amount.resets.few': '{value} متاحة',
  'amount.resets.many': '{value} متاحة',
  'amount.resets.other': '{value} متاحة',
  'amount.tokens': '{value} رمز',
  'amount.req_short': '{value} طلب',
  // Separator between items of a plain list ("78%, 80%").
  'list.sep': '، ',
  // Copying the sources summary to the clipboard.
  'copy.done': 'تم نسخ ملخص المصادر إلى الحافظة.',
  'copy.failed': 'تعذّر نسخ الملخص.',
} as const;
