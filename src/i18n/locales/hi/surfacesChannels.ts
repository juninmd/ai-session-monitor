// hi catalog: the "Quota channels" surface (what each tool could still tell us).
export const channelsMessages = {
  'channels.page.title': 'जो हम अब भी पढ़ सकते हैं',
  'channels.page.intro':
    '16 अक्टूबर 2026 का सर्वेक्षण, हर टूल के डॉक्स और कोड पर। हर चैनल बताता है कि कौन सा एंडपॉइंट उसे देता है और वह ऐप के मॉडल में कैसे बैठता है: प्रतिशत, राशि और रीसेट वाली {window}, या बिना सीमा वाला {fact}।',
  'channels.page.counts':
    '{now} नए चैनल उस क्रेडेंशियल की पहुँच में जिसे ऐप पहले से स्वीकार करता है · {already} पहले से पढ़े जा रहे, पुष्टि हुई · {forbidden} प्रोजेक्ट के नियमों से बाहर।',
  'channels.candidates.title': 'संभावित स्रोत',
  'channels.candidates.note':
    'ऐसे टूल जिनका आधिकारिक उपयोग चैनल है और जिनका क्रेडेंशियल मौजूदा मॉडल में फिट होगा।',
  'channels.candidates.credential': 'क्रेडेंशियल: {credential}',
  'channels.reach.already': 'पहले से पढ़ा जा रहा',
  'channels.reach.now': 'अभी पढ़ा जा सकता है',
  'channels.reach.forbidden': 'है, पर नियमों के खिलाफ़',
  'channels.reach_note.already': 'आज प्रोडक्शन में है।',
  'channels.reach_note.now':
    'वही क्रेडेंशियल और वही नियम: यह SafeClient से गुज़रेगा और विंडो या फ़ैक्ट बन जाएगा। हर अनौपचारिक चैनल के लिए अब भी docs/data-sources.md में प्रविष्टि और इंटरफ़ेस में "अनौपचारिक" बैज चाहिए।',
  'channels.reach_note.forbidden':
    'इसके लिए OAuth टोकन, कुकी, ब्राउज़र सत्र या मास्टर कुंजी चाहिए। ऐप इनमें से कुछ नहीं करता।',
  'channels.level.high': 'उच्च',
  'channels.level.medium': 'मध्यम',
  'channels.level.low': 'निम्न',
  'channels.confidence': 'भरोसा: {level}',
  'channels.form.window': 'विंडो',
  'channels.form.fact': 'फ़ैक्ट',
  'channels.meta': 'भरोसा: {level} · स्रोत: {source}',
  'channels.source.zai_counts': 'पहले से पढ़े जा रहे एंडपॉइंट का जवाब',
  'channels.source.community': 'कम्युनिटी (ccusage)',
  'channels.claude_extra_usage.title': 'उपयोग क्रेडिट (मासिक बजट)',
  'channels.claude_extra_usage.how':
    'GET api.anthropic.com/api/oauth/usage → {fields}, सेंट में। CLI न हो या विफल हो तो यह एंडपॉइंट पहले से फ़ॉलबैक के रूप में बुलाया जाता है; कुंजी KNOWN_KEYS में है और आज छोड़ दी जाती है।',
  'channels.claude_extra_usage.caveat':
    'केवल वैकल्पिक (अनौपचारिक) OAuth रास्ते पर मौजूद; ऑप्ट-इन और बैज चाहिए।',
  'channels.openrouter_budgets.title': 'दायरे के अनुसार बजट, रीसेट के साथ',
  'channels.openrouter_budgets.how':
    'GET /api/v1/key → {fields} और data.effective_budget। आज हम केवल limit/limit_remaining जोड़ी पढ़ते हैं।',
  'channels.openrouter_free_daily.title': 'मुफ़्त मॉडल पर दैनिक अनुरोध',
  'channels.openrouter_free_daily.how':
    'GET /api/v1/key → {fields}। गिनती पर आधारित दैनिक विंडो, जो UTC दिन पर रीसेट होती है।',
  'channels.openrouter_byok.title': 'BYOK खर्च और क्या वह सीमा में गिना जाता है',
  'channels.openrouter_byok.how':
    'GET /api/v1/key → byok_usage, byok_usage_daily/weekly/monthly, include_byok_in_limit। बताता है कि कुंजी का खर्च प्रदाता के खर्च से क्यों नहीं मिलता।',
  'channels.zai_model_usage.title': 'मॉडल के अनुसार उपयोग (24 घंटे)',
  'channels.zai_model_usage.how':
    'GET api.z.ai/api/monitor/usage/model-usage?startTime=&endTime= → कुल टोकन, कुल कॉल और 5 घंटे की विंडो का हिस्सा। सादा Authorization हेडर, पहले से पढ़े जा रहे कोटा एंडपॉइंट की तरह।',
  'channels.zai_tool_usage.title': 'MCP टूल का उपयोग (24 घंटे)',
  'channels.zai_tool_usage.how':
    'GET api.z.ai/api/monitor/usage/tool-usage?startTime=&endTime= → web_search, web_reader, zread। मासिक TIME_LIMIT का पूरक, जो आज केवल प्रतिशत के रूप में दिखता है।',
  'channels.zai_absolute_counts.title': 'कोटा की पूर्ण गिनती',
  'channels.zai_absolute_counts.how':
    '/quota/limit का जवाब प्रतिशत के अलावा टोकन और MCP के कुल भी देता है ("123/1000")। उन्हें छोड़ दिया जाता है: इंटरफ़ेस तक केवल प्रतिशत पहुँचता है।',
  'channels.litellm_soft_budget.title': 'अलर्ट बजट (soft_budget)',
  'channels.litellm_soft_budget.how':
    'GET /key/info और /team/info → soft_budget। यह कड़ी सीमा नहीं है: यह वह मान है जिस पर प्रॉक्सी चेतावनी देता है। आज इसे अनदेखा किया जाता है, और चेतावनी ट्रे में कभी नहीं दिखती।',
  'channels.litellm_temp_budget.title': 'अस्थायी बजट वृद्धि',
  'channels.litellm_temp_budget.how':
    'कुंजी पर temp_budget_increase + temp_budget_expiry (और टीम पर /team/member_update)। समाप्ति तिथि वाली पैसे की विंडो, जो आज नहीं पढ़ी जाती।',
  'channels.litellm_model_info.title': 'मॉडल के अनुसार कीमतें',
  'channels.litellm_model_info.how':
    'GET /model/info → नाम और input/output_cost_per_token। इससे क्लाइंट टोकन को पैसे में बदल सकता है, सिर्फ़ प्रॉक्सी पर निर्भर नहीं रहना पड़ता।',
  'channels.codex_token_counts.title': 'रोलआउट में सत्र के टोकन',
  'channels.codex_token_counts.how':
    '~/.codex/sessions/**/rollout-*.jsonl, token_count इवेंट → total_token_usage, last_token_usage, model_context_window। उसी इवेंट का rate_limits उप-ऑब्जेक्ट हम पहले से पढ़ते हैं; टोकन छोड़ दिए जाते हैं।',
  'channels.codex_reset_detail.title': 'हर कमाए गए रीसेट का विवरण',
  'channels.codex_reset_detail.how':
    '{fields}। app-server इसे देता है, पर आज हम केवल availableCount पढ़ते हैं; issue #29618 में पंक्तियाँ अब भी उपलब्ध नहीं कराई गई हैं।',
  'channels.caveat.new_host':
    'नया होस्ट और पथ: पढ़ने से पहले docs/data-sources.md में उसकी अपनी प्रविष्टि चाहिए।',
  'channels.claude_statusline.title': '5 घंटे और साप्ताहिक विंडो',
  'channels.claude_statusline.how':
    'statusLine JSON में rate_limits.five_hour और .seven_day (used_percentage, resets_at), साथ में डॉलर में spend_limit।',
  'channels.codex_appserver.title': 'प्राथमिक और द्वितीयक विंडो, और क्रेडिट',
  'channels.codex_appserver.how':
    'app-server पर account/rateLimits/read: windowDurationMins से नामित विंडो, प्लान, credits.balance और कमाए गए रीसेट का availableCount।',
  'channels.litellm_budget.title': 'कुंजी, उपयोगकर्ता या टीम का बजट',
  'channels.litellm_budget.how':
    'GET /key|user|team/info: spend, max_budget, budget_duration, budget_reset_at, मॉडल के अनुसार बजट और स्थिर rpm/tpm सीमाएँ।',
  'channels.openrouter_activity.title': 'मॉडल के अनुसार दैनिक खर्च',
  'channels.openrouter_activity.how':
    'GET /api/v1/activity। प्रलेखित है, पर मैनेजमेंट कुंजी चाहिए: इन्फ़रेंस कुंजी को 403 मिलता है (/credits जैसी ही स्थिति)।',
  'channels.litellm_global_spend.title': 'मॉडल के अनुसार ग्लोबल खर्च',
  'channels.litellm_global_spend.how':
    '/global/spend/models और /global/spend/report। इनके लिए प्रॉक्सी की मास्टर कुंजी चाहिए, उपयोगकर्ता की कुंजी नहीं।',
  'channels.opencode_console_budget.title': 'Zen बजट और बैलेंस',
  'channels.opencode_console_budget.how':
    'कंसोल /api/billing/status, /api/billing/account, /api/usage/cost-by-day। ये मौजूद हैं, पर केवल कंसोल लॉगिन (OAuth) के पीछे, पेस्ट की गई कुंजी से नहीं।',
  'channels.codex_wham.title': 'फ़ीचर के अनुसार सीमाएँ और जमा रीसेट',
  'channels.codex_wham.how':
    'chatgpt.com/backend-api/wham/usage और /wham/rate-limit-reset-credits: साप्ताहिक कोड रिव्यू सीमा, अपनी विंडो वाली नामित सीमाएँ, रीसेट की समाप्ति। यह CLI का निजी बैकएंड है, अपनी auth.json और हेडर के साथ।',
  'channels.antigravity_cloudcode.title': 'Cloud Code के ज़रिए कोटा और प्लान',
  'channels.antigravity_cloudcode.how':
    'POST daily-cloudcode-pa.googleapis.com/v1internal:retrieveUserQuotaSummary और :loadCodeAssist। इसके लिए IDE द्वारा सहेजा गया OAuth टोकन चाहिए, जिसे ऐप कभी नहीं पढ़ता।',
  'channels.kimi_console.title': 'कंसोल में दर का विवरण',
  'channels.kimi_console.how':
    'प्लैटफ़ॉर्म कंसोल मॉडल के अनुसार सीमाएँ दिखाता है, पर केवल ब्राउज़र सत्र के भीतर। न नया होस्ट, न कुकी, न स्क्रैपिंग — यह बाहर ही रहता है।',
  'channels.cursor_dashboard.title': 'शामिल पूल और ऑन-डिमांड खर्च',
  'channels.cursor_dashboard.how':
    'cursor.com/api/usage-summary और Connect-RPC GetClientVisibleCreditGrants। इनके लिए ब्राउज़र हेडर चाहिए और एज पर 403 मिल सकता है।',
} as const;
