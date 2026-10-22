// What the params of a failure add to its message: the `reason` a collector gives (closed
// vocabularies, worded per error code), the sentences for a redirect and a wait, and the names
// of the settings fields. A reason with a `.title` replaces the title of the code as well.
export const errorDetailMessages = {
  'error.install_failed.reason.status_line_too_large.body':
    'آپ کی موجودہ اسٹیٹس لائن کمانڈ اتنی بڑی ہے کہ مانیٹر اسے جوڑ نہیں سکتا۔ کچھ تبدیل نہیں ہوا۔',
  'error.install_failed.reason.path_unsupported.body':
    'ڈیٹا فولڈر کے پاتھ میں ایسے حروف ہیں جو اسٹیٹس لائن کمانڈ قبول نہیں کرتی۔ کچھ تبدیل نہیں ہوا۔',
  'error.install_failed.reason.serialize_failed.body':
    'نئی settings.json نہیں بن سکی۔ کچھ تبدیل نہیں ہوا۔',
  'error.install_failed.reason.state_failed.body':
    'انسٹال کی حالت محفوظ نہیں ہو سکی۔ settings.json تبدیل نہیں ہوئی۔',
  'error.install_failed.reason.concurrent_modification.body':
    'عمل کے دوران settings.json بدل گئی، اس لیے مانیٹر نے اسے ہاتھ نہیں لگایا۔ دوبارہ کوشش کریں۔',
  'error.install_failed.reason.write_failed.body':
    'settings.json نہیں لکھی جا سکی۔ فائل تبدیل نہیں ہوئی؛ دیکھیں کہ لاک تو نہیں اور دوبارہ کوشش کریں۔',
  'error.install_failed.reason.shim_missing.body':
    'کمانڈ کی ایگزیکیوٹیبل مانیٹر کے ساتھ نہیں ملی۔ مانیٹر دوبارہ انسٹال کریں۔',
  'error.install_failed.reason.copy_failed.body':
    'کمانڈ کی ایگزیکیوٹیبل ڈیٹا فولڈر میں کاپی نہیں ہو سکی۔ settings.json تبدیل نہیں ہوئی۔',
  'error.install_failed.reason.backup_failed.body':
    'settings.json کا بیک اپ محفوظ نہیں ہو سکا، اس لیے اسے تبدیل نہیں کیا گیا۔',
  // Only a removal is refused for this reason.
  'error.install_failed.reason.foreign_status_line.title': 'کمانڈ ہٹائی نہیں جا سکی',
  'error.install_failed.reason.foreign_status_line.body':
    'ترتیب شدہ اسٹیٹس لائن مانیٹر کی نہیں ہے (شاید ہاتھ سے بدلی ہوئی کاپی ہے)، اس لیے کچھ نہیں ہٹایا گیا۔',

  // LiteLLM: the CA file the user chose, not the certificate of the server.
  'error.tls_error.reason.ca_missing.body':
    'ترتیب شدہ CA فائل نہیں ملی۔ سیٹنگز میں درست فائل منتخب کریں۔',
  'error.tls_error.reason.ca_too_large.body':
    'CA فائل پڑھنے کے لیے بہت بڑی ہے۔ سیٹنگز میں چھوٹی .pem فائل منتخب کریں۔',
  'error.tls_error.reason.ca_unreadable.body':
    'CA فائل نہیں پڑھی جا سکی۔ سیٹنگز میں پاتھ اور اجازتیں چیک کریں۔',
  'error.tls_error.reason.ca_invalid.body':
    'CA فائل میں درست سرٹیفکیٹ نہیں ہے۔ سیٹنگز میں درست .pem فائل منتخب کریں۔',

  // LiteLLM: which part of the answer did not look like a key, user or team record.
  'error.endpoint_changed.reason.not_an_object.body':
    'سرور کا جواب متوقع فارمیٹ میں نہیں ہے۔ چیک کریں کہ URL LiteLLM کی طرف ہے۔',
  'error.endpoint_changed.reason.no_record.body':
    'جواب میں key، یوزر یا ٹیم کا کوئی ریکارڈ نہیں۔ URL اور اسکوپ چیک کریں۔',
  'error.endpoint_changed.reason.id_missing.body':
    'جواب اس ID کی تصدیق نہیں کرتا جس کے بارے میں پوچھا گیا تھا۔ یوزر یا ٹیم کا ID چیک کریں۔',
  'error.endpoint_changed.reason.id_mismatch.body':
    'جواب دیے گئے ID سے مختلف ID کا ہے۔ یوزر یا ٹیم کا ID چیک کریں۔',

  // Antigravity: the IDE's local server is there, but there is nothing to call.
  'error.not_running.reason.no_port.title': 'مقامی سرور ابھی تیار نہیں',
  'error.not_running.reason.no_port.body':
    'Antigravity کھلا ہے مگر اس کا مقامی سرور ابھی تک ایڈریس نہیں دے رہا۔ یہ عموماً تھوڑی دیر میں ٹھیک ہو جاتا ہے: دوبارہ کوشش کریں۔',
  'error.unauthorized.reason.no_token.title': 'مقامی ٹوکن دستیاب نہیں',
  'error.unauthorized.reason.no_token.body':
    'مانیٹر IDE کے مقامی سرور کا سیشن ٹوکن نہیں پڑھ سکا۔ Antigravity دوبارہ شروع کریں اور پھر کوشش کریں۔',

  'error.timeout.reason.locked.body':
    'Cursor کا مقامی ڈیٹا بیس ابھی استعمال میں ہے۔ تھوڑی دیر بعد دوبارہ کوشش کریں۔',
  'error.timeout.reason.timeout.body':
    'Cursor کا مقامی ڈیٹا بیس پڑھنے میں بہت دیر لگی۔ تھوڑی دیر بعد دوبارہ کوشش کریں۔',
  'error.endpoint_changed.reason.schema.body':
    'Cursor کے مقامی ڈیٹا بیس کا فارمیٹ مانیٹر کی توقع سے مختلف ہے۔ ممکن ہے مانیٹر کو اپڈیٹ کی ضرورت ہو۔',
  'error.settings_unreadable.reason.permission.body':
    'Cursor کا مقامی ڈیٹا بیس کھولنے کی اجازت نہیں۔ فائل کی اجازتیں چیک کریں۔',
  'error.settings_unreadable.reason.not_a_database.body':
    'Cursor کی ملنے والی فائل ڈیٹا بیس نہیں ہے۔',
  'error.settings_unreadable.reason.other.body': 'Cursor کا مقامی ڈیٹا بیس نہیں پڑھا جا سکا۔',
  'error.not_signed_in.reason.not_found.body':
    'Copilot ایکسٹینشن کے فولڈر میں GitHub ٹوکن نہیں ملا۔ اپنے ایڈیٹر میں Copilot میں سائن اِن کریں، یا سیٹنگز میں ٹوکن پیسٹ کریں۔',
  'error.not_signed_in.reason.no_token.body':
    'Copilot ایکسٹینشن کی فائلوں میں github.com کا کوئی ٹوکن نہیں ہے۔ اپنے ایڈیٹر میں Copilot میں سائن اِن کریں، یا سیٹنگز میں ٹوکن پیسٹ کریں۔',
  'error.not_signed_in.reason.unreadable.body':
    'Copilot ایکسٹینشن کی سائن اِن فائل نہیں پڑھی جا سکی۔ اس کے فولڈر کی اجازتیں چیک کریں۔',
  'error.redirect':
    'سرور نے درخواست ری ڈائریکٹ کرنے کی کوشش کی، اور مانیٹر ری ڈائریکٹ کی پیروی نہیں کرتا۔',
  'error.retry_after': 'سورس کی طرف سے مانگا گیا انتظار: {wait}۔',
  'error.field_hint': 'مسئلے والا خانہ: {field}۔',
  'field.value': 'API key',
  'field.baseUrl': 'سرور URL',
  'field.entityId': 'یوزر یا ٹیم ID',
  'field.authHeader': 'تصدیقی ہیڈر',
  'field.pemPath': 'CA فائل (.pem)',
  'field.host': 'سرٹیفکیٹ کی تصدیق کے بغیر سرور',
  'field.configDir': 'Claude Code کنفیگریشن فولڈر',
  'field.binaryPath': 'Codex ایگزیکیوٹیبل کا پاتھ',
  'field.dataDir': 'OpenCode ڈیٹا فولڈر',
  'field.shimConsent': 'کمانڈ انسٹال کرنے کی رضامندی',
  'field.limit': 'پچھلی ونڈوز کی تعداد',
} as const satisfies Record<string, string>;
