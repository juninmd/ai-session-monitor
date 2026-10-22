// bn catalog: the "Quota channels" surface (what each tool could still tell us).
export const channelsMessages = {
  'channels.page.title': 'যা এখনও পড়া সম্ভব',
  'channels.page.intro':
    '16 অক্টোবর 2026-এ প্রতিটি টুলের ডকুমেন্টেশন ও কোড খতিয়ে দেখা সমীক্ষা। প্রতিটি চ্যানেলে বলা আছে কোন এন্ডপয়েন্ট থেকে তথ্য আসে এবং অ্যাপের মডেলে সেটি কীভাবে বসে: শতকরা হার, পরিমাণ ও রিসেটসহ একটি {window}, অথবা কোনো সীমা ছাড়া একটি {fact}।',
  'channels.page.counts':
    'অ্যাপ যে ক্রেডেনশিয়াল ইতিমধ্যে গ্রহণ করে তাতেই নাগালে থাকা {now}টি নতুন চ্যানেল · {already}টি ইতিমধ্যে পড়া হয় বলে নিশ্চিত · {forbidden}টি প্রকল্পের নিয়মের বাইরে।',
  'channels.candidates.title': 'সম্ভাব্য সোর্স',
  'channels.candidates.note': 'যেসব টুলের অফিসিয়াল ব্যবহার-চ্যানেল আছে এবং ক্রেডেনশিয়াল বর্তমান মডেলে মানাবে।',
  'channels.candidates.credential': 'ক্রেডেনশিয়াল: {credential}',
  'channels.reach.already': 'ইতিমধ্যে পড়া হয়',
  'channels.reach.now': 'এখনই পড়া সম্ভব',
  'channels.reach.forbidden': 'আছে, কিন্তু নিয়মের বিরুদ্ধে',
  'channels.reach_note.already': 'আজ প্রোডাকশনে চলছে।',
  'channels.reach_note.now':
    'একই ক্রেডেনশিয়াল ও একই নিয়ম: এটি SafeClient দিয়ে যাবে এবং উইন্ডো বা ফ্যাক্ট হয়ে উঠবে। প্রতিটি অনানুষ্ঠানিক চ্যানেলের জন্য docs/data-sources.md-এ এন্ট্রি এবং ইন্টারফেসে "অনানুষ্ঠানিক" ব্যাজ লাগবে।',
  'channels.reach_note.forbidden':
    'OAuth টোকেন, কুকি, ব্রাউজার সেশন বা মাস্টার কী লাগে। অ্যাপ এসবের কিছুই করে না।',
  'channels.level.high': 'উচ্চ',
  'channels.level.medium': 'মাঝারি',
  'channels.level.low': 'নিম্ন',
  'channels.confidence': '{level} আস্থা',
  'channels.form.window': 'উইন্ডো',
  'channels.form.fact': 'ফ্যাক্ট',
  'channels.meta': '{level} আস্থা · সোর্স: {source}',
  'channels.source.zai_counts': 'ইতিমধ্যে পড়া এন্ডপয়েন্টের উত্তর',
  'channels.source.community': 'কমিউনিটি (ccusage)',
  'channels.claude_extra_usage.title': 'ব্যবহারের ক্রেডিট (মাসিক বাজেট)',
  'channels.claude_extra_usage.how':
    'GET api.anthropic.com/api/oauth/usage → {fields}, সেন্টে। CLI না থাকলে বা ব্যর্থ হলে ফলব্যাক হিসেবে এন্ডপয়েন্টটি ইতিমধ্যে কল করা হয়; কী-টি KNOWN_KEYS-এ আছে এবং আজ বাদ দেওয়া হয়।',
  'channels.claude_extra_usage.caveat': 'শুধু ঐচ্ছিক (অনানুষ্ঠানিক) OAuth পথে আছে; অপ্ট-ইন ও ব্যাজ লাগবে।',
  'channels.openrouter_budgets.title': 'স্কোপ অনুযায়ী বাজেট, রিসেটসহ',
  'channels.openrouter_budgets.how':
    'GET /api/v1/key → {fields} এবং data.effective_budget। আজ আমরা শুধু limit/limit_remaining জোড়া পড়ি।',
  'channels.openrouter_free_daily.title': 'ফ্রি মডেলে দৈনিক রিকোয়েস্ট',
  'channels.openrouter_free_daily.how':
    'GET /api/v1/key → {fields}। গণনাভিত্তিক দৈনিক উইন্ডো, যা UTC দিনের শেষে রিসেট হয়।',
  'channels.openrouter_byok.title': 'BYOK খরচ এবং তা সীমায় গোনা হয় কি না',
  'channels.openrouter_byok.how':
    'GET /api/v1/key → byok_usage, byok_usage_daily/weekly/monthly, include_byok_in_limit। কেন কী-এর খরচ প্রোভাইডারের খরচের সঙ্গে মেলে না, তা ব্যাখ্যা করে।',
  'channels.zai_model_usage.title': 'মডেল অনুযায়ী ব্যবহার (24 ঘণ্টা)',
  'channels.zai_model_usage.how':
    'GET api.z.ai/api/monitor/usage/model-usage?startTime=&endTime= → মোট টোকেন, মোট কল এবং 5 ঘণ্টার উইন্ডোর অংশ। সাধারণ Authorization হেডার, ইতিমধ্যে পড়া কোটা এন্ডপয়েন্টের মতোই।',
  'channels.zai_tool_usage.title': 'MCP টুলের ব্যবহার (24 ঘণ্টা)',
  'channels.zai_tool_usage.how':
    'GET api.z.ai/api/monitor/usage/tool-usage?startTime=&endTime= → web_search, web_reader, zread। মাসিক TIME_LIMIT-এর পরিপূরক, যা আজ শুধু শতকরা হার হিসেবে দেখা যায়।',
  'channels.zai_absolute_counts.title': 'কোটার পরম সংখ্যা',
  'channels.zai_absolute_counts.how':
    '/quota/limit-এর উত্তরে শতকরা হার ছাড়াও টোকেন ও MCP-র মোট সংখ্যা ("123/1000") থাকে। সেগুলো বাদ দেওয়া হয়: ইন্টারফেসে শুধু শতকরা হার পৌঁছায়।',
  'channels.litellm_soft_budget.title': 'সতর্কতার বাজেট (soft_budget)',
  'channels.litellm_soft_budget.how':
    'GET /key/info ও /team/info → soft_budget। এটি কঠোর সীমা নয়: প্রক্সি যে মানে সতর্ক করে সেটি। আজ এটি উপেক্ষা করা হয়, আর সতর্কতা ট্রেতে কখনো দেখা যায় না।',
  'channels.litellm_temp_budget.title': 'সাময়িক বাজেট বৃদ্ধি',
  'channels.litellm_temp_budget.how':
    'কী-এর temp_budget_increase + temp_budget_expiry (এবং টিমের জন্য /team/member_update)। শেষ তারিখসহ একটি টাকার উইন্ডো, যা আজ পড়া হয় না।',
  'channels.litellm_model_info.title': 'মডেল অনুযায়ী দাম',
  'channels.litellm_model_info.how':
    'GET /model/info → নাম ও input/output_cost_per_token। শুধু প্রক্সি নয়, ক্লায়েন্টও টোকেনকে টাকায় রূপান্তর করতে পারবে।',
  'channels.codex_token_counts.title': 'রোলআউটে সেশনের টোকেন',
  'channels.codex_token_counts.how':
    '~/.codex/sessions/**/rollout-*.jsonl, token_count ইভেন্ট → total_token_usage, last_token_usage, model_context_window। একই ইভেন্টের rate_limits সাব-অবজেক্ট আমরা ইতিমধ্যে পড়ি; টোকেনগুলো বাদ দেওয়া হয়।',
  'channels.codex_reset_detail.title': 'অর্জিত প্রতিটি রিসেটের বিস্তারিত',
  'channels.codex_reset_detail.how':
    '{fields}। app-server এটি দেয়, কিন্তু আজ আমরা শুধু availableCount পড়ি; ইস্যু #29618-এ এখনও সারিগুলো প্রকাশ করা হয়নি।',
  'channels.caveat.new_host': 'নতুন হোস্ট ও পাথ: পড়ার আগে docs/data-sources.md-এ নিজস্ব এন্ট্রি লাগবে।',
  'channels.claude_statusline.title': '5 ঘণ্টা ও সাপ্তাহিক উইন্ডো',
  'channels.claude_statusline.how':
    'statusLine JSON-এ rate_limits.five_hour ও .seven_day (used_percentage, resets_at), সঙ্গে ডলারে spend_limit।',
  'channels.codex_appserver.title': 'প্রাথমিক ও দ্বিতীয় উইন্ডো, এবং ক্রেডিট',
  'channels.codex_appserver.how':
    'app-server-এ account/rateLimits/read: windowDurationMins দিয়ে নামকরা উইন্ডো, প্ল্যান, credits.balance এবং অর্জিত রিসেটের availableCount।',
  'channels.litellm_budget.title': 'কী, ইউজার বা টিমের বাজেট',
  'channels.litellm_budget.how':
    'GET /key|user|team/info: spend, max_budget, budget_duration, budget_reset_at, মডেল অনুযায়ী বাজেট এবং স্থির rpm/tpm সীমা।',
  'channels.cursor_usage_summary.title': 'অন্তর্ভুক্ত ব্যবহার ও অন-ডিমান্ড খরচ',
  'channels.cursor_usage_summary.how':
    'GET cursor.com/api/usage-summary, Cursor তার লোকাল ডেটাবেসে (state.vscdb, একটি এন্ট্রি, শুধু-পড়ার মোডে) যে লগইন টোকেন রাখে তা থেকে তৈরি সেশন কুকি দিয়ে: totalPercentUsed, billingCycleEnd ও individualUsage.onDemand।',
  'channels.cursor_usage_summary.caveat':
    'অনানুষ্ঠানিক এবং কখনো সরাসরি দেখা হয়নি: পথটি কমিউনিটির নোট থেকে নেওয়া, তাই ড্যাশবোর্ডের সঙ্গে মিলিয়ে নিন। এর জন্য অনুমতি ও “অনানুষ্ঠানিক” ব্যাজ লাগে।',
  'channels.copilot_internal_user.title': 'প্রিমিয়াম রিকোয়েস্ট, চ্যাট ও কোড কমপ্লিশন',
  'channels.copilot_internal_user.how':
    'GET api.github.com/copilot_internal/user → quota_snapshots (percent_remaining, entitlement, remaining, unlimited, overage_count), copilot_plan ও quota_reset_date। টোকেনটি পেস্ট করা একটি, অথবা Copilot এক্সটেনশন apps.json বা hosts.json-এ যেটি রাখে।',
  'channels.copilot_internal_user.caveat':
    'অনানুষ্ঠানিক এবং কখনো সরাসরি দেখা হয়নি: পথটি GitHub-এর অভ্যন্তরীণ। উত্তরে ব্যবহারযোগ্য তারিখ না থাকলে রিসেট ধরে নেওয়া হয় (পরের মাসের 1 তারিখ, UTC) এবং অনুমান হিসেবে চিহ্নিত করা হয়।',
  'channels.openrouter_activity.title': 'মডেল অনুযায়ী দৈনিক খরচ',
  'channels.openrouter_activity.how':
    'GET /api/v1/activity। নথিভুক্ত, কিন্তু ম্যানেজমেন্ট কী লাগে: ইনফারেন্স কী 403 পায় (/credits-এর মতোই)।',
  'channels.litellm_global_spend.title': 'মডেল অনুযায়ী বৈশ্বিক খরচ',
  'channels.litellm_global_spend.how':
    '/global/spend/models ও /global/spend/report। এগুলোর জন্য প্রক্সির মাস্টার কী লাগে, ইউজারের কী নয়।',
  'channels.opencode_console_budget.title': 'Zen বাজেট ও ব্যালান্স',
  'channels.opencode_console_budget.how':
    'কনসোল /api/billing/status, /api/billing/account, /api/usage/cost-by-day। এগুলো আছে, কিন্তু শুধু কনসোল লগইনের (OAuth) পেছনে, পেস্ট করা কী দিয়ে নয়।',
  'channels.codex_wham.title': 'ফিচার অনুযায়ী সীমা ও জমানো রিসেট',
  'channels.codex_wham.how':
    'chatgpt.com/backend-api/wham/usage ও /wham/rate-limit-reset-credits: সাপ্তাহিক কোড রিভিউ সীমা, নিজস্ব উইন্ডোসহ নামকরা সীমা, রিসেটের মেয়াদ। এটি CLI-র ব্যক্তিগত ব্যাকএন্ড, নিজস্ব auth.json ও হেডারসহ।',
  'channels.antigravity_cloudcode.title': 'Cloud Code দিয়ে কোটা ও প্ল্যান',
  'channels.antigravity_cloudcode.how':
    'POST daily-cloudcode-pa.googleapis.com/v1internal:retrieveUserQuotaSummary ও :loadCodeAssist। IDE যে OAuth টোকেন সংরক্ষণ করে সেটি লাগে, যা অ্যাপ কখনো পড়ে না।',
  'channels.kimi_console.title': 'কনসোলে রেটের বিস্তারিত',
  'channels.kimi_console.how':
    'প্ল্যাটফর্ম কনসোল মডেল অনুযায়ী সীমা দেখায়, কিন্তু শুধু ব্রাউজার সেশনে। নতুন হোস্ট নয়, কুকি নয়, স্ক্র্যাপিং নয় — তাই বাদ।',
  'channels.cursor_dashboard.title': 'ক্রেডিট গ্রান্ট ও পুলের বিস্তারিত',
  'channels.cursor_dashboard.how':
    'Connect-RPC GetClientVisibleCreditGrants এবং ড্যাশবোর্ডের পেজগুলো। এগুলোর জন্য ব্রাউজার হেডার লাগে এবং এজে 403 ফেরত আসতে পারে। usage-summary পথটি Cursor-এর রাখা টোকেন দিয়ে ইতিমধ্যে পড়া হয়।',
} as const satisfies Record<string, string>;
