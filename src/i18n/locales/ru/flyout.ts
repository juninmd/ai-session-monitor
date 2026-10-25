// ru catalog: flyout (onboarding, command palette, provider detail, history board).
export const flyoutMessages = {
  // Onboarding wizard
  'onboarding.step.choose': 'Выбор',
  'onboarding.step.connect': 'Подключение',
  'onboarding.step.verify': 'Проверка',
  'onboarding.step.done': 'Готово',
  'onboarding.choose.title': 'Что вы хотите отслеживать?',
  'onboarding.choose.note':
    'Отмеченные пункты найдены на этом компьютере. Ничего не читается, пока вы не продолжите.',
  'onboarding.tag.responds': 'отвечает',
  'onboarding.tag.silent': 'нет данных',
  'onboarding.tag.needs_key': 'нужен ключ',
  'onboarding.tag.needs_consent': 'нужно разрешение',
  'onboarding.connect.title': 'Учётные данные и разрешения',
  'onboarding.connect.consent_note':
    'Этот источник читает файлы или процессы на этом компьютере. Ничего не читается, пока вы не согласитесь.',
  'onboarding.connect.allow': 'Разрешить чтение',
  'onboarding.connect.details': 'Подробнее',
  'onboarding.connect.litellm_note': 'Нужны адрес прокси, область и ключ.',
  'onboarding.connect.multi_note':
    'Подключиться можно разными способами: ключом или чтением локальной базы данных.',
  'onboarding.connect.configure': 'Настроить',
  'onboarding.connect.nothing':
    'Подключать нечего: у выбранных источников уже есть всё необходимое.',
  'onboarding.verify.title': 'Что ответил каждый источник',
  'onboarding.verify.reading': 'Чтение…',
  'onboarding.verify.read_now': 'Прочитать сейчас',
  'onboarding.verify.first.one': '{count} квота · первая: использовано {pct}',
  'onboarding.verify.first.few': '{count} квоты · первая: использовано {pct}',
  'onboarding.verify.first.many': '{count} квот · первая: использовано {pct}',
  'onboarding.verify.first.other': '{count} квоты · первая: использовано {pct}',
  'onboarding.verify.back_to_key': 'Вернуться и вставить ключ',
  'onboarding.done.title': 'Отслеживается: {done} из {total}',
  'onboarding.done.body':
    'Панель откроется с сортировкой по срочности: то, что скорее закончится, будет первым. Источники, которые не ответили, останутся в разделе «Нет данных», без выдуманных чисел.',
  'onboarding.done.go': 'Перейти к панели',
  'onboarding.done.again': 'Прочитать снова',
  'onboarding.back': 'Назад',
  'onboarding.continue': 'Продолжить ({count})',
  'onboarding.connected': 'Я подключил',
  'onboarding.finish': 'Завершить',
  'onboarding.outcome.loading': 'Чтение…',
  'onboarding.outcome.disabled': 'Выкл.',
  'onboarding.outcome.needs_consent': 'Ожидает вашего разрешения',
  'onboarding.outcome.needs_setup': 'Нет учётных данных',
  'onboarding.outcome.no_data': 'Данных пока нет',
  'onboarding.outcome.failed': 'Не удалось прочитать',
  'onboarding.outcome.stale': 'Старые данные',
  'onboarding.outcome.stale_ago': 'Старые данные, {age} назад',
  'onboarding.outcome.no_windows': 'Читаются значения, квот нет',
  'onboarding.outcome.quotas.one': '{count} квота',
  'onboarding.outcome.quotas.few': '{count} квоты',
  'onboarding.outcome.quotas.many': '{count} квот',
  'onboarding.outcome.quotas.other': '{count} квоты',

  // Ctrl+K palette
  'palette.group.providers': 'Провайдеры',
  'palette.group.actions': 'Действия',
  'palette.group.settings': 'Настройки',
  'palette.group.demo': 'Демо',
  'palette.refresh.label': 'Обновить все источники',
  'palette.refresh.hint': 'прочитать снова сейчас',
  'palette.refresh.keywords': 'обновить перезагрузить refresh reload update',
  'palette.copy.label': 'Копировать сводку состояния',
  'palette.copy.hint.one': '{count} источник текстом',
  'palette.copy.hint.few': '{count} источника текстом',
  'palette.copy.hint.many': '{count} источников текстом',
  'palette.copy.hint.other': '{count} источника текстом',
  'palette.copy.keywords': 'буфер обмена копировать состояние отчёт clipboard copy status report',
  'palette.general.label': 'Общие настройки',
  'palette.general.hint': 'отображение, лимиты, трей',
  'palette.general.keywords': 'параметры опции preferences options',
  'palette.configure.label': 'Настроить {name}',
  'palette.configure.hint_stale': 'старые данные',
  'palette.configure.hint_default': 'источник и учётные данные',
  'palette.configure.keywords':
    'настроить подключить источник ключ configure set up adjust source credential',
  'palette.scenario.label': 'Сценарий: {name}',
  'palette.scenario.hint': 'вымышленные данные',
  'palette.scenario.keywords': 'демо тестовые данные mock demo data',
  'palette.close_aria': 'Закрыть быстрые команды',
  'palette.dialog_aria': 'Быстрые команды',
  'palette.input_aria': 'Поиск команд',
  'palette.placeholder': 'Перейти к провайдеру, обновить, копировать сводку…',
  'palette.empty': 'Ничего не найдено',
  'palette.foot.navigate': 'навигация',
  'palette.foot.run': 'выполнить',
  'palette.key.esc': 'Esc',
  'palette.key.enter': 'Enter',

  // Provider actions (preview only)
  'provider_actions.outcome.reset': 'Сброс имитирован. Квоты и баланс обновлены в предпросмотре.',
  'provider_actions.outcome.noCredit': 'Сброс недоступен. Баланс изменился; обновите данные.',
  'provider_actions.outcome.nothingToReset': 'В предпросмотре нет квоты для обновления.',
  'provider_actions.outcome.unavailable': 'Действие недоступно для этих данных.',
  'provider_actions.resets.one': 'Доступен {count} сброс',
  'provider_actions.resets.few': 'Доступно {count} сброса',
  'provider_actions.resets.many': 'Доступно {count} сбросов',
  'provider_actions.resets.other': 'Доступно {count} сброса',
  'provider_actions.use_reset': 'Использовать сброс',
  'provider_actions.confirm_aria': 'Подтвердить сброс квоты',
  'provider_actions.confirm_body':
    'Использовать 1 сброс Codex? Это действие необратимо. В этом предпросмотре оно имитируется.',
  'provider_actions.confirm': 'Подтвердить сброс',
  'provider_actions.cancel': 'Отмена',
  'provider_actions.reset_failed': 'Не удалось имитировать сброс. Повторите попытку.',
  'provider_actions.low_priority_offer': 'Предложение с низким приоритетом · предпросмотр',
  'provider_actions.copy_command': 'Копировать /low-priority',
  'provider_actions.low_priority_hint':
    'Используйте {command} в интерактивном сеансе, который предложил этот режим.',
  'provider_actions.copied': 'Команда скопирована. Режим всё равно нужно включить в Claude Code.',
  'provider_actions.copy_failed':
    'Не удалось скопировать. Выделите команду выше и скопируйте вручную.',

  // History board and trend
  'history.board.no_readable':
    'Ни одна квота этого источника не хранит данные со временем обновления.',
  'history.board.failed': 'Не удалось прочитать историю.',
  'history.board.loading': 'Загрузка данных…',
  'history.board.empty': 'Для этой квоты пока нет сохранённых данных.',
  'history.trend.none': 'В этом окне нет данных.',
  'history.trend.first': 'Пик на данный момент: {peak}; прошлых обновлений для сравнения нет.',
  'history.trend.compare.one':
    'Пик на данный момент: {peak} · предыдущих обновлений: {count}, в среднем они закрывались на {avg}.',
  'history.trend.compare.few':
    'Пик на данный момент: {peak} · предыдущих обновлений: {count}, в среднем они закрывались на {avg}.',
  'history.trend.compare.many':
    'Пик на данный момент: {peak} · предыдущих обновлений: {count}, в среднем они закрывались на {avg}.',
  'history.trend.compare.other':
    'Пик на данный момент: {peak} · предыдущих обновлений: {count}, в среднем они закрывались на {avg}.',
  'history.trend.projection_desc':
    'Прогноз следует текущему темпу, пока квота не закончится через {duration}.',

  // Quota sheet
  'quota.status.ok': 'с запасом',
  'quota.status.warn': 'близко к лимиту',
  'quota.status.crit': 'критично',
  'quota.status.out': 'исчерпано',
  'quota.amount.unlimited': 'потрачено {used}, без лимита',
  'quota.amount.of': '{used} из {limit}',
  'quota.amount.free': 'свободно {free}',
  'quota.renew.restarted': 'перезапущено',
  'quota.renew.restarted_at': 'перезапущено в {time}',
  'quota.renew.in': 'обновится через {duration}',
  'quota.spoken_left': 'До обновления {duration}.',
  'quota.reading.expired': 'Истекло',
  'quota.reading.stale': 'Старое',
  'quota.spoken_stale': '{name}: данные устарели.',
  'quota.forecast.out': 'Исчерпано',
  'quota.forecast.risk': 'Закончится через {duration}',
  'quota.forecast.fine': 'Хватит до обновления',
  'quota.forecast.unknown': 'Недостаточно данных для прогноза',

  // Provider detail drawer
  'drawer.close': 'Закрыть подробности',
  'drawer.refresh': 'Обновить {name}',
  'drawer.pin': 'Закрепить {name} сверху',
  'drawer.unpin': 'Открепить {name}',
  'drawer.settings': 'Настроить {name}',

  // Candidate sources (not wired yet)
  'candidates.amp.credential': 'Локальный CLI (`amp`)',
  'candidates.amp.offer':
    '`amp usage` возвращает баланс кредитов и включённый расход. Тот же шаблон с фиксированным CLI, что уже используется для Claude и Antigravity.',
  'candidates.kiro.credential': 'Собственные учётные данные CLI',
  'candidates.kiro.offer':
    'GetUsageLimits возвращает один пул кредитов с использованием, лимитом, процентом и сбросом.',
  'candidates.cline.credential': 'Ключ API',
  'candidates.cline.offer':
    '`/users/me/plan/usage-limits`: окна 5 ч, неделя и месяц с процентом и resetsAt.',
  'candidates.windsurf.credential': 'Служебный ключ команды',
  'candidates.windsurf.offer':
    'GetTeamCreditBalance и GetUsageConfig через официальный API. Только тариф Enterprise.',
  'candidates.command_code.credential': 'Локальный CLI (`commandcode`)',
  'candidates.command_code.offer':
    '`/alpha/billing/credits` и `/subscriptions`: окна расходов 5 ч и неделя, тариф и месячные кредиты.',
} as const;
