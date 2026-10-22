// First-run connection surfaces: consent cards, secret fields, OpenCode and LiteLLM forms.
export const setupMessages = {
  'consent.skip': 'ابھی نہیں',
  'consent.details': 'کیا پڑھا جائے گا',

  'claude.connect.title': 'Claude Code سے جوڑیں',
  'claude.connect.lead':
    'مانیٹر Claude Code کی اسٹیٹس لائن میں ایک چھوٹی کمانڈ شامل کرتا ہے۔ یہ صرف حدود کے فیصد (5 گھنٹے اور ہفتہ) ایک مقامی فائل میں رکھتی ہے۔ کوئی لاگ اِن نہیں پڑھی جاتی۔',
  'claude.connect.detail':
    'Claude Code کی settings.json بیک اپ کے ساتھ تبدیل کرتا ہے اور آپ کی موجودہ اسٹیٹس لائن کو ساتھ جوڑ دیتا ہے۔ صرف حدود %LOCALAPPDATA% میں لکھتا ہے۔ نہ نیٹ ورک، نہ ٹوکنز۔ Pro اور Max اکاؤنٹس پر کام کرتا ہے۔ اعداد سیشن کے پہلے جواب کے بعد آتے ہیں اور صرف Claude Code کھلا ہونے پر اپڈیٹ ہوتے ہیں۔',
  'claude.connect.allow': 'کمانڈ انسٹال کریں',
  'claude.connect.tip':
    'مشورہ: Windows نئے آئیکنز کو اوور فلو تیر والے مینو میں چھپا دیتا ہے۔ مانیٹر کا آئیکن ٹاسک بار پر گھسیٹ دیں تاکہ ہمیشہ نظر آئے۔',

  'codex.connect.title': 'Codex سے جوڑیں',
  'codex.connect.lead':
    'مانیٹر یہاں انسٹال شدہ codex app-server چلاتا ہے اور پوچھتا ہے کہ 5 گھنٹے کی ونڈو اور ہفتے میں کتنا باقی ہے۔',
  'codex.connect.detail':
    'آپ کی مشین پر codex پروگرام صرف پڑھنے کے موڈ میں چلتا ہے۔ کوئی ڈیٹا اس کمپیوٹر سے باہر نہیں جاتا اور ایپ Codex کی لاگ اِن فائل کو ہاتھ نہیں لگاتی۔ OpenAI نے app-server کو تجرباتی قرار دیا ہے، اس لیے ورژنز کے درمیان فارمیٹ بدل سکتا ہے۔',
  'codex.connect.allow': 'پڑھنے کی اجازت دیں',

  'antigravity.connect.title': 'Antigravity سے جوڑیں',
  'antigravity.connect.lead':
    'کوٹا انسٹال شدہ agy (CLI) یا IDE کے مقامی سرور سے آتا ہے۔ یہ راستہ غیر سرکاری ہے اور Antigravity کی اپڈیٹ پر کام کرنا بند کر سکتا ہے۔',
  'antigravity.connect.detail':
    'آپ کی اجازت سے مانیٹر انسٹال شدہ agy صرف /usage پڑھنے کے لیے چلا سکتا ہے (ٹوکنز خرچ کیے بغیر اور لاگ اِن ڈیٹا پڑھے بغیر) اور/یا مقامی سیشن ٹوکن حاصل کرنے کے لیے IDE سرور کی کمانڈ لائن پڑھ سکتا ہے، اور صرف 127.0.0.1 سے بات کرتا ہے۔ ڈسک پر کچھ نہیں لکھا جاتا اور انٹرنیٹ پر کچھ نہیں بھیجا جاتا۔',
  'antigravity.connect.allow': 'مقامی ریڈنگ کی اجازت دیں',

  'shim.title': 'اسٹیٹس لائن کمانڈ',
  'shim.state.not_installed': 'انسٹال نہیں',
  'shim.state.installed': 'انسٹال ہے',
  'shim.state.installed_chained': 'انسٹال ہے، آپ کی اسٹیٹس لائن کے ساتھ جڑی ہوئی',
  'shim.state.foreign': 'کوئی اور اسٹیٹس لائن استعمال میں ہے',
  'shim.state.outdated': 'پرانی: ایگزیکیوٹیبل نہیں ملی',
  'shim.state.unreadable': 'settings.json پڑھی نہیں جا سکتی',
  'shim.hint.foreign':
    'انسٹال کرنے پر آپ کی موجودہ اسٹیٹس لائن چلتی رہے گی: مانیٹر اسے ساتھ جوڑ لیتا ہے۔',
  'shim.hint.unreadable':
    'جو فائل پڑھی نہ جا سکے مانیٹر اسے ہاتھ نہیں لگاتا۔ settings.json درست کریں اور دوبارہ چیک کریں۔',
  'shim.path': 'فائل: {path}',
  'shim.install': 'کمانڈ انسٹال کریں',
  'shim.reinstall': 'کمانڈ دوبارہ انسٹال کریں',
  'shim.uninstall': 'کمانڈ ہٹائیں',
  'shim.recheck': 'دوبارہ چیک کریں',
  'shim.loading': 'Claude Code چیک ہو رہا ہے',

  'secret.saved': 'key محفوظ ہے',
  'secret.not_saved': 'کوئی key محفوظ نہیں',
  'secret.placeholder': 'key پیسٹ کریں',
  'secret.placeholder_keep': 'محفوظ key رکھنے کے لیے خالی چھوڑیں',
  'secret.hint': 'Windows Credential Manager میں رہتی ہے اور انٹرفیس میں کبھی ظاہر نہیں ہوتی۔',
  'secret.save': 'key محفوظ کریں',
  'secret.clear': 'key ہٹائیں',
  'secret.empty': 'API key پیسٹ کریں۔',
  'secret.saved_ok': 'key محفوظ ہو گئی۔',
  'secret.cleared_ok': 'key ہٹا دی گئی۔',

  'opencode.title': 'OpenCode Go سے جوڑیں',
  'opencode.lead':
    'اصل ونڈوز (5 گھنٹے، ہفتہ اور مہینہ) پڑھنے کے لیے اپنی OpenCode Go API key پیسٹ کریں۔ key کے بغیر صرف استعمال کا مقامی اندازہ نظر آتا ہے۔',
  'opencode.key_label': 'OpenCode Go API key',
  'opencode.console': 'OpenCode Go کنسول کھولیں',
  'opencode.billing': 'Zen بلنگ دیکھیں',
  'opencode.zen_note': 'Zen (پری پیڈ) بیلنس کی کوئی API نہیں: بلنگ ڈیش بورڈ دیکھیں۔',
  'opencode.local_label': 'مقامی اندازہ (صرف یہی PC)',
  'opencode.local_hint':
    'OpenCode ڈیٹا بیس صرف پڑھنے کے موڈ میں کھولتا ہے اور لسٹ پرائس پر صرف لاگت اور ٹوکنز کا مجموعہ نکالتا ہے۔ یہ اصل کوٹا نہیں ہے۔',
  'opencode.data_dir': 'OpenCode ڈیٹا فولڈر',
  'opencode.data_dir_hint': 'ڈیفالٹ فولڈر استعمال کرنے کے لیے خالی چھوڑیں۔',
  'opencode.save_settings': 'فولڈر محفوظ کریں',

  'kimi.title': 'Kimi Code سے جوڑیں',
  'kimi.lead':
    'پلان کا کوٹا پڑھنے کے لیے اپنی Kimi Code API key پیسٹ کریں: 5 گھنٹے کی ونڈو، ہفتہ اور ماہانہ کل، جب پلان انہیں رپورٹ کرے۔',
  'kimi.detail':
    'غیر سرکاری ریڈنگ: استعمال کی API غیر دستاویزی ہے اور بغیر اطلاع کے بدل سکتی ہے۔ key صرف api.kimi.com کو بھیجی جاتی ہے اور Windows Credential Manager میں رہتی ہے۔ کسی دوسرے ٹول کی کوکی یا لاگ اِن استعمال نہیں ہوتی۔ Moonshot کے اوپن پلیٹ فارم کی key الگ پروڈکٹ کی ہے اور یہاں کام نہیں کرتی۔',
  'kimi.key_label': 'Kimi Code API key',
  'kimi.console': 'Kimi Code کنسول کھولیں',
  'kimi.note':
    'Kimi Code کی استعمال API کی غیر سرکاری ریڈنگ۔ ہفتہ اور مہینہ صرف تب نظر آتے ہیں جب پلان انہیں رپورٹ کرے۔',

  'zai.title': 'z.ai سے جوڑیں',
  'zai.lead':
    '5 گھنٹے کی ونڈو، ہفتہ اور ماہانہ MCP ٹولز کا کوٹا پڑھنے کے لیے اپنے GLM Coding پلان کی API key پیسٹ کریں۔',
  'zai.detail':
    'غیر سرکاری ریڈنگ: کوٹا API غیر دستاویزی ہے اور بغیر اطلاع کے بدل سکتی ہے۔ key صرف api.z.ai کو بھیجی جاتی ہے اور Windows Credential Manager میں رہتی ہے۔ کسی دوسرے ٹول کی کوکی یا لاگ اِن استعمال نہیں ہوتی۔ BigModel اکاؤنٹس (open.bigmodel.cn) نہیں پڑھے جاتے۔',
  'zai.key_label': 'z.ai API key',
  'zai.console': 'z.ai کیز کھولیں',
  'zai.note': 'گلوبل سرور (api.z.ai) پر GLM Coding پلان کی کوٹا API کی غیر سرکاری ریڈنگ۔',

  'openrouter.title': 'OpenRouter سے جوڑیں',
  'openrouter.lead':
    'key کے خرچ کی حد، استعمال اور، اگر key اجازت دے، کریڈٹ بیلنس دیکھنے کے لیے اپنی OpenRouter API key پیسٹ کریں۔',
  'openrouter.detail':
    'سرکاری ریڈنگ، OpenRouter کی دستاویزی API کے ذریعے۔ key صرف openrouter.ai کو بھیجی جاتی ہے اور Windows Credential Manager میں رہتی ہے۔ کریڈٹ بیلنس کے لیے management key ضروری ہے؛ عام key کے ساتھ صرف key کی حد اور استعمال نظر آتے ہیں۔',
  'openrouter.key_label': 'OpenRouter API key',
  'openrouter.console': 'OpenRouter کیز کھولیں',
  'openrouter.note':
    'OpenRouter API کی سرکاری ریڈنگ۔ key پر حد نہ ہو تو فیصد نہیں، صرف استعمال نظر آتا ہے۔ بیلنس صرف management key کے ساتھ نظر آتا ہے۔',

  'deepseek.title': 'DeepSeek سے جوڑیں',
  'deepseek.lead':
    'ہر اس کرنسی میں اکاؤنٹ بیلنس دیکھنے کے لیے اپنی DeepSeek API key پیسٹ کریں جس میں کریڈٹ ہو۔',
  'deepseek.detail':
    'سرکاری ریڈنگ، DeepSeek کی دستاویزی API کے ذریعے۔ key صرف api.deepseek.com کو بھیجی جاتی ہے اور Windows Credential Manager میں رہتی ہے۔ بیلنس کی کوئی حد یا ری سیٹ وقت نہیں، اس لیے فیصد نہیں ہے؛ یوآن اور ڈالر کبھی جمع نہیں کیے جاتے۔',
  'deepseek.key_label': 'DeepSeek API key',
  'deepseek.console': 'DeepSeek کیز کھولیں',
  'deepseek.note':
    'DeepSeek بیلنس API کی سرکاری ریڈنگ۔ یہ رقم ہے، کوٹا نہیں: نہ فیصد ہے نہ ری سیٹ۔',

  'deepinfra.title': 'DeepInfra سے جوڑیں',
  'deepinfra.lead':
    'بیلنس، خرچ کی حد اور اس مہینے کا خرچ دیکھنے کے لیے اپنی DeepInfra API key پیسٹ کریں۔',
  'deepinfra.detail':
    'سرکاری ریڈنگ، DeepInfra کی بلنگ API کے ذریعے۔ key صرف api.deepinfra.com کو بھیجی جاتی ہے اور Windows Credential Manager میں رہتی ہے۔ خرچ کی حد نہ ہو تو فیصد نہیں، صرف ڈالر کی رقوم نظر آتی ہیں۔',
  'deepinfra.key_label': 'DeepInfra API key',
  'deepinfra.console': 'DeepInfra کیز کھولیں',
  'deepinfra.note':
    'DeepInfra بلنگ API کی سرکاری ریڈنگ۔ قدریں ڈالر میں پڑھی جاتی ہیں؛ خرچ کی حد نہ ہو تو فیصد نہیں ہوتا۔',

  'moonshot.title': 'Moonshot سے جوڑیں',
  'moonshot.lead':
    'اکاؤنٹ بیلنس دیکھنے کے لیے اپنے Moonshot پلیٹ فارم (platform.moonshot.ai) کی API key پیسٹ کریں۔',
  'moonshot.detail':
    'سرکاری ریڈنگ، Moonshot کی دستاویزی بیلنس API کے ذریعے۔ key صرف api.moonshot.ai کو بھیجی جاتی ہے اور Windows Credential Manager میں رہتی ہے۔ kimi.com پلیٹ فارم کی keys الگ ہیں اور یہاں کام نہیں کرتیں۔ بیلنس رقم ہے: نہ فیصد ہے نہ ری سیٹ۔',
  'moonshot.key_label': 'Moonshot API key',
  'moonshot.console': 'Moonshot کیز کھولیں',
  'moonshot.note':
    'Moonshot بیلنس API کی سرکاری ریڈنگ، ڈالر میں۔ رقم واجب الادا ہو تو نقد بیلنس منفی ہو سکتا ہے۔',

  'novita.title': 'Novita سے جوڑیں',
  'novita.lead':
    'اکاؤنٹ بیلنس، کریڈٹ کی حد اور زیر التوا چارجز دیکھنے کے لیے اپنی Novita API key پیسٹ کریں۔',
  'novita.detail':
    'سرکاری ریڈنگ، Novita کی دستاویزی بلنگ API کے ذریعے۔ key صرف api.novita.ai کو بھیجی جاتی ہے اور Windows Credential Manager میں رہتی ہے۔ بیلنس رقم ہے: نہ فیصد ہے نہ ری سیٹ۔',
  'novita.key_label': 'Novita API key',
  'novita.console': 'Novita کیز کھولیں',
  'novita.note':
    'Novita بیلنس API کی سرکاری ریڈنگ، ڈالر میں۔ رقم واجب الادا ہو تو بیلنس منفی ہو سکتا ہے۔',

  'ollama.title': 'Ollama Cloud سے جوڑیں',
  'ollama.lead':
    'پلان کے شامل کریڈٹس اور خریدا گیا بیلنس دیکھنے کے لیے Ollama Cloud API key پیسٹ کریں۔',
  'ollama.detail':
    'سرکاری ریڈنگ، Ollama Cloud کی دستاویزی بیلنس API کے ذریعے۔ key ollama.com/settings/keys پر بنائیں: CLI کی سبسکرپشن key کبھی نہیں پڑھی جاتی۔ key صرف ollama.com کو بھیجی جاتی ہے اور Windows Credential Manager میں رہتی ہے۔ جواب کا فارمیٹ پلان پر منحصر ہے (ماہانہ کریڈٹس یا سیشن اور ہفتے کی ونڈوز)۔',
  'ollama.key_label': 'Ollama Cloud API key',
  'ollama.console': 'Ollama Cloud کیز کھولیں',
  'ollama.note':
    'Ollama Cloud بیلنس API کی سرکاری ریڈنگ۔ موجودہ پلانز ڈالر کریڈٹس دکھاتے ہیں؛ پرانے پلانز سیشن اور ہفتے کے فیصد دکھاتے ہیں۔',

  'minimax.title': 'MiniMax سے جوڑیں',
  'minimax.lead':
    'ہر ماڈل کا کوٹا (موجودہ وقفہ اور ہفتہ) دیکھنے کے لیے اپنی MiniMax Token Plan key پیسٹ کریں۔',
  'minimax.detail':
    'غیر سرکاری ریڈنگ: Token Plan کوٹا روٹ کی کوئی شائع شدہ اسکیما نہیں اور یہ بغیر اطلاع کے بدل سکتا ہے۔ key صرف api.minimax.io (گلوبل سرور) کو بھیجی جاتی ہے اور Windows Credential Manager میں رہتی ہے۔ پہلی بار اعداد کا MiniMax کنسول سے موازنہ کر لیں۔',
  'minimax.key_label': 'MiniMax Token Plan key',
  'minimax.console': 'MiniMax کیز کھولیں',
  'minimax.note':
    'MiniMax Token Plan کوٹا کی ہر ماڈل کے حساب سے غیر سرکاری ریڈنگ۔ جس ماڈل کا پلان میں کوٹا نہ ہو وہ نظر نہیں آتا۔',

  'litellm.title': 'LiteLLM سے جوڑیں',
  'litellm.title_edit': 'LiteLLM کنکشن',
  'litellm.url': 'سرور URL',
  'litellm.url_placeholder': 'https://litellm.yourcompany.com',
  'litellm.url_hint': 'HTTPS ضروری ہے۔ HTTP صرف localhost کے لیے قبول ہے۔',
  'litellm.url_required': 'سرور URL درج کریں۔',
  'litellm.url_invalid': 'مکمل URL درج کریں، جیسے https://litellm.yourcompany.com۔',
  'litellm.url_http': 'https:// استعمال کریں۔ HTTP صرف localhost کے لیے چلتا ہے۔',
  'litellm.url_parts': 'URL سے یوزر نیم، پاس ورڈ، کوئری پیرامیٹرز اور # ہٹا دیں۔',
  'litellm.key': 'API key',
  'litellm.key_required': 'LiteLLM API key پیسٹ کریں۔',
  'litellm.key_rebind':
    'محفوظ key صرف موجودہ کنکشن پر لاگو ہوتی ہے۔ چونکہ کنکشن بدل گیا ہے، key دوبارہ پیسٹ کریں۔',
  'litellm.scope': 'کیا ناپنا ہے',
  'litellm.scope.key': 'یہ key',
  'litellm.scope.user': 'یوزر',
  'litellm.scope.team': 'ٹیم',
  'litellm.entity.user': 'یوزر ID',
  'litellm.entity.team': 'ٹیم ID',
  'litellm.entity_required': 'ID درج کریں۔',
  'litellm.advanced': 'اعلیٰ',
  'litellm.auth_header': 'تصدیقی ہیڈر',
  'litellm.auth_header_hint': 'صرف تب بدلیں جب آپ کی پراکسی key کے لیے اپنا ہیڈر استعمال کرے۔',
  'litellm.auth_header_invalid': 'صرف حروف، اعداد اور ہائفن استعمال کریں۔',
  'litellm.poll': 'ہر اتنی دیر بعد ریفریش',
  'litellm.timeout': 'ٹائم آؤٹ',
  'litellm.tls': 'TLS سرٹیفکیٹ',
  'litellm.tls.system': 'Windows سے',
  'litellm.tls.custom_ca': 'اپنا CA',
  'litellm.tls.insecure': 'تصدیق نہ کریں',
  'litellm.pem': 'CA فائل (.pem)',
  'litellm.pem_placeholder': 'C:\\certs\\ca.pem',
  'litellm.pem_required': '.pem فائل کا پاتھ درج کریں۔',
  'litellm.insecure_host': 'تصدیق کے بغیر سرور',
  'litellm.insecure_host_hint': 'سرٹیفکیٹ صرف اسی سرور کے لیے نظر انداز ہوتا ہے۔',
  'litellm.insecure_host_required': 'سرور درج کریں۔',
  'litellm.insecure_warning':
    'کنکشن کی تصدیق نہیں ہوگی اور key کو راستے میں پکڑا جا سکتا ہے۔ صرف بھروسے مند نیٹ ورک پر استعمال کریں۔',
  'litellm.insecure_confirm':
    'میں خطرہ سمجھتا ہوں اور اس سرور کا سرٹیفکیٹ نظر انداز کرنا چاہتا ہوں۔',
  'litellm.insecure_confirm_required': 'سرٹیفکیٹ کی تصدیق کے بغیر آگے بڑھنے کے لیے تصدیق کریں۔',
  'litellm.save': 'محفوظ کریں اور جوڑیں',
  'litellm.saving': 'محفوظ ہو رہا ہے…',
  'litellm.saved': 'سیٹنگز محفوظ ہو گئیں۔ سرور پڑھا جا رہا ہے…',
  'litellm.edit': 'کنکشن میں ترمیم کریں',
  'litellm.delay_note':
    'سرور پر خرچ اور ری سیٹ میں تقریباً 10 منٹ تک کی تاخیر ہو سکتی ہے۔ حد صرف key پر نہیں بلکہ یوزر یا ٹیم پر بھی ہو سکتی ہے۔',
} as const satisfies Record<string, string>;
