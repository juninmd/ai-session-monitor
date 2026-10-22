// zh-CN catalog: app (src/app, App.tsx, and the unit words src/i18n/format.ts uses).
export const appMessages = {
  // Duration units: "1h 53m", "45 min", "4d 11h", "4 s". Short, no plural forms.
  'duration.d': '天',
  'duration.h': '小时',
  'duration.m': '分',
  'duration.min': '分钟',
  'duration.s': '秒',
  // formatClock: after a verb ("resets at 14:35", "resets on 01/22 at 09:00").
  'clock.at': '{time}',
  'clock.on_at': '{date} {time}',
  // formatAmountValue and formatFactValue; {value} is the formatted number.
  'amount.credits.other': '{value} 积分',
  'amount.requests.other': '{value} 次请求',
  'amount.resets.other': '{value} 次可用',
  'amount.tokens': '{value} Token',
  'amount.req_short': '{value} 次请求',
  // Separator between items of a plain list ("78%, 80%").
  'list.sep': '、',
  // Copying the sources summary to the clipboard.
  'copy.done': '数据源摘要已复制到剪贴板。',
  'copy.failed': '无法复制摘要。',
} as const;
