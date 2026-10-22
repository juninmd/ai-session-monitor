// Notice codes from docs/ipc.md. An unknown code renders `notice.unknown` with the code in it,
// so a missing translation is visible instead of silent.
export const noticeMessages = {
  'notice.code.claude_waiting_first_session':
    'کمانڈ انسٹال ہو گئی۔ Claude Code سیشن کے پہلے جواب کے بعد حدود آنا شروع ہوں گی۔',
  'notice.code.claude_no_rate_limits':
    'Claude Code حدود رپورٹ نہیں کرتا (API key بلنگ یا ایسا پلان جس میں حدود نہیں)۔',
  'notice.code.claude_shim_outdated':
    'Claude Code کی اسٹیٹس لائن میں انسٹال کی گئی کمانڈ نہیں ملی۔ ریڈنگ دوبارہ شروع کرنے کے لیے اسے دوبارہ انسٹال کریں۔',
  'notice.code.claude_oauth_partial':
    'ہر ماڈل کا استعمال (claude /usage یا اکاؤنٹ استعمال API) آن ہے مگر ناکام رہا ({reason})۔ صرف اسٹیٹس لائن کا ڈیٹا دکھایا جا رہا ہے۔',
  'notice.code.codex_rollout_fallback':
    'codex app-server نے جواب نہیں دیا ({reason})۔ آخری ریکارڈ شدہ سیشن پڑھا جا رہا ہے؛ اعداد پرانے ہو سکتے ہیں۔',
  'notice.code.codex_no_limits': 'Codex اکاؤنٹ حدود رپورٹ نہیں کرتا (API key سے لاگ اِن)۔',
  'notice.code.antigravity_pool_unknown':
    '{pool} کوٹا کا کوئی ڈیٹا نہیں ملا۔ یہ صفر نہیں بلکہ نامعلوم کے طور پر دکھایا جاتا ہے۔',
  'notice.code.litellm_no_budget': 'اس key، یوزر یا ٹیم کے بجٹ کی کوئی حد نہیں ہے۔',
  'notice.code.litellm_level_unreadable':
    '{level} کا بجٹ نہیں پڑھا جا سکا ({reason})۔ ہو سکتا ہے وہاں کوئی حد ہو جو یہاں نظر نہیں آتی۔',
  'notice.code.litellm_windows_truncated':
    'بجٹ ونڈوز اتنی ہیں کہ یہاں سما نہیں سکتیں۔ سب سے کم استعمال والی چھوڑ دی گئیں: {dropped}۔',
  'notice.code.opencode_window_idle':
    '{window} ونڈو 0% پر ہے: یہ اگلی درخواست سے ہی گنتی شروع کرے گی۔',
  'notice.code.opencode_rate_limited': '{window} ونڈو پر شرح کی حد لگ گئی ہے۔',
  'notice.code.opencode_no_key':
    'صرف مقامی اندازہ دکھایا جا رہا ہے۔ اصل ونڈوز دیکھنے کے لیے Go API key پیسٹ کریں۔',
  'notice.code.opencode_local_estimate':
    'مقامی اعداد لسٹ پرائس پر مبنی اندازہ ہیں اور صرف اسی PC کے ہیں۔ یہ کوٹا نہیں ہیں۔',
  'notice.code.opencode_db_unreadable': 'مقامی OpenCode ڈیٹا بیس نہیں پڑھا جا سکا: {reason}۔',
  'notice.unknown': 'غیر ترجمہ شدہ نوٹس: {code}',
  'notice.action.reinstall': 'دوبارہ انسٹال کریں',
  'notice.action.settings': 'key درج کریں',
  'notice.param.window.rolling': '5 گھنٹے کی',
  'notice.param.window.weekly': 'ہفتہ وار',
  'notice.param.window.monthly': 'ماہانہ',
  'notice.param.level.key': 'key',
  'notice.param.level.user': 'یوزر',
  'notice.param.level.team': 'ٹیم',
  'notice.param.dropped.one': '{n} ونڈو',
  'notice.param.dropped.other': '{n} ونڈوز',
  // The OpenCode database reasons are their own vocabulary: `not_found` there is a missing file.
  'notice.reason.opencode_db_unreadable.invalid_path': 'ڈیٹا فولڈر کا راستہ درست نہیں',
  'notice.reason.opencode_db_unreadable.not_found': 'ڈیٹا بیس فائل نہیں ملی',
  'notice.reason.opencode_db_unreadable.permission': 'ڈیٹا بیس فائل کھولنے کی اجازت نہیں',
  'notice.reason.opencode_db_unreadable.locked': 'ڈیٹا بیس کوئی اور پروگرام استعمال کر رہا ہے',
  'notice.reason.opencode_db_unreadable.not_a_database': 'ملنے والی فائل ڈیٹا بیس نہیں ہے',
  'notice.reason.opencode_db_unreadable.schema': 'ڈیٹا بیس کا فارمیٹ بدل گیا ہے',
  'notice.reason.opencode_db_unreadable.timeout': 'ڈیٹا بیس پڑھنے میں بہت دیر لگی',
  'notice.reason.opencode_db_unreadable.other': 'نامعلوم وجہ',
  'notice.severity.info': 'معلومات',
  'notice.severity.warn': 'انتباہ',
  'notice.severity.error': 'خرابی',
} as const satisfies Record<string, string>;
