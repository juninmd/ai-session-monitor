// What the params of a failure add to its message: the `reason` a collector gives (closed
// vocabularies, worded per error code), the sentences for a redirect and a wait, and the names
// of the settings fields. A reason with a `.title` replaces the title of the code as well.
export const errorDetailMessages = {
  'error.install_failed.reason.status_line_too_large.body':
    'Текущая команда строки состояния слишком велика, монитор не может её объединить со своей. Ничего не изменено.',
  'error.install_failed.reason.path_unsupported.body':
    'В пути к папке с данными есть символы, которые команда строки состояния не принимает. Ничего не изменено.',
  'error.install_failed.reason.serialize_failed.body':
    'Не удалось сформировать новый settings.json. Ничего не изменено.',
  'error.install_failed.reason.state_failed.body':
    'Не удалось сохранить состояние установки. settings.json не изменён.',
  'error.install_failed.reason.concurrent_modification.body':
    'settings.json изменился во время операции, поэтому монитор его не тронул. Повторите попытку.',
  'error.install_failed.reason.write_failed.body':
    'Не удалось записать settings.json. Файл не изменён; проверьте, что он не заблокирован, и повторите попытку.',
  'error.install_failed.reason.shim_missing.body':
    'Исполняемый файл команды не найден рядом с монитором. Переустановите монитор.',
  'error.install_failed.reason.copy_failed.body':
    'Не удалось скопировать исполняемый файл команды в папку с данными. settings.json не изменён.',
  'error.install_failed.reason.backup_failed.body':
    'Не удалось сохранить резервную копию settings.json, поэтому файл не изменён.',
  // Only a removal is refused for this reason.
  'error.install_failed.reason.foreign_status_line.title': 'Не удалось удалить команду',
  'error.install_failed.reason.foreign_status_line.body':
    'Настроенная строка состояния не принадлежит монитору (возможно, это копия, изменённая вручную), поэтому ничего не удалено.',

  // LiteLLM: the CA file the user chose, not the certificate of the server.
  'error.tls_error.reason.ca_missing.body':
    'Указанный файл центра сертификации не найден. Выберите правильный файл в настройках.',
  'error.tls_error.reason.ca_too_large.body':
    'Файл центра сертификации слишком велик для чтения. Выберите файл .pem поменьше в настройках.',
  'error.tls_error.reason.ca_unreadable.body':
    'Не удалось прочитать файл центра сертификации. Проверьте путь и права доступа в настройках.',
  'error.tls_error.reason.ca_invalid.body':
    'В файле центра сертификации нет действительного сертификата. Выберите корректный файл .pem в настройках.',

  // LiteLLM: which part of the answer did not look like a key, user or team record.
  'error.endpoint_changed.reason.not_an_object.body':
    'Ответ сервера имеет неожиданный формат. Убедитесь, что URL указывает на LiteLLM.',
  'error.endpoint_changed.reason.no_record.body':
    'В ответе нет записи о ключе, пользователе или команде. Проверьте URL и область действия.',
  'error.endpoint_changed.reason.id_missing.body':
    'Ответ не подтверждает запрошенный ID. Проверьте ID пользователя или команды.',
  'error.endpoint_changed.reason.id_mismatch.body':
    'Ответ относится к другому ID, а не к указанному. Проверьте ID пользователя или команды.',

  // Antigravity: the IDE's local server is there, but there is nothing to call.
  'error.not_running.reason.no_port.title': 'Локальный сервер ещё не готов',
  'error.not_running.reason.no_port.body':
    'Antigravity открыт, но его локальный сервер ещё не предоставил адрес. Обычно это быстро проходит: повторите попытку.',
  'error.unauthorized.reason.no_token.title': 'Локальный токен недоступен',
  'error.unauthorized.reason.no_token.body':
    'Монитор не смог прочитать токен сеанса локального сервера IDE. Перезапустите Antigravity и повторите попытку.',

  'error.redirect': 'Сервер попытался перенаправить запрос, а монитор не следует перенаправлениям.',
  'error.retry_after': 'Источник просит подождать: {wait}.',
  'error.field_hint': 'Поле с ошибкой: {field}.',
  'field.value': 'ключ API',
  'field.baseUrl': 'URL сервера',
  'field.entityId': 'ID пользователя или команды',
  'field.authHeader': 'заголовок аутентификации',
  'field.pemPath': 'файл центра сертификации (.pem)',
  'field.host': 'сервер без проверки сертификата',
  'field.configDir': 'папка конфигурации Claude Code',
  'field.binaryPath': 'путь к исполняемому файлу Codex',
  'field.dataDir': 'папка данных OpenCode',
  'field.shimConsent': 'согласие на установку команды',
  'field.limit': 'число прошлых окон',
  'error.timeout.reason.locked.body':
    'Локальная база данных Cursor сейчас занята. Повторите попытку через минуту.',
  'error.timeout.reason.timeout.body':
    'Чтение локальной базы данных Cursor заняло слишком много времени. Повторите попытку через минуту.',
  'error.endpoint_changed.reason.schema.body':
    'Структура локальной базы данных Cursor отличается от ожидаемой монитором. Возможно, нужно обновить монитор.',
  'error.settings_unreadable.reason.permission.body':
    'Нет прав на открытие локальной базы данных Cursor. Проверьте права доступа к файлу.',
  'error.settings_unreadable.reason.not_a_database.body':
    'Найденный файл Cursor не является базой данных.',
  'error.settings_unreadable.reason.other.body':
    'Не удалось прочитать локальную базу данных Cursor.',
  'error.not_signed_in.reason.not_found.body':
    'В папке расширения Copilot не найден токен GitHub. Войдите в Copilot в редакторе или вставьте токен в настройках.',
  'error.not_signed_in.reason.no_token.body':
    'В файлах расширения Copilot нет токена для github.com. Войдите в Copilot в редакторе или вставьте токен в настройках.',
  'error.not_signed_in.reason.unreadable.body':
    'Не удалось прочитать файл входа расширения Copilot. Проверьте права доступа к его папке.',
} as const satisfies Record<string, string>;
