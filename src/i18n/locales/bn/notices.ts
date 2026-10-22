// Notice codes from docs/ipc.md. An unknown code renders `notice.unknown` with the code in it,
// so a missing translation is visible instead of silent.
export const noticeMessages = {
  'notice.code.claude_waiting_first_session':
    'কমান্ড ইনস্টল হয়েছে। Claude Code সেশনের প্রথম উত্তরের পরে সীমার তথ্য আসবে।',
  'notice.code.claude_no_rate_limits': 'Claude Code সীমার তথ্য জানায় না (API কী বিলিং বা সীমাহীন প্ল্যান)।',
  'notice.code.claude_shim_outdated':
    'Claude Code স্ট্যাটাস লাইনে ইনস্টল করা কমান্ডটি পাওয়া যায়নি। রিডিং আবার শুরু করতে সেটি পুনরায় ইনস্টল করুন।',
  'notice.code.claude_oauth_partial':
    'মডেল অনুযায়ী ব্যবহার (claude /usage বা অ্যাকাউন্ট ব্যবহারের API) চালু আছে, কিন্তু ব্যর্থ হয়েছে ({reason})। শুধু স্ট্যাটাস লাইনের ডেটা দেখানো হচ্ছে।',
  'notice.code.codex_rollout_fallback':
    'codex app-server সাড়া দেয়নি ({reason})। সর্বশেষ রেকর্ড করা সেশন থেকে পড়া হচ্ছে; সংখ্যাগুলো পুরোনো হতে পারে।',
  'notice.code.codex_no_limits': 'Codex অ্যাকাউন্ট সীমার তথ্য জানায় না (API কী দিয়ে লগইন)।',
  'notice.code.antigravity_pool_unknown':
    '{pool} কোটার কোনো ডেটা আসেনি। এটি শূন্য নয়, অজানা হিসেবে দেখানো হচ্ছে।',
  'notice.code.litellm_no_budget': 'এই কী, ইউজার বা টিমের কোনো বাজেট সীমা নেই।',
  'notice.code.litellm_level_unreadable':
    '{level} বাজেট পড়া যায়নি ({reason})। সেখানে এমন কোনো সীমা থাকতে পারে যা এখানে দেখা যাচ্ছে না।',
  'notice.code.litellm_windows_truncated':
    'এখানে দেখানোর চেয়ে বেশি বাজেট উইন্ডো আছে। সবচেয়ে কম ব্যবহৃতগুলো বাদ দেওয়া হয়েছে: {dropped}।',
  'notice.code.opencode_window_idle':
    '{window} উইন্ডো 0%-এ আছে: পরের রিকোয়েস্ট থেকেই এটি গণনা শুরু করবে।',
  'notice.code.opencode_rate_limited': '{window} উইন্ডোতে রেট লিমিট কার্যকর।',
  'notice.code.opencode_no_key': 'শুধু লোকাল অনুমান দেখানো হচ্ছে। আসল উইন্ডো দেখতে Go API কী পেস্ট করুন।',
  'notice.code.opencode_local_estimate': 'লোকাল মান লিস্ট প্রাইসে করা অনুমান, শুধু এই পিসি থেকে। এটি কোটা নয়।',
  'notice.code.opencode_db_unreadable': 'লোকাল OpenCode ডেটাবেস পড়া যায়নি: {reason}।',
  'notice.code.copilot_reset_derived':
    'রিসেটের তারিখটি অনুমান: GitHub-এর নথি অনুযায়ী প্রিমিয়াম রিকোয়েস্ট প্রতি মাসের 1 তারিখে, UTC-তে রিসেট হয়।',
  'notice.unknown': 'অনূদিত নয় এমন নোটিশ: {code}',
  'notice.action.reinstall': 'আবার ইনস্টল করুন',
  'notice.action.settings': 'কী দিন',
  'notice.param.window.rolling': '5 ঘণ্টার',
  'notice.param.window.weekly': 'সাপ্তাহিক',
  'notice.param.window.monthly': 'মাসিক',
  'notice.param.level.key': 'কী',
  'notice.param.level.user': 'ইউজার',
  'notice.param.level.team': 'টিম',
  'notice.param.dropped.one': '{n}টি উইন্ডো',
  'notice.param.dropped.other': '{n}টি উইন্ডো',
  // The OpenCode database reasons are their own vocabulary: `not_found` there is a missing file.
  'notice.reason.opencode_db_unreadable.invalid_path': 'ডেটা ফোল্ডারের পাথ অবৈধ',
  'notice.reason.opencode_db_unreadable.not_found': 'ডেটাবেস ফাইল পাওয়া যায়নি',
  'notice.reason.opencode_db_unreadable.permission': 'ডেটাবেস ফাইল খোলার অনুমতি নেই',
  'notice.reason.opencode_db_unreadable.locked': 'ডেটাবেসটি অন্য প্রোগ্রাম ব্যবহার করছে',
  'notice.reason.opencode_db_unreadable.not_a_database': 'পাওয়া ফাইলটি ডেটাবেস নয়',
  'notice.reason.opencode_db_unreadable.schema': 'ডেটাবেসের ফরম্যাট বদলে গেছে',
  'notice.reason.opencode_db_unreadable.timeout': 'ডেটাবেস পড়তে অনেক সময় লেগেছে',
  'notice.reason.opencode_db_unreadable.other': 'অজানা কারণ',
  'notice.severity.info': 'তথ্য',
  'notice.severity.warn': 'সতর্কতা',
  'notice.severity.error': 'ত্রুটি',
} as const satisfies Record<string, string>;
