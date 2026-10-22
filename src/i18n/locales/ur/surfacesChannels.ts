// Urdu catalog: the "Quota channels" surface (what each tool could still tell us).
export const channelsMessages = {
  'channels.page.title': 'ہم اب بھی کیا پڑھ سکتے ہیں',
  'channels.page.intro':
    '16 اکتوبر 2026 کا جائزہ، ہر ٹول کی دستاویزات اور کوڈ کے مطابق۔ ہر چینل اس endpoint کا نام بتاتا ہے جو اسے فراہم کرتا ہے اور یہ کہ وہ ایپ کے ماڈل میں کیسے بیٹھتا ہے: فیصد، رقم اور ری سیٹ والی {window}، یا بغیر حد کے {fact}۔',
  'channels.page.counts':
    '{now} نئے چینلز اس credential کی پہنچ میں جو ایپ پہلے ہی قبول کرتی ہے · {already} پہلے سے پڑھے جانے کی تصدیق شدہ · {forbidden} پروجیکٹ کے اصولوں سے باہر۔',
  'channels.candidates.title': 'امیدوار سورسز',
  'channels.candidates.note':
    'ایسے ٹولز جن کا سرکاری استعمال چینل ہے اور ایسا credential جو موجودہ ماڈل میں فٹ ہو۔',
  'channels.candidates.credential': 'credential: {credential}',
  'channels.reach.already': 'پہلے سے پڑھا جا رہا ہے',
  'channels.reach.now': 'ابھی پڑھا جا سکتا ہے',
  'channels.reach.forbidden': 'موجود ہے، مگر اصولوں کے خلاف',
  'channels.reach_note.already': 'آج پروڈکشن میں ہے۔',
  'channels.reach_note.now':
    'وہی credential اور وہی اصول: یہ SafeClient سے گزرے گا اور ونڈو یا fact بن جائے گا۔ ہر غیر سرکاری چینل کو پھر بھی docs/data-sources.md میں اندراج اور انٹرفیس میں "غیر سرکاری" بیج چاہیے۔',
  'channels.reach_note.forbidden':
    'OAuth ٹوکن، کوکی، براؤزر سیشن یا ماسٹر key چاہیے۔ ایپ ان میں سے کچھ نہیں کرتی۔',
  'channels.level.high': 'اعلیٰ',
  'channels.level.medium': 'درمیانی',
  'channels.level.low': 'کم',
  'channels.confidence': '{level} اعتماد',
  'channels.form.window': 'ونڈو',
  'channels.form.fact': 'fact',
  'channels.meta': '{level} اعتماد · سورس: {source}',
  'channels.source.zai_counts': 'پہلے سے پڑھے جانے والے endpoint کا جواب',
  'channels.source.community': 'کمیونٹی (ccusage)',
  'channels.claude_extra_usage.title': 'استعمال کریڈٹس (ماہانہ بجٹ)',
  'channels.claude_extra_usage.how':
    'GET api.anthropic.com/api/oauth/usage → {fields}، سینٹس میں۔ CLI نہ ہو یا ناکام ہو تو endpoint پہلے ہی متبادل کے طور پر کال ہوتا ہے؛ key KNOWN_KEYS میں ہے اور آج رد کر دی جاتی ہے۔',
  'channels.claude_extra_usage.caveat':
    'صرف اختیاری (غیر سرکاری) OAuth راستے پر موجود ہے؛ opt-in اور بیج چاہیے۔',
  'channels.openrouter_budgets.title': 'اسکوپ کے حساب سے بجٹ، ری سیٹ کے ساتھ',
  'channels.openrouter_budgets.how':
    'GET /api/v1/key → {fields} اور data.effective_budget۔ آج ہم صرف limit/limit_remaining کا جوڑا پڑھتے ہیں۔',
  'channels.openrouter_free_daily.title': 'مفت ماڈلز پر روزانہ درخواستیں',
  'channels.openrouter_free_daily.how':
    'GET /api/v1/key → {fields}۔ گنتی پر مبنی روزانہ ونڈو جو UTC دن پر ری سیٹ ہوتی ہے۔',
  'channels.openrouter_byok.title': 'BYOK خرچ اور کیا یہ حد میں شمار ہوتا ہے',
  'channels.openrouter_byok.how':
    'GET /api/v1/key → byok_usage, byok_usage_daily/weekly/monthly, include_byok_in_limit۔ وضاحت کرتا ہے کہ key کا خرچ فراہم کنندہ کے خرچ سے کیوں نہیں ملتا۔',
  'channels.zai_model_usage.title': 'ماڈل کے حساب سے استعمال (24 گھنٹے)',
  'channels.zai_model_usage.how':
    'GET api.z.ai/api/monitor/usage/model-usage?startTime=&endTime= → کل ٹوکنز، کل کالز اور 5 گھنٹے کی ونڈو کا حصہ۔ سادہ Authorization ہیڈر، جیسا پہلے سے پڑھے جانے والے کوٹا endpoint میں ہے۔',
  'channels.zai_tool_usage.title': 'MCP ٹول کا استعمال (24 گھنٹے)',
  'channels.zai_tool_usage.how':
    'GET api.z.ai/api/monitor/usage/tool-usage?startTime=&endTime= → web_search, web_reader, zread۔ ماہانہ TIME_LIMIT کی تکمیل کرتا ہے، جو آج صرف فیصد کے طور پر نظر آتی ہے۔',
  'channels.zai_absolute_counts.title': 'کوٹا کی مطلق گنتیاں',
  'channels.zai_absolute_counts.how':
    '/quota/limit کا جواب فیصد کے علاوہ ٹوکنز اور MCP کے کل ("123/1000") بھی رکھتا ہے۔ انہیں رد کر دیا جاتا ہے: انٹرفیس تک صرف فیصد پہنچتا ہے۔',
  'channels.litellm_soft_budget.title': 'الرٹ بجٹ (soft_budget)',
  'channels.litellm_soft_budget.how':
    'GET /key/info اور /team/info → soft_budget۔ یہ سخت حد نہیں: یہ وہ قدر ہے جس پر پراکسی خبردار کرتی ہے۔ آج اسے نظر انداز کیا جاتا ہے، اور انتباہ ٹرے میں کبھی نہیں آتا۔',
  'channels.litellm_temp_budget.title': 'بجٹ میں عارضی اضافہ',
  'channels.litellm_temp_budget.how':
    'key پر temp_budget_increase + temp_budget_expiry (اور ٹیم پر /team/member_update)۔ اختتامی تاریخ والی رقم کی ونڈو، جو آج نہیں پڑھی جاتی۔',
  'channels.litellm_model_info.title': 'ہر ماڈل کی قیمتیں',
  'channels.litellm_model_info.how':
    'GET /model/info → نام اور input/output_cost_per_token۔ کلائنٹ کو ٹوکنز کو رقم میں بدلنے دیتا ہے، صرف پراکسی کے کرنے کے بجائے۔',
  'channels.codex_token_counts.title': 'rollout میں سیشن ٹوکنز',
  'channels.codex_token_counts.how':
    '~/.codex/sessions/**/rollout-*.jsonl، token_count ایونٹ → total_token_usage, last_token_usage, model_context_window۔ اسی ایونٹ کا rate_limits ذیلی آبجیکٹ ہم پہلے ہی پڑھتے ہیں؛ ٹوکنز رد کر دیے جاتے ہیں۔',
  'channels.codex_reset_detail.title': 'ہر حاصل کردہ ری سیٹ کی تفصیل',
  'channels.codex_reset_detail.how':
    '{fields}۔ app-server اسے ظاہر کرتا ہے، لیکن آج ہم صرف availableCount پڑھتے ہیں؛ issue #29618 اب بھی قطاریں ظاہر نہیں کرتا۔',
  'channels.caveat.new_host':
    'نیا host اور پاتھ: پڑھنے سے پہلے docs/data-sources.md میں اپنا اندراج چاہیے۔',
  'channels.claude_statusline.title': '5 گھنٹے اور ہفتہ وار ونڈوز',
  'channels.claude_statusline.how':
    'statusLine JSON میں rate_limits.five_hour اور .seven_day (used_percentage, resets_at)، ساتھ ڈالر میں spend_limit۔',
  'channels.codex_appserver.title': 'بنیادی اور ثانوی ونڈوز، اور کریڈٹس',
  'channels.codex_appserver.how':
    'app-server پر account/rateLimits/read: windowDurationMins سے نام پانے والی ونڈوز، پلان، credits.balance اور حاصل کردہ ری سیٹس کی availableCount۔',
  'channels.litellm_budget.title': 'key، یوزر یا ٹیم کا بجٹ',
  'channels.litellm_budget.how':
    'GET /key|user|team/info: spend, max_budget, budget_duration, budget_reset_at، ہر ماڈل کے بجٹ اور مستقل rpm/tpm حدود۔',
  'channels.openrouter_activity.title': 'ماڈل کے حساب سے روزانہ خرچ',
  'channels.openrouter_activity.how':
    'GET /api/v1/activity۔ دستاویزی ہے، مگر management key چاہیے: inference key کو 403 ملتا ہے (/credits جیسی صورتحال)۔',
  'channels.litellm_global_spend.title': 'ماڈل کے حساب سے عالمی خرچ',
  'channels.litellm_global_spend.how':
    '/global/spend/models اور /global/spend/report۔ انہیں پراکسی کی master key چاہیے، یوزر کی key نہیں۔',
  'channels.opencode_console_budget.title': 'Zen بجٹ اور بیلنس',
  'channels.opencode_console_budget.how':
    'کنسول /api/billing/status، /api/billing/account، /api/usage/cost-by-day۔ موجود ہیں، لیکن صرف کنسول لاگ اِن (OAuth) کے پیچھے، پیسٹ کی گئی key سے نہیں۔',
  'channels.codex_wham.title': 'ہر فیچر کی حدود اور جمع شدہ ری سیٹس',
  'channels.codex_wham.how':
    'chatgpt.com/backend-api/wham/usage اور /wham/rate-limit-reset-credits: ہفتہ وار کوڈ ریویو کی حد، اپنی ونڈو والی نام زد حدود، ری سیٹ کی میعاد۔ یہ CLI کا نجی بیک اینڈ ہے، اپنی auth.json اور ہیڈرز کے ساتھ۔',
  'channels.antigravity_cloudcode.title': 'Cloud Code کے ذریعے کوٹا اور پلان',
  'channels.antigravity_cloudcode.how':
    'POST daily-cloudcode-pa.googleapis.com/v1internal:retrieveUserQuotaSummary اور :loadCodeAssist۔ IDE کا محفوظ کردہ OAuth ٹوکن چاہیے، جسے ایپ کبھی نہیں پڑھتی۔',
  'channels.kimi_console.title': 'کنسول میں شرح کی تفصیل',
  'channels.kimi_console.how':
    'پلیٹ فارم کنسول ہر ماڈل کی حدود ظاہر کرتا ہے، لیکن صرف ہر براؤزر سیشن کے لیے۔ نیا host نہیں، کوکی نہیں، اسکریپنگ نہیں — یہ باہر رہتا ہے۔',
  'channels.cursor_dashboard.title': 'شامل پولز اور آن ڈیمانڈ خرچ',
  'channels.cursor_dashboard.how':
    'cursor.com/api/usage-summary اور Connect-RPC GetClientVisibleCreditGrants۔ انہیں براؤزر ہیڈرز چاہیے اور edge پر 403 آ سکتا ہے۔',
} as const satisfies Record<string, string>;
