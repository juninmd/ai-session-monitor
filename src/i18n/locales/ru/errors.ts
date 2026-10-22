// One title and one body for every ErrorCode (src/lib/contract/view.ts), plus the OpenCode
// ambiguity wording. src/i18n/catalog.test.ts fails when a code is missing.
export const errorMessages = {
  'error.internal.title': 'Что-то пошло не так',
  'error.internal.body':
    'При чтении этого источника произошла внутренняя ошибка. Повторите попытку; если ошибка повторяется, перезапустите монитор.',
  'error.timeout.title': 'Чтение заняло слишком много времени',
  'error.timeout.body': 'Источник не ответил вовремя. Повторите попытку чуть позже.',
  'error.network.title': 'Источник недоступен',
  'error.network.body': 'Не удалось связаться с сервером. Проверьте сеть и повторите попытку.',
  'error.bad_response.title': 'Неожиданный ответ',
  'error.bad_response.body':
    'Источник ответил так, что монитор не смог это понять. Повторите попытку позже.',
  'error.response_too_large.title': 'Слишком большой ответ',
  'error.response_too_large.body':
    'Ответ превысил допустимый размер и был отброшен в целях безопасности.',
  'error.rate_limited.title': 'Слишком много запросов',
  'error.rate_limited.body':
    'Источник попросил немного подождать. Монитор повторит попытку сам, с увеличенным интервалом.',
  'error.unauthorized.title': 'Ключ или вход отклонён',
  'error.unauthorized.body':
    'Сервер отклонил учётные данные. Проверьте ключ и его права в настройках.',
  'error.unauthorized_ambiguous.title': 'Не удалось проверить ключ или подписку Go',
  'error.unauthorized_ambiguous.body':
    'Сервис отклонил запрос. Так бывает и с действующими ключами, и без подписки Go. Проверьте ключ и подписку в консоли OpenCode.',
  'error.unauthorized_ambiguous.kimi.title': 'Не удалось проверить ключ Kimi Code',
  'error.unauthorized_ambiguous.kimi.body':
    'Сервис отклонил запрос. Так бывает с просроченными ключами, а также с ключом открытой платформы Moonshot — это другой продукт. Убедитесь в консоли Kimi Code, что ключ относится к вашему тарифу.',
  'error.unauthorized_ambiguous.zai.title': 'Не удалось проверить ключ z.ai',
  'error.unauthorized_ambiguous.zai.body':
    'Сервис отклонил запрос. Так бывает и с действующими ключами, и с аккаунтами без тарифа GLM Coding. Проверьте ключ и тариф в панели z.ai.',
  'error.unauthorized_ambiguous.cursor.title': 'Не удалось проверить вход в Cursor',
  'error.unauthorized_ambiguous.cursor.body':
    'Сервис отклонил запрос. Так бывает с просроченным или устаревшим сеансом, а также с аккаунтом без активного тарифа. Откройте Cursor, чтобы обновить вход, и проверьте тариф на панели Cursor.',
  'error.unauthorized_ambiguous.copilot.title': 'Не удалось проверить токен GitHub',
  'error.unauthorized_ambiguous.copilot.body':
    'GitHub отклонил запрос. Так бывает с просроченным токеном, с детализированным токеном или токеном приложения, а также с аккаунтом без Copilot. Проверьте токен и подписку Copilot в настройках GitHub.',
  'error.not_found.title': 'Адрес не найден',
  'error.not_found.body':
    'Сервер не распознаёт указанный путь или ID. Проверьте URL, область действия и ID.',
  'error.server_error.title': 'Ошибка сервера',
  'error.server_error.body': 'Сбой на сервере источника. Повторите попытку позже.',
  'error.tls_error.title': 'Ошибка сертификата TLS',
  'error.tls_error.body':
    'Сертификат сервера не принят. Выберите подходящий центр сертификации в настройках.',
  'error.invalid_url.title': 'Недопустимый URL',
  'error.invalid_url.body':
    'Используйте https://. HTTP работает только для localhost, а URL не должен содержать имя пользователя, пароль, параметры запроса или #.',
  'error.invalid_input.title': 'Недопустимое значение',
  'error.invalid_input.body': 'Исправьте указанное поле и повторите попытку.',
  'error.settings_unreadable.title': 'Файл настроек не читается',
  'error.settings_unreadable.body':
    'Не удалось прочитать settings.json. Ничего не изменено. Исправьте файл и повторите попытку.',
  'error.install_failed.title': 'Не удалось установить',
  'error.install_failed.body':
    'Не удалось установить команду в строку состояния Claude Code. Ничего не изменено.',
  'error.unsupported_plan.title': 'В тарифе нет лимитов для чтения',
  'error.unsupported_plan.body':
    'Аккаунт использует тариф или ключ API, которые не сообщают лимиты. Этот источник можно отключить.',
  'error.oauth_expired.title': 'Срок входа в Claude Code истёк',
  'error.oauth_expired.body': 'Откройте Claude Code, обновите вход и повторите попытку.',
  'error.binary_not_found.title': 'Программа не найдена',
  'error.binary_not_found.body':
    'Монитор не нашёл исполняемый файл. Установите программу или укажите путь к ней в настройках.',
  'error.spawn_failed.title': 'Не удалось запустить программу',
  'error.spawn_failed.body': 'Процесс не запустился. Проверьте путь к исполняемому файлу.',
  'error.handshake_failed.title': 'Нет связи с программой',
  'error.handshake_failed.body':
    'Программа запустилась, но ответила не так, как ожидалось. Обновите её и повторите попытку.',
  'error.protocol_error.title': 'Неожиданный формат',
  'error.protocol_error.body': 'Формат данных изменился. Обновите монитор или программу.',
  'error.not_signed_in.title': 'Аккаунт не подключён',
  'error.not_signed_in.body': 'Войдите в аккаунт в программе и повторите попытку.',
  'error.not_running.title': 'Программа закрыта',
  'error.not_running.body':
    'Для этого чтения программа должна быть открыта. Откройте её и повторите попытку.',
  'error.endpoint_changed.title': 'Источник изменил формат',
  'error.endpoint_changed.body':
    'Сервис изменился, и это чтение больше не работает как раньше. Возможно, нужно обновить монитор.',
  'error.http': 'Код HTTP: {httpStatus}.',
} as const satisfies Record<string, string>;
