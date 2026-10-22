// hi catalog: errorDetails. Same keys as the English file; plural keys use this language's forms.
// What the params of a failure add to its message: the `reason` a collector gives (closed
// vocabularies, worded per error code), the sentences for a redirect and a wait, and the names
// of the settings fields. A reason with a `.title` replaces the title of the code as well.
export const errorDetailMessages = {
  'error.install_failed.reason.status_line_too_large.body':
    'आपकी मौजूदा स्टेटस लाइन कमांड मॉनिटर के जोड़ने के लिए बहुत बड़ी है। कुछ भी नहीं बदला गया।',
  'error.install_failed.reason.path_unsupported.body':
    'डेटा फ़ोल्डर के पथ में ऐसे अक्षर हैं जिन्हें स्टेटस लाइन कमांड स्वीकार नहीं करती। कुछ भी नहीं बदला गया।',
  'error.install_failed.reason.serialize_failed.body':
    'नई settings.json नहीं बन सकी। कुछ भी नहीं बदला गया।',
  'error.install_failed.reason.state_failed.body':
    'इंस्टॉल की स्थिति सहेजी नहीं जा सकी। settings.json नहीं बदली गई।',
  'error.install_failed.reason.concurrent_modification.body':
    'ऑपरेशन के दौरान settings.json बदल गई, इसलिए मॉनिटर ने उसे नहीं छुआ। फिर कोशिश करें।',
  'error.install_failed.reason.write_failed.body':
    'settings.json में लिखा नहीं जा सका। फ़ाइल नहीं बदली गई; जाँचें कि वह लॉक नहीं है और फिर कोशिश करें।',
  'error.install_failed.reason.shim_missing.body':
    'कमांड का एक्ज़ीक्यूटेबल मॉनिटर के बगल में नहीं मिला। मॉनिटर दोबारा इंस्टॉल करें।',
  'error.install_failed.reason.copy_failed.body':
    'कमांड का एक्ज़ीक्यूटेबल डेटा फ़ोल्डर में कॉपी नहीं हो सका। settings.json नहीं बदली गई।',
  'error.install_failed.reason.backup_failed.body':
    'settings.json का बैकअप सहेजा नहीं जा सका, इसलिए उसे नहीं बदला गया।',
  // Only a removal is refused for this reason.
  'error.install_failed.reason.foreign_status_line.title': 'कमांड हटाई नहीं जा सकी',
  'error.install_failed.reason.foreign_status_line.body':
    'कॉन्फ़िगर की गई स्टेटस लाइन मॉनिटर की नहीं है (शायद हाथ से बदली गई कॉपी), इसलिए कुछ नहीं हटाया गया।',

  // LiteLLM: the CA file the user chose, not the certificate of the server.
  'error.tls_error.reason.ca_missing.body': 'कॉन्फ़िगर की गई CA फ़ाइल नहीं मिली। सेटिंग में सही फ़ाइल चुनें।',
  'error.tls_error.reason.ca_too_large.body':
    'CA फ़ाइल पढ़ने के लिए बहुत बड़ी है। सेटिंग में छोटी .pem फ़ाइल चुनें।',
  'error.tls_error.reason.ca_unreadable.body': 'CA फ़ाइल पढ़ी नहीं जा सकी। सेटिंग में पथ और अनुमतियाँ जाँचें।',
  'error.tls_error.reason.ca_invalid.body': 'CA फ़ाइल में वैध प्रमाणपत्र नहीं है। सेटिंग में वैध .pem फ़ाइल चुनें।',

  // LiteLLM: which part of the answer did not look like a key, user or team record.
  'error.endpoint_changed.reason.not_an_object.body':
    'सर्वर का जवाब अपेक्षित फ़ॉर्मैट में नहीं है। जाँचें कि URL LiteLLM की ओर इशारा करता है।',
  'error.endpoint_changed.reason.no_record.body':
    'जवाब में कुंजी, उपयोगकर्ता या टीम का कोई रिकॉर्ड नहीं है। URL और दायरा जाँचें।',
  'error.endpoint_changed.reason.id_missing.body':
    'जवाब उस ID की पुष्टि नहीं करता जिसे पूछा गया था। उपयोगकर्ता या टीम का ID जाँचें।',
  'error.endpoint_changed.reason.id_mismatch.body':
    'जवाब दिए गए ID से अलग ID का है। उपयोगकर्ता या टीम का ID जाँचें।',

  // Antigravity: the IDE's local server is there, but there is nothing to call.
  'error.not_running.reason.no_port.title': 'लोकल सर्वर अभी तैयार नहीं है',
  'error.not_running.reason.no_port.body':
    'Antigravity खुला है, पर उसका लोकल सर्वर अभी कोई पता नहीं दे रहा। यह आमतौर पर थोड़ी देर में ठीक हो जाता है: फिर कोशिश करें।',
  'error.unauthorized.reason.no_token.title': 'लोकल टोकन उपलब्ध नहीं',
  'error.unauthorized.reason.no_token.body':
    'मॉनिटर IDE के लोकल सर्वर का सत्र टोकन नहीं पढ़ सका। Antigravity रीस्टार्ट करें और फिर कोशिश करें।',

  'error.timeout.reason.locked.body':
    'Cursor का लोकल डेटाबेस अभी उपयोग में है। थोड़ी देर बाद फिर कोशिश करें।',
  'error.timeout.reason.timeout.body':
    'Cursor का लोकल डेटाबेस पढ़ने में बहुत देर लगी। थोड़ी देर बाद फिर कोशिश करें।',
  'error.endpoint_changed.reason.schema.body':
    'Cursor के लोकल डेटाबेस का फ़ॉर्मैट वह नहीं है जो मॉनिटर अपेक्षित करता है। मॉनिटर को अपडेट की ज़रूरत हो सकती है।',
  'error.settings_unreadable.reason.permission.body':
    'Cursor का लोकल डेटाबेस खोलने की अनुमति नहीं है। फ़ाइल की अनुमतियाँ जाँचें।',
  'error.settings_unreadable.reason.not_a_database.body': 'मिली हुई Cursor फ़ाइल डेटाबेस नहीं है।',
  'error.settings_unreadable.reason.other.body': 'Cursor का लोकल डेटाबेस पढ़ा नहीं जा सका।',
  'error.not_signed_in.reason.not_found.body':
    'Copilot एक्सटेंशन के फ़ोल्डर में कोई GitHub टोकन नहीं मिला। अपने एडिटर में Copilot में साइन इन करें, या सेटिंग में टोकन चिपकाएँ।',
  'error.not_signed_in.reason.no_token.body':
    'Copilot एक्सटेंशन की फ़ाइलों में github.com के लिए कोई टोकन नहीं है। अपने एडिटर में Copilot में साइन इन करें, या सेटिंग में टोकन चिपकाएँ।',
  'error.not_signed_in.reason.unreadable.body':
    'Copilot एक्सटेंशन की साइन-इन फ़ाइल पढ़ी नहीं जा सकी। उसके फ़ोल्डर की अनुमतियाँ जाँचें।',

  'error.redirect':
    'सर्वर ने अनुरोध को रीडायरेक्ट करने की कोशिश की, और मॉनिटर रीडायरेक्ट का पालन नहीं करता।',
  'error.retry_after': 'स्रोत ने रुकने को कहा है: {wait}।',
  'error.field_hint': 'समस्या वाला फ़ील्ड: {field}।',
  'field.value': 'API कुंजी',
  'field.baseUrl': 'सर्वर URL',
  'field.entityId': 'उपयोगकर्ता या टीम ID',
  'field.authHeader': 'ऑथेंटिकेशन हेडर',
  'field.pemPath': 'CA फ़ाइल (.pem)',
  'field.host': 'प्रमाणपत्र सत्यापन के बिना सर्वर',
  'field.configDir': 'Claude Code कॉन्फ़िगरेशन फ़ोल्डर',
  'field.binaryPath': 'Codex एक्ज़ीक्यूटेबल का पथ',
  'field.dataDir': 'OpenCode डेटा फ़ोल्डर',
  'field.shimConsent': 'कमांड इंस्टॉल करने की सहमति',
  'field.limit': 'पिछली विंडो की संख्या',
} as const;
