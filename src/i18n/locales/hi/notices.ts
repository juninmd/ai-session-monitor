// hi catalog: notices. Same keys as the English file; plural keys use this language's forms.
// Notice codes from docs/ipc.md. An unknown code renders `notice.unknown` with the code in it,
// so a missing translation is visible instead of silent.
export const noticeMessages = {
  'notice.code.claude_waiting_first_session':
    'कमांड इंस्टॉल हो गई। सीमाएँ Claude Code सत्र में पहले जवाब के बाद आती हैं।',
  'notice.code.claude_no_rate_limits':
    'Claude Code सीमाएँ नहीं बताता (API कुंजी बिलिंग या बिना सीमा वाला प्लान)।',
  'notice.code.claude_shim_outdated':
    'Claude Code की स्टेटस लाइन में इंस्टॉल की गई कमांड नहीं मिली। रीडिंग फिर शुरू करने के लिए उसे दोबारा इंस्टॉल करें।',
  'notice.code.claude_oauth_partial':
    'मॉडल के अनुसार उपयोग (claude /usage या खाता उपयोग API) चालू है, पर वह विफल रहा ({reason})। केवल स्टेटस लाइन का डेटा दिख रहा है।',
  'notice.code.codex_rollout_fallback':
    'codex app-server ने जवाब नहीं दिया ({reason})। आखिरी रिकॉर्ड किया गया सत्र पढ़ा जा रहा है; आँकड़े पुराने हो सकते हैं।',
  'notice.code.codex_no_limits': 'Codex खाता सीमाएँ नहीं बताता (API कुंजी से लॉगिन)।',
  'notice.code.antigravity_pool_unknown':
    '{pool} कोटा से कोई डेटा नहीं मिला। यह शून्य नहीं, अज्ञात दिखता है।',
  'notice.code.litellm_no_budget': 'इस कुंजी, उपयोगकर्ता या टीम पर बजट की कोई सीमा नहीं है।',
  'notice.code.litellm_level_unreadable':
    '{level} का बजट पढ़ा नहीं जा सका ({reason})। वहाँ कोई सीमा हो सकती है जो यहाँ नहीं दिखती।',
  'notice.code.litellm_windows_truncated':
    'बजट विंडो यहाँ समाने से ज़्यादा हैं। सबसे कम उपयोग वाली छोड़ दी गईं: {dropped}।',
  'notice.code.opencode_window_idle': '{window} विंडो 0% पर है: यह अगले अनुरोध से ही गिनना शुरू करती है।',
  'notice.code.opencode_rate_limited': '{window} विंडो पर दर सीमा लगी है।',
  'notice.code.opencode_no_key':
    'केवल स्थानीय अनुमान दिख रहा है। असली विंडो देखने के लिए Go API कुंजी पेस्ट करें।',
  'notice.code.opencode_local_estimate':
    'स्थानीय मान लिस्ट प्राइस पर अनुमान हैं, और केवल इसी PC के हैं। ये कोटा नहीं हैं।',
  'notice.code.opencode_db_unreadable': 'स्थानीय OpenCode डेटाबेस पढ़ा नहीं जा सका: {reason}।',
  'notice.unknown': 'अनूदित नहीं की गई सूचना: {code}',
  'notice.action.reinstall': 'दोबारा इंस्टॉल करें',
  'notice.action.settings': 'कुंजी दर्ज करें',
  'notice.param.window.rolling': '5 घंटे की',
  'notice.param.window.weekly': 'साप्ताहिक',
  'notice.param.window.monthly': 'मासिक',
  'notice.param.level.key': 'कुंजी',
  'notice.param.level.user': 'उपयोगकर्ता',
  'notice.param.level.team': 'टीम',
  'notice.param.dropped.one': '{n} विंडो',
  'notice.param.dropped.other': '{n} विंडो',
  // The OpenCode database reasons are their own vocabulary: `not_found` there is a missing file.
  'notice.reason.opencode_db_unreadable.invalid_path': 'डेटा फ़ोल्डर का पथ अमान्य है',
  'notice.reason.opencode_db_unreadable.not_found': 'डेटाबेस फ़ाइल नहीं मिली',
  'notice.reason.opencode_db_unreadable.permission': 'डेटाबेस फ़ाइल खोलने की अनुमति नहीं है',
  'notice.reason.opencode_db_unreadable.locked': 'डेटाबेस किसी दूसरे प्रोग्राम में उपयोग हो रहा है',
  'notice.reason.opencode_db_unreadable.not_a_database': 'मिली हुई फ़ाइल डेटाबेस नहीं है',
  'notice.reason.opencode_db_unreadable.schema': 'डेटाबेस का फ़ॉर्मैट बदल गया है',
  'notice.reason.opencode_db_unreadable.timeout': 'डेटाबेस पढ़ने में बहुत देर लगी',
  'notice.reason.opencode_db_unreadable.other': 'अज्ञात कारण',
  'notice.severity.info': 'जानकारी',
  'notice.severity.warn': 'चेतावनी',
  'notice.severity.error': 'त्रुटि',
} as const;
