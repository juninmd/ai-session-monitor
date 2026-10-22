// ar catalog: the "Quota channels" surface (what each tool could still tell us).
export const channelsMessages = {
  'channels.page.title': 'ما يمكننا قراءته بعد',
  'channels.page.intro':
    'مسح بتاريخ 16 أكتوبر 2026 لوثائق كل أداة وشيفرتها. تحدد كل قناة نقطة النهاية التي توفّرها وكيف تلائم نموذج التطبيق: إما {window} بنسبة مئوية ومبلغ وإعادة ضبط، أو {fact} بلا حد.',
  'channels.page.counts':
    'قنوات جديدة في متناول بيانات الاعتماد التي يقبلها التطبيق بالفعل: {now} · مؤكَّد أنها تُقرأ بالفعل: {already} · خارج قواعد المشروع: {forbidden}.',
  'channels.candidates.title': 'مصادر مرشّحة',
  'channels.candidates.note': 'أدوات لها قناة استخدام رسمية وبيانات اعتماد تلائم النموذج الحالي.',
  'channels.candidates.credential': 'بيانات الاعتماد: {credential}',
  'channels.reach.already': 'تُقرأ بالفعل',
  'channels.reach.now': 'قابلة للقراءة الآن',
  'channels.reach.forbidden': 'موجودة، لكنها تخالف القواعد',
  'channels.reach_note.already': 'قيد الإنتاج حاليًا.',
  'channels.reach_note.now':
    'بيانات الاعتماد نفسها والقاعدة نفسها: ستمر عبر SafeClient وتصبح نافذة أو معلومة. وتحتاج كل قناة غير رسمية إلى مدخل في docs/data-sources.md وشارة "غير رسمي" في الواجهة.',
  'channels.reach_note.forbidden':
    'تحتاج إلى رمز OAuth أو ملف تعريف ارتباط أو جلسة متصفح أو مفتاح رئيسي. والتطبيق لا يفعل شيئًا من ذلك.',
  'channels.level.high': 'عالية',
  'channels.level.medium': 'متوسطة',
  'channels.level.low': 'منخفضة',
  'channels.confidence': 'ثقة {level}',
  'channels.form.window': 'نافذة',
  'channels.form.fact': 'معلومة',
  'channels.meta': 'ثقة {level} · المصدر: {source}',
  'channels.source.zai_counts': 'استجابة نقطة النهاية التي تُقرأ بالفعل',
  'channels.source.community': 'المجتمع (ccusage)',
  'channels.claude_extra_usage.title': 'رصيد الاستخدام (ميزانية شهرية)',
  'channels.claude_extra_usage.how':
    'GET api.anthropic.com/api/oauth/usage → {fields}، بالسنتات. تُستدعى نقطة النهاية بالفعل كمسار بديل عند غياب CLI أو فشله؛ والمفتاح موجود في KNOWN_KEYS ويُهمل حاليًا.',
  'channels.claude_extra_usage.caveat':
    'موجودة فقط في مسار OAuth الاختياري (غير الرسمي)؛ تحتاج إلى موافقة وشارة.',
  'channels.openrouter_budgets.title': 'ميزانيات حسب النطاق، مع إعادة ضبط',
  'channels.openrouter_budgets.how':
    'GET /api/v1/key → {fields} وdata.effective_budget. نقرأ حاليًا الزوج limit/limit_remaining فقط.',
  'channels.openrouter_free_daily.title': 'الطلبات اليومية على النماذج المجانية',
  'channels.openrouter_free_daily.how':
    'GET /api/v1/key → {fields}. نافذة يومية تعتمد على العدّ وتُعاد ضبطها مع يوم UTC.',
  'channels.openrouter_byok.title': 'إنفاق BYOK وهل يُحتسب ضمن الحد',
  'channels.openrouter_byok.how':
    'GET /api/v1/key → byok_usage وbyok_usage_daily/weekly/monthly وinclude_byok_in_limit. يفسّر سبب اختلاف إنفاق المفتاح عن إنفاق المزوّد.',
  'channels.zai_model_usage.title': 'الاستخدام حسب النموذج (24 ساعة)',
  'channels.zai_model_usage.how':
    'GET api.z.ai/api/monitor/usage/model-usage?startTime=&endTime= → إجمالي الرموز وإجمالي الاستدعاءات ونسبة نافذة 5 ساعات. ترويسة Authorization عادية، مثل نقطة نهاية الحصة التي تُقرأ بالفعل.',
  'channels.zai_tool_usage.title': 'استخدام أدوات MCP (24 ساعة)',
  'channels.zai_tool_usage.how':
    'GET api.z.ai/api/monitor/usage/tool-usage?startTime=&endTime= → web_search وweb_reader وzread. تكمّل TIME_LIMIT الشهرية التي لا تظهر حاليًا إلا كنسبة مئوية.',
  'channels.zai_absolute_counts.title': 'أعداد الحصة المطلقة',
  'channels.zai_absolute_counts.how':
    'تحمل استجابة /quota/limit بالفعل، إلى جانب النسبة المئوية، إجماليات الرموز وMCP ("123/1000"). وتُهمل: لا يصل إلى الواجهة سوى النسبة المئوية.',
  'channels.litellm_soft_budget.title': 'ميزانية التنبيه (soft_budget)',
  'channels.litellm_soft_budget.how':
    'GET /key/info و/team/info → soft_budget. ليست حدًا صارمًا: إنها القيمة التي ينبّه عندها الوكيل. تُتجاهل حاليًا، ولا يظهر التحذير في شريط النظام.',
  'channels.litellm_temp_budget.title': 'زيادة مؤقتة في الميزانية',
  'channels.litellm_temp_budget.how':
    'temp_budget_increase + temp_budget_expiry على المفتاح (و/team/member_update على الفريق). نافذة مالية لها تاريخ انتهاء، ولا تُقرأ حاليًا.',
  'channels.litellm_model_info.title': 'أسعار كل نموذج',
  'channels.litellm_model_info.how':
    'GET /model/info → الأسماء وinput/output_cost_per_token. تتيح للعميل تحويل الرموز إلى مال بدل أن يقوم الوكيل وحده بذلك.',
  'channels.codex_token_counts.title': 'رموز الجلسة في rollout',
  'channels.codex_token_counts.how':
    '~/.codex/sessions/**/rollout-*.jsonl، الحدث token_count → total_token_usage وlast_token_usage وmodel_context_window. نقرأ بالفعل الكائن الفرعي rate_limits من الحدث نفسه؛ أما الرموز فتُهمل.',
  'channels.codex_reset_detail.title': 'تفاصيل كل إعادة ضبط مكتسبة',
  'channels.codex_reset_detail.how':
    '{fields}. يوفّرها app-server، لكننا نقرأ حاليًا availableCount فقط؛ ولا تزال المشكلة #29618 لا تكشف الصفوف.',
  'channels.caveat.new_host':
    'مضيف ومسار جديدان: يحتاجان إلى مدخل خاص في docs/data-sources.md قبل القراءة.',
  'channels.claude_statusline.title': 'نافذتا 5 ساعات والأسبوع',
  'channels.claude_statusline.how':
    'rate_limits.five_hour و.seven_day (used_percentage وresets_at) في JSON الخاص بـ statusLine، إضافةً إلى spend_limit بالدولار.',
  'channels.codex_appserver.title': 'النافذتان الأساسية والثانوية، والرصيد',
  'channels.codex_appserver.how':
    'account/rateLimits/read على app-server: نوافذ تُسمّى بحسب windowDurationMins، والباقة، وcredits.balance وavailableCount لإعادات الضبط المكتسبة.',
  'channels.litellm_budget.title': 'ميزانية المفتاح أو المستخدم أو الفريق',
  'channels.litellm_budget.how':
    'GET /key|user|team/info: spend وmax_budget وbudget_duration وbudget_reset_at، وميزانيات كل نموذج، وحدود rpm/tpm الثابتة.',
  'channels.openrouter_activity.title': 'الإنفاق اليومي حسب النموذج',
  'channels.openrouter_activity.how':
    'GET /api/v1/activity. موثّقة، لكنها تحتاج إلى مفتاح إدارة: يحصل مفتاح الاستدلال على 403 (الحال نفسه مع /credits).',
  'channels.litellm_global_spend.title': 'الإنفاق الشامل حسب النموذج',
  'channels.litellm_global_spend.how':
    '/global/spend/models و/global/spend/report. تحتاجان إلى المفتاح الرئيسي للوكيل، لا إلى مفتاح المستخدم.',
  'channels.opencode_console_budget.title': 'ميزانية Zen ورصيدها',
  'channels.opencode_console_budget.how':
    'Console /api/billing/status و/api/billing/account و/api/usage/cost-by-day. موجودة، لكن خلف تسجيل دخول Console فقط (OAuth)، وليس بالمفتاح الملصوق.',
  'channels.codex_wham.title': 'حدود كل ميزة وإعادات الضبط المخزّنة',
  'channels.codex_wham.how':
    'chatgpt.com/backend-api/wham/usage و/wham/rate-limit-reset-credits: حد مراجعة الشيفرة الأسبوعي، وحدود مسمّاة لها نافذتها الخاصة، وانتهاء إعادات الضبط. إنها الخلفية الخاصة بـ CLI، ولها auth.json وترويسات خاصة بها.',
  'channels.antigravity_cloudcode.title': 'الحصة والباقة عبر Cloud Code',
  'channels.antigravity_cloudcode.how':
    'POST daily-cloudcode-pa.googleapis.com/v1internal:retrieveUserQuotaSummary و:loadCodeAssist. تحتاج إلى رمز OAuth الذي تحفظه بيئة التطوير، ولا يقرؤه التطبيق أبدًا.',
  'channels.kimi_console.title': 'تفاصيل المعدّل في وحدة التحكم',
  'channels.kimi_console.how':
    'تعرض وحدة تحكم المنصة حدودًا لكل نموذج، لكن لكل جلسة متصفح فقط. لا مضيف جديد ولا ملف تعريف ارتباط ولا استخراج من الصفحات — فتبقى خارج النطاق.',
  'channels.cursor_dashboard.title': 'المجموعات المضمّنة والإنفاق عند الطلب',
  'channels.cursor_dashboard.how':
    'cursor.com/api/usage-summary وConnect-RPC GetClientVisibleCreditGrants. تحتاجان إلى ترويسات المتصفح وقد تُرجعان 403 عند الحافة.',
} as const satisfies Record<string, string>;
