// Notice codes from docs/ipc.md. An unknown code renders `notice.unknown` with the code in it,
// so a missing translation is visible instead of silent.
export const noticeMessages = {
  'notice.code.claude_waiting_first_session':
    'Команда установлена. Лимиты появятся после первого ответа в сеансе Claude Code.',
  'notice.code.claude_no_rate_limits':
    'Claude Code не сообщает лимиты (оплата по ключу API или тариф без лимитов).',
  'notice.code.claude_shim_outdated':
    'Команда, установленная в строке состояния Claude Code, не найдена. Установите её заново, чтобы возобновить чтение.',
  'notice.code.claude_oauth_partial':
    'Расход по моделям (claude /usage или API использования аккаунта) включён, но получить его не удалось ({reason}). Показаны только данные строки состояния.',
  'notice.code.codex_rollout_fallback':
    'Сервер приложения codex не ответил ({reason}). Читается последний записанный сеанс; данные могут быть устаревшими.',
  'notice.code.codex_no_limits': 'Аккаунт Codex не сообщает лимиты (вход по ключу API).',
  'notice.code.antigravity_pool_unknown':
    'Квота {pool} не вернула данных. Она показана как неизвестная, а не как нулевая.',
  'notice.code.litellm_no_budget': 'У этого ключа, пользователя или команды нет лимита бюджета.',
  'notice.code.litellm_level_unreadable':
    'Не удалось прочитать бюджет уровня «{level}» ({reason}). Там может быть лимит, который здесь не виден.',
  'notice.code.litellm_windows_truncated':
    'Окон бюджета больше, чем помещается здесь. Наименее используемые скрыты: {dropped}.',
  'notice.code.opencode_window_idle':
    'Окно «{window}» на 0%: отсчёт начнётся со следующего запроса.',
  'notice.code.opencode_rate_limited':
    'Для окна «{window}» действует ограничение частоты запросов.',
  'notice.code.opencode_no_key':
    'Показана только локальная оценка. Вставьте ключ API Go, чтобы увидеть реальные окна.',
  'notice.code.opencode_local_estimate':
    'Локальные значения — оценка по прайс-листу, только по этому ПК. Это не квота.',
  'notice.code.opencode_db_unreadable':
    'Не удалось прочитать локальную базу данных OpenCode: {reason}.',
  'notice.unknown': 'Сообщение без перевода: {code}',
  'notice.action.reinstall': 'Установить заново',
  'notice.action.settings': 'Ввести ключ',
  'notice.param.window.rolling': '5-часовое',
  'notice.param.window.weekly': 'недельное',
  'notice.param.window.monthly': 'месячное',
  'notice.param.level.key': 'ключ',
  'notice.param.level.user': 'пользователь',
  'notice.param.level.team': 'команда',
  'notice.param.dropped.one': '{n} окно',
  'notice.param.dropped.few': '{n} окна',
  'notice.param.dropped.many': '{n} окон',
  'notice.param.dropped.other': 'окон: {n}',
  // The OpenCode database reasons are their own vocabulary: `not_found` there is a missing file.
  'notice.reason.opencode_db_unreadable.invalid_path': 'недопустимый путь к папке с данными',
  'notice.reason.opencode_db_unreadable.not_found': 'файл базы данных не найден',
  'notice.reason.opencode_db_unreadable.permission': 'нет прав на открытие файла базы данных',
  'notice.reason.opencode_db_unreadable.locked': 'база данных занята другой программой',
  'notice.reason.opencode_db_unreadable.not_a_database': 'найденный файл не является базой данных',
  'notice.reason.opencode_db_unreadable.schema': 'формат базы данных изменился',
  'notice.reason.opencode_db_unreadable.timeout': 'чтение базы данных заняло слишком много времени',
  'notice.reason.opencode_db_unreadable.other': 'причина не установлена',
  'notice.severity.info': 'Информация',
  'notice.severity.warn': 'Предупреждение',
  'notice.severity.error': 'Ошибка',
} as const satisfies Record<string, string>;
