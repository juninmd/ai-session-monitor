// hi catalog: setup. Same keys as the English file; plural keys use this language's forms.
// First-run connection surfaces: consent cards, secret fields, OpenCode and LiteLLM forms.
export const setupMessages = {
  'consent.skip': 'अभी नहीं',
  'consent.details': 'क्या पढ़ा जाएगा',

  'claude.connect.title': 'Claude Code से कनेक्ट करें',
  'claude.connect.lead':
    'मॉनिटर Claude Code की स्टेटस लाइन में एक छोटी कमांड जोड़ता है। यह एक लोकल फ़ाइल में केवल सीमाओं का प्रतिशत (5 घंटे और सप्ताह) रखती है। कोई लॉगिन नहीं पढ़ा जाता।',
  'claude.connect.detail':
    'बैकअप के साथ Claude Code की settings.json बदलता है और आपकी मौजूदा स्टेटस लाइन को साथ जोड़े रखता है। केवल सीमाएँ %LOCALAPPDATA% में लिखता है। न नेटवर्क, न टोकन। Pro और Max खातों पर काम करता है। आँकड़े सत्र में पहले जवाब के बाद आते हैं और केवल तब अपडेट होते हैं जब Claude Code खुला हो।',
  'claude.connect.allow': 'कमांड इंस्टॉल करें',
  'claude.connect.tip':
    'सुझाव: Windows नए आइकन ओवरफ़्लो ऐरो मेन्यू में छिपा देता है। मॉनिटर आइकन को हमेशा दिखाने के लिए उसे टास्कबार पर खींचकर रखें।',

  'codex.connect.title': 'Codex से कनेक्ट करें',
  'codex.connect.lead':
    'मॉनिटर यहाँ इंस्टॉल किया गया codex app-server चलाता है और पूछता है कि 5 घंटे की विंडो और सप्ताह में कितना बचा है।',
  'codex.connect.detail':
    'आपकी मशीन पर codex प्रोग्राम केवल पढ़ने के मोड में चलता है। कोई डेटा इस कंप्यूटर से बाहर नहीं जाता और ऐप Codex की लॉगिन फ़ाइल को नहीं छूता। OpenAI app-server को प्रायोगिक बताता है, इसलिए संस्करणों के बीच फ़ॉर्मैट बदल सकता है।',
  'codex.connect.allow': 'पढ़ने की अनुमति दें',

  'antigravity.connect.title': 'Antigravity से कनेक्ट करें',
  'antigravity.connect.lead':
    'कोटा इंस्टॉल किए गए agy (CLI) या IDE के लोकल सर्वर से आता है। यह रास्ता अनौपचारिक है और Antigravity के अपडेट होने पर काम करना बंद कर सकता है।',
  'antigravity.connect.detail':
    'आपकी अनुमति से मॉनिटर इंस्टॉल किया गया agy केवल /usage पढ़ने के लिए चला सकता है (टोकन खर्च किए बिना और लॉगिन डेटा पढ़े बिना) और/या लोकल सत्र टोकन पाने के लिए IDE सर्वर की कमांड लाइन पढ़ सकता है, जो केवल 127.0.0.1 से बात करता है। डिस्क पर कुछ नहीं लिखा जाता और इंटरनेट पर कुछ नहीं भेजा जाता।',
  'antigravity.connect.allow': 'लोकल रीडिंग की अनुमति दें',
  'cursor.connect.title': 'Cursor से कनेक्ट करें',
  'cursor.connect.lead':
    'आपकी अनुमति से मॉनिटर Cursor के लोकल डेटाबेस में रखा साइन-इन टोकन पढ़ता है और उससे cursor.com से पूछता है कि आपके प्लान का कितना उपयोग हुआ है। यह रास्ता अनौपचारिक है और Cursor में बदलाव होने पर काम करना बंद कर सकता है।',
  'cursor.connect.detail':
    'Cursor का लोकल डेटाबेस (state.vscdb) केवल पढ़ने के मोड में खोलता है और एक ही एंट्री पढ़ता है: साइन-इन टोकन। टोकन केवल cursor.com को भेजा जाता है, ठीक वैसे ही जैसे Cursor डैशबोर्ड करता है, और उसे कभी सहेजा या दिखाया नहीं जाता। कोई दूसरी एंट्री (चैट, फ़ाइलें, ई-मेल) नहीं पढ़ी जाती और कुछ लिखा नहीं जाता। Cursor प्लान के उपयोग के लिए कोई API प्रकाशित नहीं करता, इसलिए पहली बार आँकड़ों की तुलना डैशबोर्ड से करें।',
  'cursor.connect.allow': 'पढ़ने की अनुमति दें',
  'cursor.signin.title': 'Cursor में साइन इन करें',
  'cursor.signin.body':
    'मॉनिटर को कोई वैध साइन-इन टोकन नहीं मिला। Cursor खोलें और साइन इन करें (खोलने से समाप्त हो चुका टोकन भी नवीनीकृत हो जाता है), फिर दोबारा पढ़ें।',
  'cursor.dashboard': 'Cursor डैशबोर्ड खोलें',
  'cursor.note':
    'Cursor के उपयोग-सारांश की अनौपचारिक रीडिंग, उस साइन-इन टोकन से जो Cursor इस PC पर रखता है। इसकी तुलना डैशबोर्ड से करें। शामिल उपयोग केवल प्रतिशत के रूप में दिखता है; असीमित प्लान में कोई विंडो नहीं होती।',

  'shim.title': 'स्टेटस लाइन कमांड',
  'shim.state.not_installed': 'इंस्टॉल नहीं है',
  'shim.state.installed': 'इंस्टॉल है',
  'shim.state.installed_chained': 'इंस्टॉल है, आपकी स्टेटस लाइन के साथ जुड़ी',
  'shim.state.foreign': 'दूसरी स्टेटस लाइन उपयोग में है',
  'shim.state.outdated': 'पुराना: एक्ज़ीक्यूटेबल नहीं मिला',
  'shim.state.unreadable': 'settings.json पढ़ी नहीं जा सकती',
  'shim.hint.foreign':
    'इंस्टॉल करने पर आपकी मौजूदा स्टेटस लाइन काम करती रहती है: मॉनिटर उसे साथ जोड़ लेता है।',
  'shim.hint.unreadable':
    'जो फ़ाइल पढ़ी नहीं जा सकती, मॉनिटर उसे नहीं छूता। settings.json ठीक करें और फिर जाँचें।',
  'shim.path': 'फ़ाइल: {path}',
  'shim.install': 'कमांड इंस्टॉल करें',
  'shim.reinstall': 'कमांड दोबारा इंस्टॉल करें',
  'shim.uninstall': 'कमांड हटाएँ',
  'shim.recheck': 'फिर जाँचें',
  'shim.loading': 'Claude Code जाँचा जा रहा है',

  'secret.saved': 'कुंजी सहेजी गई',
  'secret.not_saved': 'कोई कुंजी सहेजी नहीं है',
  'secret.placeholder': 'कुंजी पेस्ट करें',
  'secret.placeholder_keep': 'सहेजी गई कुंजी रखने के लिए खाली छोड़ें',
  'secret.hint': 'Windows Credential Manager में रहती है और इंटरफ़ेस में कभी नहीं दिखती।',
  'secret.save': 'कुंजी सहेजें',
  'secret.clear': 'कुंजी हटाएँ',
  'secret.empty': 'API कुंजी पेस्ट करें।',
  'secret.saved_ok': 'कुंजी सहेज ली गई।',
  'secret.cleared_ok': 'कुंजी हटा दी गई।',

  'opencode.title': 'OpenCode Go से कनेक्ट करें',
  'opencode.lead':
    'असली विंडो (5 घंटे, सप्ताह और माह) पढ़ने के लिए अपनी OpenCode Go API कुंजी पेस्ट करें। कुंजी के बिना केवल स्थानीय उपयोग का अनुमान दिखता है।',
  'opencode.key_label': 'OpenCode Go API कुंजी',
  'opencode.console': 'OpenCode Go कंसोल खोलें',
  'opencode.billing': 'Zen बिलिंग देखें',
  'opencode.zen_note': 'Zen (प्रीपेड) बैलेंस के लिए कोई API नहीं है: बिलिंग डैशबोर्ड देखें।',
  'opencode.local_label': 'स्थानीय अनुमान (केवल यह PC)',
  'opencode.local_hint':
    'OpenCode डेटाबेस को केवल पढ़ने के मोड में खोलता है और लिस्ट प्राइस पर सिर्फ़ लागत और टोकन जोड़ता है। यह असली कोटा नहीं है।',
  'opencode.data_dir': 'OpenCode डेटा फ़ोल्डर',
  'opencode.data_dir_hint': 'डिफ़ॉल्ट फ़ोल्डर के लिए खाली छोड़ें।',
  'opencode.save_settings': 'फ़ोल्डर सहेजें',

  'copilot.title': 'GitHub Copilot से कनेक्ट करें',
  'copilot.lead':
    'इस माह इस्तेमाल हुए प्रीमियम अनुरोध देखने के लिए GitHub टोकन चिपकाएँ, या मॉनिटर को वह टोकन पढ़ने दें जो Copilot एक्सटेंशन डिस्क पर रखता है। यह रास्ता अनौपचारिक है।',
  'copilot.detail':
    'अनौपचारिक रीडिंग: यह GitHub के एक आंतरिक एंडपॉइंट का उपयोग करती है, जिसे Copilot एडिटर अपना कोटा दिखाने के लिए कॉल करते हैं। वह प्रलेखित नहीं है और बिना सूचना के बदल सकता है। चिपकाया गया टोकन Windows Credential Manager में रहता है। आपकी अनुमति से मॉनिटर इसके बजाय वह GitHub टोकन पढ़ता है जो Copilot एक्सटेंशन डिस्क पर रखता है (उसके अपने फ़ोल्डर की दो निश्चित फ़ाइलें, केवल पढ़ने के मोड में)। दोनों में से कोई भी टोकन केवल api.github.com को भेजा जाता है। GitHub Enterprise के टोकन उपयोग नहीं होते। पहली बार आँकड़ों की तुलना github.com/settings/copilot से करें।',
  'copilot.key_label': 'GitHub टोकन (वैकल्पिक)',
  'copilot.console': 'GitHub Copilot सेटिंग खोलें',
  'copilot.local_label': 'Copilot एक्सटेंशन का टोकन पढ़ें (केवल यह PC)',
  'copilot.local_hint':
    'अनौपचारिक: Copilot एक्सटेंशन डिस्क पर जो GitHub टोकन रखता है उसे पढ़ता है (दो निश्चित फ़ाइलें, केवल पढ़ने के मोड में) और केवल api.github.com को भेजता है। टोकन चिपकाने पर इसकी ज़रूरत नहीं।',
  'copilot.note':
    'GitHub Copilot कोटा की अनौपचारिक रीडिंग। रीसेट की तिथि अनुमान हो सकती है (माह की 1 तारीख़, UTC)। असीमित कोटा प्रतिशत के रूप में नहीं, विवरण के रूप में सूचीबद्ध होते हैं।',

  'kimi.title': 'Kimi Code से कनेक्ट करें',
  'kimi.lead':
    'प्लान का कोटा पढ़ने के लिए अपनी Kimi Code API कुंजी पेस्ट करें: 5 घंटे की विंडो, सप्ताह और माह का कुल, जब प्लान उन्हें बताता है।',
  'kimi.detail':
    'अनौपचारिक रीडिंग: उपयोग API प्रलेखित नहीं है और बिना सूचना के बदल सकती है। कुंजी केवल api.kimi.com को भेजी जाती है और Windows Credential Manager में रहती है। किसी दूसरे टूल की कुकी या लॉगिन का उपयोग नहीं होता। Moonshot के ओपन प्लैटफ़ॉर्म की कुंजी एक अलग प्रोडक्ट की है और यहाँ काम नहीं करती।',
  'kimi.key_label': 'Kimi Code API कुंजी',
  'kimi.console': 'Kimi Code कंसोल खोलें',
  'kimi.note':
    'Kimi Code उपयोग API की अनौपचारिक रीडिंग। सप्ताह और माह तभी दिखते हैं जब प्लान उन्हें बताता है।',

  'zai.title': 'z.ai से कनेक्ट करें',
  'zai.lead':
    '5 घंटे की विंडो, सप्ताह और मासिक MCP टूल कोटा पढ़ने के लिए अपने GLM Coding प्लान की API कुंजी पेस्ट करें।',
  'zai.detail':
    'अनौपचारिक रीडिंग: कोटा API प्रलेखित नहीं है और बिना सूचना के बदल सकती है। कुंजी केवल api.z.ai को भेजी जाती है और Windows Credential Manager में रहती है। किसी दूसरे टूल की कुकी या लॉगिन का उपयोग नहीं होता। BigModel खाते (open.bigmodel.cn) नहीं पढ़े जाते।',
  'zai.key_label': 'z.ai API कुंजी',
  'zai.console': 'z.ai कुंजियाँ खोलें',
  'zai.note': 'ग्लोबल सर्वर (api.z.ai) पर GLM Coding प्लान की कोटा API की अनौपचारिक रीडिंग।',

  'openrouter.title': 'OpenRouter से कनेक्ट करें',
  'openrouter.lead':
    'कुंजी की खर्च सीमा, उपयोग और, जब कुंजी अनुमति दे, क्रेडिट बैलेंस देखने के लिए अपनी OpenRouter API कुंजी पेस्ट करें।',
  'openrouter.detail':
    'आधिकारिक रीडिंग, OpenRouter की प्रलेखित API से। कुंजी केवल openrouter.ai को भेजी जाती है और Windows Credential Manager में रहती है। क्रेडिट बैलेंस के लिए मैनेजमेंट कुंजी चाहिए; सामान्य कुंजी से केवल कुंजी की सीमा और उपयोग दिखते हैं।',
  'openrouter.key_label': 'OpenRouter API कुंजी',
  'openrouter.console': 'OpenRouter कुंजियाँ खोलें',
  'openrouter.note':
    'OpenRouter API की आधिकारिक रीडिंग। कुंजी पर सीमा न हो तो प्रतिशत नहीं होता, केवल उपयोग दिखता है। बैलेंस केवल मैनेजमेंट कुंजी से दिखता है।',

  'deepseek.title': 'DeepSeek से कनेक्ट करें',
  'deepseek.lead': 'खाते का बैलेंस, हर उस मुद्रा में देखने के लिए जिसमें क्रेडिट है, अपनी DeepSeek API कुंजी पेस्ट करें।',
  'deepseek.detail':
    'आधिकारिक रीडिंग, DeepSeek की प्रलेखित API से। कुंजी केवल api.deepseek.com को भेजी जाती है और Windows Credential Manager में रहती है। बैलेंस की कोई सीमा या रीसेट समय नहीं होता, इसलिए प्रतिशत नहीं है; युआन और डॉलर कभी जोड़े नहीं जाते।',
  'deepseek.key_label': 'DeepSeek API कुंजी',
  'deepseek.console': 'DeepSeek कुंजियाँ खोलें',
  'deepseek.note': 'DeepSeek बैलेंस API की आधिकारिक रीडिंग। यह पैसा है, कोटा नहीं: न प्रतिशत है, न रीसेट।',

  'deepinfra.title': 'DeepInfra से कनेक्ट करें',
  'deepinfra.lead': 'बैलेंस, खर्च की सीमा और इस माह का खर्च देखने के लिए अपनी DeepInfra API कुंजी पेस्ट करें।',
  'deepinfra.detail':
    'आधिकारिक रीडिंग, DeepInfra की बिलिंग API से। कुंजी केवल api.deepinfra.com को भेजी जाती है और Windows Credential Manager में रहती है। खर्च की सीमा न हो तो प्रतिशत नहीं होता, केवल डॉलर की राशियाँ दिखती हैं।',
  'deepinfra.key_label': 'DeepInfra API कुंजी',
  'deepinfra.console': 'DeepInfra कुंजियाँ खोलें',
  'deepinfra.note':
    'DeepInfra बिलिंग API की आधिकारिक रीडिंग। मान डॉलर में पढ़े जाते हैं; खर्च की सीमा न हो तो प्रतिशत नहीं होता।',

  'moonshot.title': 'Moonshot से कनेक्ट करें',
  'moonshot.lead':
    'खाते का बैलेंस देखने के लिए अपने Moonshot प्लैटफ़ॉर्म (platform.moonshot.ai) की API कुंजी पेस्ट करें।',
  'moonshot.detail':
    'आधिकारिक रीडिंग, Moonshot की प्रलेखित बैलेंस API से। कुंजी केवल api.moonshot.ai को भेजी जाती है और Windows Credential Manager में रहती है। kimi.com प्लैटफ़ॉर्म की कुंजियाँ अलग हैं और यहाँ काम नहीं करतीं। बैलेंस पैसा है: न प्रतिशत है, न रीसेट।',
  'moonshot.key_label': 'Moonshot API कुंजी',
  'moonshot.console': 'Moonshot कुंजियाँ खोलें',
  'moonshot.note':
    'Moonshot बैलेंस API की आधिकारिक रीडिंग, डॉलर में। कुछ राशि बकाया हो तो नकद बैलेंस ऋणात्मक हो सकता है।',

  'novita.title': 'Novita से कनेक्ट करें',
  'novita.lead': 'खाते का बैलेंस, क्रेडिट सीमा और लंबित शुल्क देखने के लिए अपनी Novita API कुंजी पेस्ट करें।',
  'novita.detail':
    'आधिकारिक रीडिंग, Novita की प्रलेखित बिलिंग API से। कुंजी केवल api.novita.ai को भेजी जाती है और Windows Credential Manager में रहती है। बैलेंस पैसा है: न प्रतिशत है, न रीसेट।',
  'novita.key_label': 'Novita API कुंजी',
  'novita.console': 'Novita कुंजियाँ खोलें',
  'novita.note':
    'Novita बैलेंस API की आधिकारिक रीडिंग, डॉलर में। कुछ राशि बकाया हो तो बैलेंस ऋणात्मक हो सकता है।',

  'ollama.title': 'Ollama Cloud से कनेक्ट करें',
  'ollama.lead': 'प्लान के शामिल क्रेडिट और खरीदा हुआ बैलेंस देखने के लिए Ollama Cloud की API कुंजी पेस्ट करें।',
  'ollama.detail':
    'आधिकारिक रीडिंग, Ollama Cloud की प्रलेखित बैलेंस API से। कुंजी ollama.com/settings/keys पर बनाएँ: CLI की सब्सक्रिप्शन कुंजी कभी नहीं पढ़ी जाती। कुंजी केवल ollama.com को भेजी जाती है और Windows Credential Manager में रहती है। जवाब का फ़ॉर्मैट प्लान पर निर्भर करता है (मासिक क्रेडिट या सत्र और सप्ताह की विंडो)।',
  'ollama.key_label': 'Ollama Cloud API कुंजी',
  'ollama.console': 'Ollama Cloud कुंजियाँ खोलें',
  'ollama.note':
    'Ollama Cloud बैलेंस API की आधिकारिक रीडिंग। मौजूदा प्लान डॉलर क्रेडिट दिखाते हैं; पुराने प्लान सत्र और सप्ताह का प्रतिशत दिखाते हैं।',

  'minimax.title': 'MiniMax से कनेक्ट करें',
  'minimax.lead':
    'हर मॉडल का कोटा, यानी मौजूदा अंतराल और सप्ताह, देखने के लिए अपनी MiniMax Token Plan कुंजी पेस्ट करें।',
  'minimax.detail':
    'अनौपचारिक रीडिंग: Token Plan कोटा रूट का कोई प्रकाशित स्कीमा नहीं है और वह बिना सूचना के बदल सकता है। कुंजी केवल api.minimax.io (ग्लोबल सर्वर) को भेजी जाती है और Windows Credential Manager में रहती है। पहली बार आँकड़ों की तुलना MiniMax कंसोल से कर लें।',
  'minimax.key_label': 'MiniMax Token Plan कुंजी',
  'minimax.console': 'MiniMax कुंजियाँ खोलें',
  'minimax.note':
    'MiniMax Token Plan कोटा की, हर मॉडल के अनुसार, अनौपचारिक रीडिंग। प्लान में जिस मॉडल का कोटा नहीं है, वह नहीं दिखता।',

  'litellm.title': 'LiteLLM से कनेक्ट करें',
  'litellm.title_edit': 'LiteLLM कनेक्शन',
  'litellm.url': 'सर्वर URL',
  'litellm.url_placeholder': 'https://litellm.yourcompany.com',
  'litellm.url_hint': 'HTTPS ज़रूरी है। HTTP केवल localhost के लिए स्वीकार होता है।',
  'litellm.url_required': 'सर्वर URL दर्ज करें।',
  'litellm.url_invalid': 'पूरा URL दर्ज करें, जैसे https://litellm.yourcompany.com।',
  'litellm.url_http': 'https:// का उपयोग करें। HTTP केवल localhost के लिए चलता है।',
  'litellm.url_parts': 'URL से उपयोगकर्ता नाम, पासवर्ड, क्वेरी पैरामीटर और # हटाएँ।',
  'litellm.key': 'API कुंजी',
  'litellm.key_required': 'LiteLLM API कुंजी पेस्ट करें।',
  'litellm.key_rebind':
    'सहेजी गई कुंजी केवल मौजूदा कनेक्शन पर लागू होती है। कनेक्शन बदलने के कारण कुंजी दोबारा पेस्ट करें।',
  'litellm.scope': 'क्या मापना है',
  'litellm.scope.key': 'यह कुंजी',
  'litellm.scope.user': 'उपयोगकर्ता',
  'litellm.scope.team': 'टीम',
  'litellm.entity.user': 'उपयोगकर्ता ID',
  'litellm.entity.team': 'टीम ID',
  'litellm.entity_required': 'ID दर्ज करें।',
  'litellm.advanced': 'उन्नत',
  'litellm.auth_header': 'ऑथेंटिकेशन हेडर',
  'litellm.auth_header_hint': 'केवल तब बदलें जब आपका प्रॉक्सी कुंजी के लिए अपना हेडर उपयोग करता हो।',
  'litellm.auth_header_invalid': 'केवल अक्षर, अंक और हाइफ़न का उपयोग करें।',
  'litellm.poll': 'रीफ़्रेश अंतराल',
  'litellm.timeout': 'टाइमआउट',
  'litellm.tls': 'TLS प्रमाणपत्र',
  'litellm.tls.system': 'Windows से',
  'litellm.tls.custom_ca': 'कस्टम CA',
  'litellm.tls.insecure': 'सत्यापित न करें',
  'litellm.pem': 'CA फ़ाइल (.pem)',
  'litellm.pem_placeholder': 'C:\\certs\\ca.pem',
  'litellm.pem_required': '.pem फ़ाइल का पथ दर्ज करें।',
  'litellm.insecure_host': 'बिना सत्यापन वाला सर्वर',
  'litellm.insecure_host_hint': 'प्रमाणपत्र केवल इसी सर्वर के लिए छोड़ा जाता है।',
  'litellm.insecure_host_required': 'सर्वर दर्ज करें।',
  'litellm.insecure_warning':
    'कनेक्शन सत्यापित नहीं होगा और कुंजी बीच में पकड़ी जा सकती है। केवल भरोसेमंद नेटवर्क पर उपयोग करें।',
  'litellm.insecure_confirm': 'मुझे जोखिम की जानकारी है; इस सर्वर का प्रमाणपत्र अनदेखा करें।',
  'litellm.insecure_confirm_required': 'प्रमाणपत्र सत्यापित किए बिना आगे बढ़ने के लिए पुष्टि करें।',
  'litellm.save': 'सहेजें और कनेक्ट करें',
  'litellm.saving': 'सहेजा जा रहा है…',
  'litellm.saved': 'सेटिंग सहेज ली गई। सर्वर पढ़ा जा रहा है…',
  'litellm.edit': 'कनेक्शन बदलें',
  'litellm.delay_note':
    'सर्वर पर खर्च और रीसेट में लगभग 10 मिनट तक की देरी हो सकती है। सीमा केवल कुंजी पर नहीं, उपयोगकर्ता या टीम पर भी हो सकती है।',
} as const;
