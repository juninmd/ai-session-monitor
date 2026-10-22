// One title and one body for every ErrorCode (src/lib/contract/view.ts), plus the OpenCode
// ambiguity wording. src/i18n/catalog.test.ts fails when a code is missing.
export const errorMessages = {
  'error.internal.title': 'کچھ غلط ہو گیا',
  'error.internal.body':
    'اس سورس کو پڑھتے وقت اندرونی خرابی پیش آئی۔ دوبارہ کوشش کریں؛ اگر بار بار ہو تو مانیٹر دوبارہ شروع کریں۔',
  'error.timeout.title': 'ریڈنگ میں بہت دیر لگی',
  'error.timeout.body': 'سورس نے وقت پر جواب نہیں دیا۔ تھوڑی دیر بعد دوبارہ کوشش کریں۔',
  'error.network.title': 'سورس تک رسائی نہیں ہو رہی',
  'error.network.body': 'سرور سے رابطہ نہیں ہو سکا۔ نیٹ ورک چیک کریں اور دوبارہ کوشش کریں۔',
  'error.bad_response.title': 'غیر متوقع جواب',
  'error.bad_response.body':
    'سورس نے ایسا جواب دیا جو مانیٹر کی سمجھ میں نہیں آیا۔ بعد میں دوبارہ کوشش کریں۔',
  'error.response_too_large.title': 'جواب بہت بڑا ہے',
  'error.response_too_large.body': 'جواب قبول شدہ حد سے بڑا تھا اور حفاظت کے لیے رد کر دیا گیا۔',
  'error.rate_limited.title': 'بہت زیادہ درخواستیں',
  'error.rate_limited.body':
    'سورس نے تھوڑا انتظار کرنے کو کہا ہے۔ مانیٹر لمبے وقفے کے ساتھ خود دوبارہ کوشش کرے گا۔',
  'error.unauthorized.title': 'key یا لاگ اِن مسترد',
  'error.unauthorized.body': 'سرور نے اسناد مسترد کر دیں۔ سیٹنگز میں key اور اس کا اسکوپ چیک کریں۔',
  'error.unauthorized_ambiguous.title': 'key یا Go سبسکرپشن کی تصدیق نہیں ہو سکی',
  'error.unauthorized_ambiguous.body':
    'سروس نے درخواست مسترد کر دی۔ ایسا درست key کے ساتھ بھی ہوتا ہے اور Go سبسکرپشن نہ ہونے پر بھی۔ OpenCode کنسول میں key اور سبسکرپشن چیک کریں۔',
  'error.unauthorized_ambiguous.kimi.title': 'Kimi Code key کی تصدیق نہیں ہو سکی',
  'error.unauthorized_ambiguous.kimi.body':
    'سروس نے درخواست مسترد کر دی۔ ایسا ختم شدہ key کے ساتھ بھی ہوتا ہے اور Moonshot اوپن پلیٹ فارم کے key کے ساتھ بھی، جو الگ پروڈکٹ ہے۔ Kimi Code کنسول میں دیکھیں کہ key اسی پلان کا ہے۔',
  'error.unauthorized_ambiguous.zai.title': 'z.ai key کی تصدیق نہیں ہو سکی',
  'error.unauthorized_ambiguous.zai.body':
    'سروس نے درخواست مسترد کر دی۔ ایسا درست key کے ساتھ بھی ہوتا ہے اور GLM Coding پلان کے بغیر اکاؤنٹس پر بھی۔ z.ai ڈیش بورڈ میں key اور پلان چیک کریں۔',
  'error.not_found.title': 'پتہ نہیں ملا',
  'error.not_found.body': 'سرور دیے گئے پاتھ یا ID کو نہیں پہچانتا۔ URL، اسکوپ اور ID چیک کریں۔',
  'error.server_error.title': 'سرور کی خرابی',
  'error.server_error.body': 'سورس کا سرور ناکام ہو گیا۔ بعد میں دوبارہ کوشش کریں۔',
  'error.tls_error.title': 'TLS سرٹیفکیٹ کی ناکامی',
  'error.tls_error.body': 'سرور کا سرٹیفکیٹ قبول نہیں ہوا۔ سیٹنگز میں درست CA منتخب کریں۔',
  'error.invalid_url.title': 'غلط URL',
  'error.invalid_url.body':
    'https:// استعمال کریں۔ HTTP صرف localhost کے لیے چلتا ہے، اور URL میں یوزر نیم، پاس ورڈ، کوئری پیرامیٹرز یا # نہیں ہو سکتے۔',
  'error.invalid_input.title': 'غلط قدر',
  'error.invalid_input.body': 'نشان زد خانہ درست کریں اور دوبارہ کوشش کریں۔',
  'error.settings_unreadable.title': 'سیٹنگز فائل نہیں پڑھی جا سکی',
  'error.settings_unreadable.body':
    'settings.json نہیں پڑھی جا سکی۔ کچھ تبدیل نہیں ہوا۔ فائل درست کریں اور دوبارہ کوشش کریں۔',
  'error.install_failed.title': 'انسٹال ناکام',
  'error.install_failed.body':
    'Claude Code کی اسٹیٹس لائن میں کمانڈ انسٹال نہیں ہو سکی۔ کچھ تبدیل نہیں ہوا۔',
  'error.unsupported_plan.title': 'پلان میں پڑھنے کے لیے کوئی حد نہیں',
  'error.unsupported_plan.body':
    'اکاؤنٹ ایسا پلان یا API key استعمال کرتا ہے جو حدود رپورٹ نہیں کرتا۔ آپ یہ سورس بند کر سکتے ہیں۔',
  'error.oauth_expired.title': 'Claude Code کی لاگ اِن ختم ہو گئی',
  'error.oauth_expired.body': 'لاگ اِن تجدید کرنے کے لیے Claude Code کھولیں اور دوبارہ کوشش کریں۔',
  'error.binary_not_found.title': 'پروگرام نہیں ملا',
  'error.binary_not_found.body':
    'مانیٹر کو ایگزیکیوٹیبل نہیں ملا۔ پروگرام انسٹال کریں یا سیٹنگز میں اس کا پاتھ دیں۔',
  'error.spawn_failed.title': 'پروگرام شروع نہیں ہو سکا',
  'error.spawn_failed.body': 'پروسیس شروع نہیں ہوا۔ ایگزیکیوٹیبل کا پاتھ چیک کریں۔',
  'error.handshake_failed.title': 'پروگرام سے رابطہ نہیں ہو سکا',
  'error.handshake_failed.body':
    'پروگرام شروع ہو گیا مگر توقع کے مطابق جواب نہیں دیا۔ اسے اپڈیٹ کریں اور دوبارہ کوشش کریں۔',
  'error.protocol_error.title': 'غیر متوقع فارمیٹ',
  'error.protocol_error.body': 'ڈیٹا کا فارمیٹ بدل گیا ہے۔ مانیٹر یا پروگرام کو اپڈیٹ کریں۔',
  'error.not_signed_in.title': 'اکاؤنٹ جڑا ہوا نہیں',
  'error.not_signed_in.body': 'پروگرام میں اپنے اکاؤنٹ میں سائن اِن کریں اور دوبارہ کوشش کریں۔',
  'error.not_running.title': 'پروگرام بند ہے',
  'error.not_running.body':
    'اس ریڈنگ کے لیے پروگرام کا کھلا ہونا ضروری ہے۔ اسے کھولیں اور دوبارہ کوشش کریں۔',
  'error.endpoint_changed.title': 'سورس کا فارمیٹ بدل گیا',
  'error.endpoint_changed.body':
    'سروس بدل گئی ہے اور یہ ریڈنگ پہلے کی طرح کام نہیں کرتی۔ ممکن ہے مانیٹر کو اپڈیٹ کی ضرورت ہو۔',
  'error.http': 'HTTP کوڈ: {httpStatus}۔',
} as const satisfies Record<string, string>;
