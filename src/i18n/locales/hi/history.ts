// hi catalog: history. Same keys as the English file; plural keys use this language's forms.
// Quota history: the "History" section of a panel and its switch in the settings.
export const historyMessages = {
  'history.title': 'इतिहास',
  'history.window': 'विंडो',
  'history.copy_csv': 'रीडिंग CSV के रूप में कॉपी करें',
  'history.copied': 'रीडिंग CSV के रूप में कॉपी हो गईं।',
  'history.copy_failed': 'रीडिंग कॉपी नहीं हो सकीं।',
  'history.loading': 'इतिहास लोड हो रहा है',
  'history.failed': 'इतिहास अभी लोड नहीं हो सका।',
  'history.empty': 'अभी कोई इतिहास नहीं है। मॉनिटर चलने के साथ रीडिंग यहाँ दिखेंगी।',
  'history.chart_title': 'विंडो में {title} का उपयोग',
  'history.chart_desc':
    'क्षैतिज: विंडो की शुरुआत से रीसेट तक। ऊर्ध्वाधर: 0 से 100% तक उपयोग। मोटी रेखा मौजूदा विंडो है और हल्की रेखाएँ पिछली विंडो हैं।',
  'history.axis_start': 'शुरू',
  'history.axis_end': 'रीसेट',
  'history.label_current': 'मौजूदा {pct}',
  'history.label_past': 'पिछली',
  'history.peak_now': 'इस विंडो में {when} {pct} का शिखर।',
  'history.no_current': 'इस विंडो में अभी कोई रीडिंग नहीं है।',
  'history.past_one': 'पिछली विंडो: शिखर {peaks}।',
  'history.past_many': 'पिछली {n} विंडो: शिखर {peaks}।',
  'history.table_toggle': 'तालिका के रूप में देखें',
  'history.table_caption': 'विंडो के अनुसार उपयोग के शिखर',
  'history.col_reset': 'रीसेट',
  'history.col_peak': 'शिखर',
  'history.col_readings': 'रीडिंग',
  'history.row_current': '{when} (मौजूदा)',

  'settings.history': 'उपयोग का इतिहास',
  'settings.history_enabled': 'उपयोग का इतिहास रखें',
  'settings.history_hint':
    'इस कंप्यूटर पर हर विंडो के केवल प्रतिशत और समय 35 दिन तक सहेजता है। कुछ भी यहाँ से बाहर नहीं जाता। बंद करने पर रिकॉर्डिंग रुक जाती है; मिटाने के लिए नीचे का बटन उपयोग करें।',
  'settings.history_clear': 'इतिहास साफ़ करें',
  'settings.history_clear_confirm': 'सहेजा गया पूरा इतिहास मिटा दें? इसे वापस नहीं लाया जा सकता।',
  'settings.history_clear_yes': 'मिटाएँ',
  'settings.history_clear_no': 'रद्द करें',
  'settings.history_cleared': 'इतिहास मिटा दिया गया।',
} as const;
