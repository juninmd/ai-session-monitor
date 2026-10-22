// First-run connection surfaces: consent cards, secret fields, OpenCode and LiteLLM forms.
export const setupMessages = {
  'consent.skip': 'এখন নয়',
  'consent.details': 'কী কী পড়া হবে',

  'claude.connect.title': 'Claude Code-এ কানেক্ট করুন',
  'claude.connect.lead':
    'মনিটর Claude Code স্ট্যাটাস লাইনে একটি ছোট কমান্ড যোগ করে। এটি শুধু সীমার শতকরা হার (5 ঘণ্টা ও সপ্তাহ) একটি লোকাল ফাইলে রাখে। কোনো লগইন তথ্য পড়া হয় না।',
  'claude.connect.detail':
    'ব্যাকআপ রেখে Claude Code-এর settings.json বদলায় এবং আপনার বর্তমান স্ট্যাটাস লাইনটি সঙ্গে জুড়ে রাখে। সীমার তথ্য শুধু %LOCALAPPDATA%-এ লেখে। নেটওয়ার্ক বা টোকেন ব্যবহার হয় না। Pro ও Max অ্যাকাউন্টে কাজ করে। সেশনের প্রথম উত্তরের পরে সংখ্যা আসে এবং Claude Code খোলা থাকলেই আপডেট হয়।',
  'claude.connect.allow': 'কমান্ড ইনস্টল করুন',
  'claude.connect.tip':
    'টিপ: Windows নতুন আইকন ওভারফ্লো তীর মেনুতে লুকিয়ে রাখে। সবসময় দেখা যাওয়ার জন্য মনিটরের আইকনটি টাস্কবারে টেনে আনুন।',

  'codex.connect.title': 'Codex-এ কানেক্ট করুন',
  'codex.connect.lead':
    'মনিটর এখানে ইনস্টল করা codex app-server চালু করে এবং 5 ঘণ্টার উইন্ডো ও সপ্তাহের কতটা বাকি, তা জিজ্ঞাসা করে।',
  'codex.connect.detail':
    'আপনার মেশিনে শুধু-পড়ার মোডে codex প্রোগ্রাম চালায়। কোনো ডেটা এই কম্পিউটার থেকে বাইরে যায় না, আর অ্যাপ Codex-এর লগইন ফাইল ছোঁয় না। OpenAI app-server-কে পরীক্ষামূলক হিসেবে চিহ্নিত করেছে, তাই সংস্করণ বদলালে ফরম্যাট বদলাতে পারে।',
  'codex.connect.allow': 'পড়ার অনুমতি দিন',

  'antigravity.connect.title': 'Antigravity-তে কানেক্ট করুন',
  'antigravity.connect.lead':
    'কোটা আসে ইনস্টল করা agy (CLI) বা IDE-র লোকাল সার্ভার থেকে। এই পথ অনানুষ্ঠানিক, Antigravity আপডেট হলে কাজ করা বন্ধ হতে পারে।',
  'antigravity.connect.detail':
    'আপনার অনুমতি পেলে মনিটর ইনস্টল করা agy চালিয়ে শুধু /usage পড়তে পারে (টোকেন খরচ বা লগইন তথ্য না পড়ে) এবং/অথবা লোকাল সেশন টোকেন পেতে IDE সার্ভারের কমান্ড লাইন পড়তে পারে; যোগাযোগ হয় শুধু 127.0.0.1-এর সঙ্গে। ডিস্কে কিছু লেখা হয় না, ইন্টারনেটেও কিছু পাঠানো হয় না।',
  'antigravity.connect.allow': 'লোকাল পড়ার অনুমতি দিন',

  'shim.title': 'স্ট্যাটাস লাইন কমান্ড',
  'shim.state.not_installed': 'ইনস্টল করা নেই',
  'shim.state.installed': 'ইনস্টল করা আছে',
  'shim.state.installed_chained': 'ইনস্টল করা আছে, আপনার স্ট্যাটাস লাইনের সঙ্গে জুড়ে',
  'shim.state.foreign': 'অন্য একটি স্ট্যাটাস লাইন ব্যবহৃত হচ্ছে',
  'shim.state.outdated': 'পুরোনো: এক্সিকিউটেবল পাওয়া যায়নি',
  'shim.state.unreadable': 'settings.json পড়া যাচ্ছে না',
  'shim.hint.foreign': 'ইনস্টল করলেও আপনার বর্তমান স্ট্যাটাস লাইন কাজ করতে থাকবে: মনিটর সেটিকে সঙ্গে জুড়ে নেয়।',
  'shim.hint.unreadable': 'যে ফাইল পড়া যায় না, মনিটর সেটি ছোঁয় না। settings.json ঠিক করে আবার দেখুন।',
  'shim.path': 'ফাইল: {path}',
  'shim.install': 'কমান্ড ইনস্টল করুন',
  'shim.reinstall': 'কমান্ড আবার ইনস্টল করুন',
  'shim.uninstall': 'কমান্ড সরান',
  'shim.recheck': 'আবার দেখুন',
  'shim.loading': 'Claude Code দেখা হচ্ছে',

  'secret.saved': 'কী সংরক্ষিত',
  'secret.not_saved': 'কোনো কী সংরক্ষিত নেই',
  'secret.placeholder': 'কী পেস্ট করুন',
  'secret.placeholder_keep': 'সংরক্ষিত কী রাখতে ফাঁকা রাখুন',
  'secret.hint': 'Windows Credential Manager-এ থাকে এবং ইন্টারফেসে কখনো দেখা যায় না।',
  'secret.save': 'কী সংরক্ষণ করুন',
  'secret.clear': 'কী মুছুন',
  'secret.empty': 'API কী পেস্ট করুন।',
  'secret.saved_ok': 'কী সংরক্ষিত হয়েছে।',
  'secret.cleared_ok': 'কী মুছে ফেলা হয়েছে।',

  'opencode.title': 'OpenCode Go-তে কানেক্ট করুন',
  'opencode.lead':
    'আসল উইন্ডো (5 ঘণ্টা, সপ্তাহ ও মাস) দেখতে আপনার OpenCode Go API কী পেস্ট করুন। কী ছাড়া শুধু লোকাল ব্যবহারের অনুমান দেখা যায়।',
  'opencode.key_label': 'OpenCode Go API কী',
  'opencode.console': 'OpenCode Go কনসোল খুলুন',
  'opencode.billing': 'Zen বিলিং দেখুন',
  'opencode.zen_note': 'Zen (প্রিপেইড) ব্যালান্সের কোনো API নেই: বিলিং ড্যাশবোর্ডে দেখুন।',
  'opencode.local_label': 'লোকাল অনুমান (শুধু এই পিসি)',
  'opencode.local_hint':
    'OpenCode ডেটাবেস শুধু-পড়ার মোডে খুলে লিস্ট প্রাইসে কেবল খরচ ও টোকেন যোগ করে। এটি আসল কোটা নয়।',
  'opencode.data_dir': 'OpenCode ডেটা ফোল্ডার',
  'opencode.data_dir_hint': 'ডিফল্ট ফোল্ডার ব্যবহার করতে ফাঁকা রাখুন।',
  'opencode.save_settings': 'ফোল্ডার সংরক্ষণ করুন',

  'kimi.title': 'Kimi Code-এ কানেক্ট করুন',
  'kimi.lead':
    'প্ল্যানের কোটা দেখতে আপনার Kimi Code API কী পেস্ট করুন: 5 ঘণ্টার উইন্ডো, সপ্তাহ ও মাসের মোট, প্ল্যান যেগুলো জানায়।',
  'kimi.detail':
    'অনানুষ্ঠানিক রিডিং: ব্যবহারের API নথিবিহীন এবং বিনা নোটিশে বদলাতে পারে। কী শুধু api.kimi.com-এ পাঠানো হয় এবং Windows Credential Manager-এ থাকে। অন্য টুলের কুকি বা লগইন ব্যবহার করা হয় না। Moonshot ওপেন প্ল্যাটফর্মের কী আলাদা প্রোডাক্টের, এখানে কাজ করে না।',
  'kimi.key_label': 'Kimi Code API কী',
  'kimi.console': 'Kimi Code কনসোল খুলুন',
  'kimi.note':
    'Kimi Code ব্যবহারের API-র অনানুষ্ঠানিক রিডিং। সপ্তাহ ও মাস শুধু তখনই দেখা যায় যখন প্ল্যান সেগুলো জানায়।',

  'zai.title': 'z.ai-তে কানেক্ট করুন',
  'zai.lead': '5 ঘণ্টার উইন্ডো, সপ্তাহ ও মাসিক MCP টুল কোটা দেখতে আপনার GLM Coding প্ল্যানের API কী পেস্ট করুন।',
  'zai.detail':
    'অনানুষ্ঠানিক রিডিং: কোটা API নথিবিহীন এবং বিনা নোটিশে বদলাতে পারে। কী শুধু api.z.ai-তে পাঠানো হয় এবং Windows Credential Manager-এ থাকে। অন্য টুলের কুকি বা লগইন ব্যবহার করা হয় না। BigModel অ্যাকাউন্ট (open.bigmodel.cn) পড়া হয় না।',
  'zai.key_label': 'z.ai API কী',
  'zai.console': 'z.ai কী খুলুন',
  'zai.note': 'গ্লোবাল সার্ভারে (api.z.ai) GLM Coding প্ল্যানের কোটা API-র অনানুষ্ঠানিক রিডিং।',

  'openrouter.title': 'OpenRouter-এ কানেক্ট করুন',
  'openrouter.lead':
    'কী-এর খরচের সীমা, ব্যবহার এবং (কী অনুমতি দিলে) ক্রেডিট ব্যালান্স দেখতে আপনার OpenRouter API কী পেস্ট করুন।',
  'openrouter.detail':
    'অফিসিয়াল রিডিং, OpenRouter-এর নথিভুক্ত API দিয়ে। কী শুধু openrouter.ai-তে পাঠানো হয় এবং Windows Credential Manager-এ থাকে। ক্রেডিট ব্যালান্সের জন্য ম্যানেজমেন্ট কী লাগে; সাধারণ কী দিয়ে শুধু কী-এর সীমা ও ব্যবহার দেখা যায়।',
  'openrouter.key_label': 'OpenRouter API কী',
  'openrouter.console': 'OpenRouter কী খুলুন',
  'openrouter.note':
    'OpenRouter API-র অফিসিয়াল রিডিং। কী-তে সীমা না থাকলে শতকরা হার থাকে না, শুধু ব্যবহার দেখা যায়। ব্যালান্স শুধু ম্যানেজমেন্ট কী দিয়ে দেখা যায়।',

  'deepseek.title': 'DeepSeek-এ কানেক্ট করুন',
  'deepseek.lead':
    'অ্যাকাউন্টের ব্যালান্স, যে যে মুদ্রায় ক্রেডিট আছে সেগুলোতে আলাদা করে দেখতে আপনার DeepSeek API কী পেস্ট করুন।',
  'deepseek.detail':
    'অফিসিয়াল রিডিং, DeepSeek-এর নথিভুক্ত API দিয়ে। কী শুধু api.deepseek.com-এ পাঠানো হয় এবং Windows Credential Manager-এ থাকে। ব্যালান্সের কোনো সীমা বা রিসেটের সময় নেই, তাই শতকরা হার নেই; ইউয়ান ও ডলার কখনো যোগ করা হয় না।',
  'deepseek.key_label': 'DeepSeek API কী',
  'deepseek.console': 'DeepSeek কী খুলুন',
  'deepseek.note': 'DeepSeek ব্যালান্স API-র অফিসিয়াল রিডিং। এটি টাকার অঙ্ক, কোটা নয়: শতকরা হার বা রিসেট নেই।',

  'deepinfra.title': 'DeepInfra-তে কানেক্ট করুন',
  'deepinfra.lead': 'ব্যালান্স, খরচের সীমা এবং এ মাসের খরচ দেখতে আপনার DeepInfra API কী পেস্ট করুন।',
  'deepinfra.detail':
    'অফিসিয়াল রিডিং, DeepInfra বিলিং API দিয়ে। কী শুধু api.deepinfra.com-এ পাঠানো হয় এবং Windows Credential Manager-এ থাকে। খরচের সীমা না থাকলে শতকরা হার থাকে না, শুধু ডলারের অঙ্ক দেখা যায়।',
  'deepinfra.key_label': 'DeepInfra API কী',
  'deepinfra.console': 'DeepInfra কী খুলুন',
  'deepinfra.note':
    'DeepInfra বিলিং API-র অফিসিয়াল রিডিং। মান ডলারে পড়া হয়; খরচের সীমা না থাকলে শতকরা হার নেই।',

  'moonshot.title': 'Moonshot-এ কানেক্ট করুন',
  'moonshot.lead':
    'অ্যাকাউন্টের ব্যালান্স দেখতে আপনার Moonshot প্ল্যাটফর্মের (platform.moonshot.ai) API কী পেস্ট করুন।',
  'moonshot.detail':
    'অফিসিয়াল রিডিং, Moonshot-এর নথিভুক্ত ব্যালান্স API দিয়ে। কী শুধু api.moonshot.ai-তে পাঠানো হয় এবং Windows Credential Manager-এ থাকে। kimi.com প্ল্যাটফর্মের কী আলাদা, এখানে কাজ করে না। ব্যালান্স টাকার অঙ্ক: শতকরা হার বা রিসেট নেই।',
  'moonshot.key_label': 'Moonshot API কী',
  'moonshot.console': 'Moonshot কী খুলুন',
  'moonshot.note':
    'Moonshot ব্যালান্স API-র অফিসিয়াল রিডিং, ডলারে। কিছু বকেয়া থাকলে ক্যাশ ব্যালান্স ঋণাত্মক হতে পারে।',

  'novita.title': 'Novita-তে কানেক্ট করুন',
  'novita.lead': 'অ্যাকাউন্টের ব্যালান্স, ক্রেডিট সীমা এবং অমীমাংসিত চার্জ দেখতে আপনার Novita API কী পেস্ট করুন।',
  'novita.detail':
    'অফিসিয়াল রিডিং, Novita-র নথিভুক্ত বিলিং API দিয়ে। কী শুধু api.novita.ai-তে পাঠানো হয় এবং Windows Credential Manager-এ থাকে। ব্যালান্স টাকার অঙ্ক: শতকরা হার বা রিসেট নেই।',
  'novita.key_label': 'Novita API কী',
  'novita.console': 'Novita কী খুলুন',
  'novita.note': 'Novita ব্যালান্স API-র অফিসিয়াল রিডিং, ডলারে। কিছু বকেয়া থাকলে ব্যালান্স ঋণাত্মক হতে পারে।',

  'ollama.title': 'Ollama Cloud-এ কানেক্ট করুন',
  'ollama.lead': 'প্ল্যানের অন্তর্ভুক্ত ক্রেডিট ও কেনা ব্যালান্স দেখতে একটি Ollama Cloud API কী পেস্ট করুন।',
  'ollama.detail':
    'অফিসিয়াল রিডিং, Ollama Cloud-এর নথিভুক্ত ব্যালান্স API দিয়ে। কী তৈরি করুন ollama.com/settings/keys-এ: CLI-র সাবস্ক্রিপশন কী কখনো পড়া হয় না। কী শুধু ollama.com-এ পাঠানো হয় এবং Windows Credential Manager-এ থাকে। উত্তরের ফরম্যাট প্ল্যানের ওপর নির্ভর করে (মাসিক ক্রেডিট অথবা সেশন ও সপ্তাহের উইন্ডো)।',
  'ollama.key_label': 'Ollama Cloud API কী',
  'ollama.console': 'Ollama Cloud কী খুলুন',
  'ollama.note':
    'Ollama Cloud ব্যালান্স API-র অফিসিয়াল রিডিং। বর্তমান প্ল্যানে ডলার ক্রেডিট দেখায়; পুরোনো প্ল্যানে সেশন ও সপ্তাহের শতকরা হার দেখায়।',

  'minimax.title': 'MiniMax-এ কানেক্ট করুন',
  'minimax.lead': 'মডেল অনুযায়ী কোটা দেখতে আপনার MiniMax Token Plan কী পেস্ট করুন: বর্তমান ব্যবধান ও সপ্তাহ।',
  'minimax.detail':
    'অনানুষ্ঠানিক রিডিং: Token Plan কোটা রুটের কোনো প্রকাশিত স্কিমা নেই এবং বিনা নোটিশে বদলাতে পারে। কী শুধু api.minimax.io-তে (গ্লোবাল সার্ভার) পাঠানো হয় এবং Windows Credential Manager-এ থাকে। প্রথমবার সংখ্যাগুলো MiniMax কনসোলের সঙ্গে মিলিয়ে নিন।',
  'minimax.key_label': 'MiniMax Token Plan কী',
  'minimax.console': 'MiniMax কী খুলুন',
  'minimax.note':
    'মডেল অনুযায়ী MiniMax Token Plan কোটার অনানুষ্ঠানিক রিডিং। প্ল্যানে যে মডেলের কোটা নেই, সেটি দেখানো হয় না।',

  'litellm.title': 'LiteLLM-এ কানেক্ট করুন',
  'litellm.title_edit': 'LiteLLM সংযোগ',
  'litellm.url': 'সার্ভার URL',
  'litellm.url_placeholder': 'https://litellm.yourcompany.com',
  'litellm.url_hint': 'HTTPS বাধ্যতামূলক। HTTP শুধু localhost-এর জন্য চলে।',
  'litellm.url_required': 'সার্ভার URL দিন।',
  'litellm.url_invalid': 'পূর্ণ URL দিন, যেমন https://litellm.yourcompany.com।',
  'litellm.url_http': 'https:// ব্যবহার করুন। HTTP শুধু localhost-এর জন্য চলে।',
  'litellm.url_parts': 'URL থেকে ইউজারনেম, পাসওয়ার্ড, কোয়েরি প্যারামিটার ও # সরিয়ে দিন।',
  'litellm.key': 'API কী',
  'litellm.key_required': 'LiteLLM API কী পেস্ট করুন।',
  'litellm.key_rebind': 'সংরক্ষিত কী শুধু বর্তমান সংযোগের জন্য প্রযোজ্য। সংযোগ বদলেছে, তাই কী আবার পেস্ট করুন।',
  'litellm.scope': 'কী মাপা হবে',
  'litellm.scope.key': 'এই কী',
  'litellm.scope.user': 'ইউজার',
  'litellm.scope.team': 'টিম',
  'litellm.entity.user': 'ইউজার আইডি',
  'litellm.entity.team': 'টিম আইডি',
  'litellm.entity_required': 'আইডি দিন।',
  'litellm.advanced': 'উন্নত',
  'litellm.auth_header': 'অথেনটিকেশন হেডার',
  'litellm.auth_header_hint': 'আপনার প্রক্সি কী-এর জন্য নিজস্ব হেডার ব্যবহার করলেই শুধু এটি বদলান।',
  'litellm.auth_header_invalid': 'শুধু ইংরেজি অক্ষর, সংখ্যা ও হাইফেন ব্যবহার করুন।',
  'litellm.poll': 'রিফ্রেশের ব্যবধান',
  'litellm.timeout': 'টাইমআউট',
  'litellm.tls': 'TLS সার্টিফিকেট',
  'litellm.tls.system': 'Windows থেকে',
  'litellm.tls.custom_ca': 'কাস্টম CA',
  'litellm.tls.insecure': 'যাচাই করবে না',
  'litellm.pem': 'CA ফাইল (.pem)',
  'litellm.pem_placeholder': 'C:\\certs\\ca.pem',
  'litellm.pem_required': '.pem ফাইলের পাথ দিন।',
  'litellm.insecure_host': 'যাচাই ছাড়া সার্ভার',
  'litellm.insecure_host_hint': 'সার্টিফিকেট যাচাই শুধু এই সার্ভারের জন্য বাদ যাবে।',
  'litellm.insecure_host_required': 'সার্ভার দিন।',
  'litellm.insecure_warning':
    'সংযোগ যাচাই করা হবে না এবং কী আটকে ফেলা হতে পারে। শুধু বিশ্বস্ত নেটওয়ার্কে ব্যবহার করুন।',
  'litellm.insecure_confirm': 'আমি ঝুঁকি বুঝি এবং এই সার্ভারের সার্টিফিকেট উপেক্ষা করতে চাই।',
  'litellm.insecure_confirm_required': 'সার্টিফিকেট যাচাই ছাড়া এগোতে নিশ্চিত করুন।',
  'litellm.save': 'সংরক্ষণ করে কানেক্ট করুন',
  'litellm.saving': 'সংরক্ষণ হচ্ছে…',
  'litellm.saved': 'সেটিংস সংরক্ষিত হয়েছে। সার্ভার পড়া হচ্ছে…',
  'litellm.edit': 'সংযোগ সম্পাদনা করুন',
  'litellm.delay_note':
    'সার্ভারে খরচ ও রিসেট প্রায় 10 মিনিট পর্যন্ত পিছিয়ে থাকতে পারে। সীমাটি শুধু কী-এ না হয়ে ইউজার বা টিমে থাকতে পারে।',
} as const satisfies Record<string, string>;
