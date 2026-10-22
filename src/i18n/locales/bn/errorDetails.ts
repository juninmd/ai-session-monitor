// What the params of a failure add to its message: the `reason` a collector gives (closed
// vocabularies, worded per error code), the sentences for a redirect and a wait, and the names
// of the settings fields. A reason with a `.title` replaces the title of the code as well.
export const errorDetailMessages = {
  'error.install_failed.reason.status_line_too_large.body':
    'আপনার বর্তমান স্ট্যাটাস লাইন কমান্ডটি এত বড় যে মনিটর সেটির সঙ্গে নিজের কমান্ড জুড়তে পারছে না। কিছুই বদলানো হয়নি।',
  'error.install_failed.reason.path_unsupported.body':
    'ডেটা ফোল্ডারের পাথে এমন অক্ষর আছে যা স্ট্যাটাস লাইন কমান্ড গ্রহণ করে না। কিছুই বদলানো হয়নি।',
  'error.install_failed.reason.serialize_failed.body':
    'নতুন settings.json তৈরি করা যায়নি। কিছুই বদলানো হয়নি।',
  'error.install_failed.reason.state_failed.body':
    'ইনস্টলের অবস্থা সংরক্ষণ করা যায়নি। settings.json বদলানো হয়নি।',
  'error.install_failed.reason.concurrent_modification.body':
    'কাজ চলার সময় settings.json বদলে গেছে, তাই মনিটর ফাইলটি ছুঁয়ে দেখেনি। আবার চেষ্টা করুন।',
  'error.install_failed.reason.write_failed.body':
    'settings.json লেখা যায়নি। ফাইলটি বদলানো হয়নি; ফাইলটি লক করা নেই কি না দেখে আবার চেষ্টা করুন।',
  'error.install_failed.reason.shim_missing.body':
    'মনিটরের পাশে কমান্ডের এক্সিকিউটেবল পাওয়া যায়নি। মনিটর আবার ইনস্টল করুন।',
  'error.install_failed.reason.copy_failed.body':
    'কমান্ডের এক্সিকিউটেবল ডেটা ফোল্ডারে কপি করা যায়নি। settings.json বদলানো হয়নি।',
  'error.install_failed.reason.backup_failed.body':
    'settings.json-এর ব্যাকআপ সংরক্ষণ করা যায়নি, তাই ফাইলটি বদলানো হয়নি।',
  // Only a removal is refused for this reason.
  'error.install_failed.reason.foreign_status_line.title': 'কমান্ডটি সরানো যায়নি',
  'error.install_failed.reason.foreign_status_line.body':
    'কনফিগার করা স্ট্যাটাস লাইনটি মনিটরের নয় (হয়তো হাতে বদলানো কপি), তাই কিছুই সরানো হয়নি।',

  // LiteLLM: the CA file the user chose, not the certificate of the server.
  'error.tls_error.reason.ca_missing.body': 'কনফিগার করা CA ফাইল পাওয়া যায়নি। সেটিংসে সঠিক ফাইল বেছে নিন।',
  'error.tls_error.reason.ca_too_large.body':
    'CA ফাইলটি পড়ার জন্য অনেক বড়। সেটিংসে ছোট একটি .pem ফাইল বেছে নিন।',
  'error.tls_error.reason.ca_unreadable.body': 'CA ফাইল পড়া যায়নি। সেটিংসে পাথ ও অনুমতি দেখে নিন।',
  'error.tls_error.reason.ca_invalid.body':
    'CA ফাইলে বৈধ সার্টিফিকেট নেই। সেটিংসে একটি বৈধ .pem ফাইল বেছে নিন।',

  // LiteLLM: which part of the answer did not look like a key, user or team record.
  'error.endpoint_changed.reason.not_an_object.body':
    'সার্ভারের উত্তর প্রত্যাশিত ফরম্যাটে নেই। URL ঠিক LiteLLM-এর দিকে যাচ্ছে কি না দেখে নিন।',
  'error.endpoint_changed.reason.no_record.body':
    'উত্তরে কী, ইউজার বা টিমের কোনো রেকর্ড নেই। URL ও স্কোপ দেখে নিন।',
  'error.endpoint_changed.reason.id_missing.body':
    'যে আইডি খোঁজা হয়েছিল, উত্তরে তার নিশ্চিতকরণ নেই। ইউজার বা টিমের আইডি দেখে নিন।',
  'error.endpoint_changed.reason.id_mismatch.body':
    'উত্তরটি দেওয়া আইডির বদলে অন্য একটি আইডির। ইউজার বা টিমের আইডি দেখে নিন।',

  // Antigravity: the IDE's local server is there, but there is nothing to call.
  'error.not_running.reason.no_port.title': 'লোকাল সার্ভার এখনও প্রস্তুত নয়',
  'error.not_running.reason.no_port.body':
    'Antigravity খোলা আছে, কিন্তু তার লোকাল সার্ভার এখনও কোনো ঠিকানা দিচ্ছে না। সাধারণত কিছুক্ষণেই ঠিক হয়ে যায়: আবার চেষ্টা করুন।',
  'error.unauthorized.reason.no_token.title': 'লোকাল টোকেন পাওয়া যাচ্ছে না',
  'error.unauthorized.reason.no_token.body':
    'মনিটর IDE-র লোকাল সার্ভারের সেশন টোকেন পড়তে পারেনি। Antigravity রিস্টার্ট করে আবার চেষ্টা করুন।',

  'error.redirect': 'সার্ভার রিকোয়েস্টটি রিডাইরেক্ট করার চেষ্টা করেছে, আর মনিটর রিডাইরেক্ট অনুসরণ করে না।',
  'error.retry_after': 'সোর্সের চাওয়া অপেক্ষা: {wait}।',
  'error.field_hint': 'সমস্যাযুক্ত ঘর: {field}।',
  'field.value': 'API কী',
  'field.baseUrl': 'সার্ভার URL',
  'field.entityId': 'ইউজার বা টিমের আইডি',
  'field.authHeader': 'অথেনটিকেশন হেডার',
  'field.pemPath': 'CA ফাইল (.pem)',
  'field.host': 'সার্টিফিকেট যাচাই ছাড়া সার্ভার',
  'field.configDir': 'Claude Code কনফিগারেশন ফোল্ডার',
  'field.binaryPath': 'Codex এক্সিকিউটেবলের পাথ',
  'field.dataDir': 'OpenCode ডেটা ফোল্ডার',
  'field.shimConsent': 'কমান্ড ইনস্টলের সম্মতি',
  'field.limit': 'আগের উইন্ডোর সংখ্যা',
} as const satisfies Record<string, string>;
