// Quota history: the "History" section of a panel and its switch in the settings.
export const historyMessages = {
  'history.title': 'Historique',
  'history.window': 'Fenêtre',
  'history.copy_csv': 'Copier les relevés en CSV',
  'history.copied': 'Relevés copiés au format CSV.',
  'history.copy_failed': 'Impossible de copier les relevés.',
  'history.loading': 'Chargement de l’historique',
  'history.failed': 'Impossible de charger l’historique pour le moment.',
  'history.empty':
    'Aucun historique pour l’instant. Les relevés apparaissent ici au fil de la surveillance.',
  'history.chart_title': 'Utilisation de {title} sur la fenêtre',
  'history.chart_desc':
    'Horizontal : du début de la fenêtre à la réinitialisation. Vertical : utilisation de 0 à 100 %. La ligne épaisse est la fenêtre en cours, les lignes pâles sont les fenêtres précédentes.',
  'history.axis_start': 'début',
  'history.axis_end': 'réinit.',
  'history.label_current': 'actuel {pct}',
  'history.label_past': 'précédentes',
  'history.peak_now': 'Pic de {pct} {when} dans cette fenêtre.',
  'history.no_current': 'Aucun relevé dans cette fenêtre pour l’instant.',
  'history.past_one': 'Fenêtre précédente : pic de {peaks}.',
  'history.past_many': '{n} fenêtres précédentes : pics de {peaks}.',
  'history.table_toggle': 'Afficher sous forme de tableau',
  'history.table_caption': 'Pics d’utilisation par fenêtre',
  'history.col_reset': 'Réinitialisation',
  'history.col_peak': 'Pic',
  'history.col_readings': 'Relevés',
  'history.row_current': '{when} (en cours)',

  'settings.history': 'Historique d’utilisation',
  'settings.history_enabled': 'Conserver l’historique d’utilisation',
  'settings.history_hint':
    'Enregistre uniquement les pourcentages et les heures de chaque fenêtre sur cet ordinateur, pendant 35 jours. Rien ne quitte votre machine. Désactiver cette option arrête l’enregistrement ; utilisez le bouton ci-dessous pour effacer.',
  'settings.history_clear': 'Effacer l’historique',
  'settings.history_clear_confirm':
    'Effacer tout l’historique enregistré ? Cette action est irréversible.',
  'settings.history_clear_yes': 'Effacer',
  'settings.history_clear_no': 'Annuler',
  'settings.history_cleared': 'Historique effacé.',
} as const satisfies Record<string, string>;
