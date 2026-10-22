// hi catalog: errors. Same keys as the English file; plural keys use this language's forms.
// One title and one body for every ErrorCode (src/lib/contract/view.ts), plus the OpenCode
// ambiguity wording. src/i18n/catalog.test.ts fails when a code is missing.
export const errorMessages = {
  'error.internal.title': 'कुछ गड़बड़ हो गई',
  'error.internal.body':
    'इस स्रोत को पढ़ते समय आंतरिक त्रुटि हुई। फिर कोशिश करें; बार-बार होने पर मॉनिटर रीस्टार्ट करें।',
  'error.timeout.title': 'रीडिंग में बहुत देर लगी',
  'error.timeout.body': 'स्रोत ने समय पर जवाब नहीं दिया। थोड़ी देर में फिर कोशिश करें।',
  'error.network.title': 'स्रोत तक नहीं पहुँच सके',
  'error.network.body': 'सर्वर से संपर्क नहीं हो सका। अपना नेटवर्क जाँचें और फिर कोशिश करें।',
  'error.bad_response.title': 'अनपेक्षित जवाब',
  'error.bad_response.body': 'स्रोत ने ऐसा जवाब दिया जो मॉनिटर समझ नहीं पाया। बाद में फिर कोशिश करें।',
  'error.response_too_large.title': 'जवाब बहुत बड़ा है',
  'error.response_too_large.body': 'जवाब स्वीकार्य आकार से बड़ा था, इसलिए सुरक्षा के लिए छोड़ दिया गया।',
  'error.rate_limited.title': 'बहुत अधिक अनुरोध',
  'error.rate_limited.body':
    'स्रोत ने थोड़ा रुकने को कहा है। मॉनिटर अपने आप, लंबे अंतराल के साथ, फिर कोशिश करेगा।',
  'error.unauthorized.title': 'कुंजी या लॉगिन अस्वीकृत',
  'error.unauthorized.body': 'सर्वर ने क्रेडेंशियल अस्वीकार कर दिए। सेटिंग में कुंजी और उसका दायरा जाँचें।',
  'error.unauthorized_ambiguous.title': 'कुंजी या Go सदस्यता सत्यापित नहीं हो सकी',
  'error.unauthorized_ambiguous.body':
    'सेवा ने अनुरोध अस्वीकार कर दिया। ऐसा वैध कुंजी के साथ भी होता है और Go सदस्यता के बिना भी। OpenCode कंसोल में कुंजी और सदस्यता जाँचें।',
  'error.unauthorized_ambiguous.kimi.title': 'Kimi Code की कुंजी सत्यापित नहीं हो सकी',
  'error.unauthorized_ambiguous.kimi.body':
    'सेवा ने अनुरोध अस्वीकार कर दिया। ऐसा समाप्त हो चुकी कुंजी के साथ होता है, और Moonshot ओपन प्लैटफ़ॉर्म की कुंजी के साथ भी, जो एक अलग प्रोडक्ट है। Kimi Code कंसोल में जाँचें कि कुंजी इसी प्लान की है।',
  'error.unauthorized_ambiguous.zai.title': 'z.ai की कुंजी सत्यापित नहीं हो सकी',
  'error.unauthorized_ambiguous.zai.body':
    'सेवा ने अनुरोध अस्वीकार कर दिया। ऐसा वैध कुंजी के साथ भी होता है और GLM Coding प्लान के बिना खातों में भी। z.ai डैशबोर्ड में कुंजी और प्लान जाँचें।',
  'error.not_found.title': 'पता नहीं मिला',
  'error.not_found.body': 'सर्वर दिए गए पथ या ID को नहीं पहचानता। URL, दायरा और ID जाँचें।',
  'error.server_error.title': 'सर्वर त्रुटि',
  'error.server_error.body': 'स्रोत का सर्वर विफल हो गया। बाद में फिर कोशिश करें।',
  'error.tls_error.title': 'TLS प्रमाणपत्र विफल',
  'error.tls_error.body': 'सर्वर का प्रमाणपत्र स्वीकार नहीं किया गया। सेटिंग में सही CA चुनें।',
  'error.invalid_url.title': 'अमान्य URL',
  'error.invalid_url.body':
    'https:// का उपयोग करें। HTTP केवल localhost के लिए चलता है, और URL में उपयोगकर्ता नाम, पासवर्ड, क्वेरी पैरामीटर या # नहीं हो सकते।',
  'error.invalid_input.title': 'अमान्य मान',
  'error.invalid_input.body': 'बताए गए फ़ील्ड को ठीक करें और फिर कोशिश करें।',
  'error.settings_unreadable.title': 'सेटिंग फ़ाइल पढ़ी नहीं जा सकी',
  'error.settings_unreadable.body':
    'settings.json नहीं पढ़ी जा सकी। कुछ भी नहीं बदला गया। फ़ाइल ठीक करें और फिर कोशिश करें।',
  'error.install_failed.title': 'इंस्टॉल विफल',
  'error.install_failed.body':
    'Claude Code की स्टेटस लाइन में कमांड इंस्टॉल नहीं हो सकी। कुछ भी नहीं बदला गया।',
  'error.unsupported_plan.title': 'प्लान में पढ़ने लायक सीमाएँ नहीं हैं',
  'error.unsupported_plan.body':
    'खाता ऐसे प्लान या API कुंजी का उपयोग करता है जो सीमाएँ नहीं बताता। आप इस स्रोत को बंद कर सकते हैं।',
  'error.oauth_expired.title': 'Claude Code का लॉगिन समाप्त हो गया',
  'error.oauth_expired.body': 'लॉगिन नवीनीकृत करने के लिए Claude Code खोलें और फिर कोशिश करें।',
  'error.binary_not_found.title': 'प्रोग्राम नहीं मिला',
  'error.binary_not_found.body':
    'मॉनिटर को एक्ज़ीक्यूटेबल नहीं मिला। प्रोग्राम इंस्टॉल करें या सेटिंग में उसका पथ दें।',
  'error.spawn_failed.title': 'प्रोग्राम शुरू नहीं हो सका',
  'error.spawn_failed.body': 'प्रक्रिया शुरू नहीं हुई। एक्ज़ीक्यूटेबल का पथ जाँचें।',
  'error.handshake_failed.title': 'प्रोग्राम से संपर्क नहीं हो सका',
  'error.handshake_failed.body':
    'प्रोग्राम शुरू हुआ, पर अपेक्षा के अनुसार जवाब नहीं दिया। उसे अपडेट करें और फिर कोशिश करें।',
  'error.protocol_error.title': 'अनपेक्षित फ़ॉर्मैट',
  'error.protocol_error.body': 'डेटा का फ़ॉर्मैट बदल गया है। मॉनिटर या प्रोग्राम को अपडेट करें।',
  'error.not_signed_in.title': 'खाता कनेक्ट नहीं है',
  'error.not_signed_in.body': 'प्रोग्राम में अपने खाते में साइन इन करें और फिर कोशिश करें।',
  'error.not_running.title': 'प्रोग्राम बंद है',
  'error.not_running.body': 'इस रीडिंग के लिए प्रोग्राम खुला होना चाहिए। उसे खोलें और फिर कोशिश करें।',
  'error.endpoint_changed.title': 'स्रोत का फ़ॉर्मैट बदल गया',
  'error.endpoint_changed.body':
    'सेवा बदल गई है और यह रीडिंग अब पहले की तरह काम नहीं करती। मॉनिटर को अपडेट की ज़रूरत हो सकती है।',
  'error.http': 'HTTP कोड: {httpStatus}।',
} as const;
