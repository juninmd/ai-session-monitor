// One title and one body for every ErrorCode (src/lib/contract/view.ts), plus the OpenCode
// ambiguity wording. src/i18n/catalog.test.ts fails when a code is missing.
export const errorMessages = {
  'error.internal.title': 'কিছু একটা গোলমাল হয়েছে',
  'error.internal.body':
    'এই সোর্স পড়তে গিয়ে অভ্যন্তরীণ ত্রুটি হয়েছে। আবার চেষ্টা করুন; বারবার হলে মনিটর রিস্টার্ট করুন।',
  'error.timeout.title': 'রিডিং নিতে অনেক সময় লেগেছে',
  'error.timeout.body': 'সোর্স সময়মতো সাড়া দেয়নি। একটু পরে আবার চেষ্টা করুন।',
  'error.network.title': 'সোর্সে পৌঁছানো যাচ্ছে না',
  'error.network.body': 'সার্ভারের সঙ্গে যোগাযোগ করা যায়নি। নেটওয়ার্ক দেখে আবার চেষ্টা করুন।',
  'error.bad_response.title': 'অপ্রত্যাশিত উত্তর',
  'error.bad_response.body': 'সোর্স এমন কিছু পাঠিয়েছে যা মনিটর বুঝতে পারছে না। পরে আবার চেষ্টা করুন।',
  'error.response_too_large.title': 'উত্তরটি অনেক বড়',
  'error.response_too_large.body':
    'উত্তরটি গ্রহণযোগ্য আকারের চেয়ে বড় ছিল, তাই নিরাপত্তার জন্য বাদ দেওয়া হয়েছে।',
  'error.rate_limited.title': 'অনেক বেশি রিকোয়েস্ট',
  'error.rate_limited.body': 'সোর্স একটু অপেক্ষা করতে বলেছে। মনিটর নিজেই বড় বিরতি দিয়ে আবার চেষ্টা করবে।',
  'error.unauthorized.title': 'কী বা লগইন গ্রহণ করা হয়নি',
  'error.unauthorized.body': 'সার্ভার ক্রেডেনশিয়াল গ্রহণ করেনি। সেটিংসে কী ও তার স্কোপ দেখে নিন।',
  'error.unauthorized_ambiguous.title': 'কী বা Go সাবস্ক্রিপশন যাচাই করা যায়নি',
  'error.unauthorized_ambiguous.body':
    'সার্ভিস রিকোয়েস্টটি গ্রহণ করেনি। সঠিক কী থাকলেও এমন হয়, আবার Go সাবস্ক্রিপশন না থাকলেও হয়। OpenCode কনসোলে কী ও সাবস্ক্রিপশন দেখে নিন।',
  'error.unauthorized_ambiguous.kimi.title': 'Kimi Code কী যাচাই করা যায়নি',
  'error.unauthorized_ambiguous.kimi.body':
    'সার্ভিস রিকোয়েস্টটি গ্রহণ করেনি। কী-এর মেয়াদ শেষ হলে এমন হয়, আবার Moonshot ওপেন প্ল্যাটফর্মের কী ব্যবহার করলেও হয়, যা আলাদা প্রোডাক্ট। Kimi Code কনসোলে দেখে নিন কী-টি প্ল্যানের অন্তর্ভুক্ত কি না।',
  'error.unauthorized_ambiguous.zai.title': 'z.ai কী যাচাই করা যায়নি',
  'error.unauthorized_ambiguous.zai.body':
    'সার্ভিস রিকোয়েস্টটি গ্রহণ করেনি। সঠিক কী থাকলেও এমন হয়, আবার GLM Coding প্ল্যান ছাড়া অ্যাকাউন্টেও হয়। z.ai ড্যাশবোর্ডে কী ও প্ল্যান দেখে নিন।',
  'error.unauthorized_ambiguous.cursor.title': 'Cursor লগইন যাচাই করা যায়নি',
  'error.unauthorized_ambiguous.cursor.body':
    'সার্ভিস রিকোয়েস্টটি গ্রহণ করেনি। লগইনের মেয়াদ শেষ বা পুরোনো হলে এমন হয়, আবার সক্রিয় প্ল্যান ছাড়া অ্যাকাউন্টেও হয়। Cursor খুলে লগইন নতুন করুন এবং Cursor ড্যাশবোর্ডে প্ল্যান দেখে নিন।',
  'error.unauthorized_ambiguous.copilot.title': 'GitHub টোকেন যাচাই করা যায়নি',
  'error.unauthorized_ambiguous.copilot.body':
    'GitHub রিকোয়েস্টটি গ্রহণ করেনি। টোকেনের মেয়াদ শেষ হলে, ফাইন-গ্রেইনড বা অ্যাপ টোকেন হলে এবং Copilot ছাড়া অ্যাকাউন্টে এমন হয়। GitHub সেটিংসে টোকেন ও আপনার Copilot সাবস্ক্রিপশন দেখে নিন।',
  'error.not_found.title': 'ঠিকানা পাওয়া যায়নি',
  'error.not_found.body': 'সার্ভার দেওয়া পাথ বা আইডি চিনতে পারছে না। URL, স্কোপ ও আইডি দেখে নিন।',
  'error.server_error.title': 'সার্ভারের ত্রুটি',
  'error.server_error.body': 'সোর্সের সার্ভারে সমস্যা হয়েছে। পরে আবার চেষ্টা করুন।',
  'error.tls_error.title': 'TLS সার্টিফিকেটের সমস্যা',
  'error.tls_error.body': 'সার্ভারের সার্টিফিকেট গ্রহণ করা হয়নি। সেটিংসে সঠিক CA বেছে নিন।',
  'error.invalid_url.title': 'অবৈধ URL',
  'error.invalid_url.body':
    'https:// ব্যবহার করুন। HTTP শুধু localhost-এর জন্য চলে, আর URL-এ ইউজারনেম, পাসওয়ার্ড, কোয়েরি প্যারামিটার বা # থাকতে পারবে না।',
  'error.invalid_input.title': 'অবৈধ মান',
  'error.invalid_input.body': 'চিহ্নিত ঘরটি ঠিক করে আবার চেষ্টা করুন।',
  'error.settings_unreadable.title': 'সেটিংস ফাইল পড়া যাচ্ছে না',
  'error.settings_unreadable.body':
    'settings.json পড়া যায়নি। কিছুই বদলানো হয়নি। ফাইলটি ঠিক করে আবার চেষ্টা করুন।',
  'error.install_failed.title': 'ইনস্টল ব্যর্থ হয়েছে',
  'error.install_failed.body': 'Claude Code স্ট্যাটাস লাইনে কমান্ডটি ইনস্টল করা যায়নি। কিছুই বদলানো হয়নি।',
  'error.unsupported_plan.title': 'প্ল্যানে পড়ার মতো সীমা নেই',
  'error.unsupported_plan.body':
    'অ্যাকাউন্টটি এমন প্ল্যান বা API কী ব্যবহার করছে যা সীমার তথ্য জানায় না। এই সোর্স বন্ধ করে দিতে পারেন।',
  'error.oauth_expired.title': 'Claude Code লগইনের মেয়াদ শেষ',
  'error.oauth_expired.body': 'লগইন নবায়ন করতে Claude Code খুলুন, তারপর আবার চেষ্টা করুন।',
  'error.binary_not_found.title': 'প্রোগ্রাম পাওয়া যায়নি',
  'error.binary_not_found.body':
    'মনিটর এক্সিকিউটেবলটি খুঁজে পায়নি। প্রোগ্রামটি ইনস্টল করুন অথবা সেটিংসে তার পাথ দিন।',
  'error.spawn_failed.title': 'প্রোগ্রাম চালু করা যায়নি',
  'error.spawn_failed.body': 'প্রসেসটি শুরু হয়নি। এক্সিকিউটেবলের পাথ দেখে নিন।',
  'error.handshake_failed.title': 'প্রোগ্রামের সঙ্গে যোগাযোগ করা যায়নি',
  'error.handshake_failed.body':
    'প্রোগ্রামটি চালু হয়েছে, কিন্তু প্রত্যাশামতো সাড়া দেয়নি। এটি আপডেট করে আবার চেষ্টা করুন।',
  'error.protocol_error.title': 'অপ্রত্যাশিত ফরম্যাট',
  'error.protocol_error.body': 'ডেটার ফরম্যাট বদলে গেছে। মনিটর বা প্রোগ্রামটি আপডেট করুন।',
  'error.not_signed_in.title': 'অ্যাকাউন্ট কানেক্ট করা নেই',
  'error.not_signed_in.body': 'প্রোগ্রামে আপনার অ্যাকাউন্টে সাইন ইন করে আবার চেষ্টা করুন।',
  'error.not_running.title': 'প্রোগ্রাম বন্ধ আছে',
  'error.not_running.body': 'এই রিডিংয়ের জন্য প্রোগ্রামটি খোলা থাকতে হবে। খুলে আবার চেষ্টা করুন।',
  'error.endpoint_changed.title': 'সোর্সের ফরম্যাট বদলে গেছে',
  'error.endpoint_changed.body':
    'সার্ভিসটি বদলে গেছে, তাই এই রিডিং আর আগের মতো কাজ করছে না। মনিটর আপডেট করতে হতে পারে।',
  'error.http': 'HTTP কোড: {httpStatus}।',
} as const satisfies Record<string, string>;
