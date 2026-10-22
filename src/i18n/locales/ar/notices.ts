// Notice codes from docs/ipc.md. An unknown code renders `notice.unknown` with the code in it,
// so a missing translation is visible instead of silent.
export const noticeMessages = {
  'notice.code.claude_waiting_first_session':
    'تم تثبيت الأمر. تصل الحدود بعد أول ردّ في جلسة Claude Code.',
  'notice.code.claude_no_rate_limits':
    'لا يُبلغ Claude Code عن الحدود (فوترة بمفتاح API أو باقة بلا حدود).',
  'notice.code.claude_shim_outdated':
    'لم يُعثر على الأمر المثبّت في شريط حالة Claude Code. أعد تثبيته لاستئناف القراءة.',
  'notice.code.claude_oauth_partial':
    'الاستخدام لكل نموذج (claude /usage أو واجهة استخدام الحساب) مفعّل لكنه فشل ({reason}). يُعرض فقط ما يأتي من شريط الحالة.',
  'notice.code.codex_rollout_fallback':
    'لم يستجب codex app-server ({reason}). تتم قراءة آخر جلسة مسجّلة؛ قد تكون الأرقام قديمة.',
  'notice.code.codex_no_limits': 'حساب Codex لا يُبلغ عن الحدود (تسجيل دخول بمفتاح API).',
  'notice.code.antigravity_pool_unknown':
    'لم تُرجع حصة {pool} أي بيانات. تظهر كقيمة غير معروفة وليست صفرًا.',
  'notice.code.litellm_no_budget': 'ليس لهذا المفتاح أو المستخدم أو الفريق سقف ميزانية.',
  'notice.code.litellm_level_unreadable':
    'تعذّرت قراءة ميزانية {level} ({reason}). قد يوجد سقف هناك لا يظهر هنا.',
  'notice.code.litellm_windows_truncated':
    'عدد نوافذ الميزانية أكبر مما يتسع هنا. تم حذف الأقل استخدامًا: {dropped}.',
  'notice.code.opencode_window_idle':
    'نافذة {window} عند 0%: لا يبدأ احتسابها إلا مع الطلب التالي.',
  'notice.code.opencode_rate_limited': 'نافذة {window} خاضعة لتقييد المعدّل.',
  'notice.code.opencode_no_key':
    'يُعرض التقدير المحلي فقط. الصق مفتاح Go API لرؤية النوافذ الحقيقية.',
  'notice.code.opencode_local_estimate':
    'القيم المحلية تقدير بسعر القائمة، من هذا الكمبيوتر فقط. وهي ليست الحصة.',
  'notice.code.opencode_db_unreadable': 'تعذّرت قراءة قاعدة بيانات OpenCode المحلية: {reason}.',
  'notice.unknown': 'إشعار غير مترجم: {code}',
  'notice.action.reinstall': 'إعادة التثبيت',
  'notice.action.settings': 'إدخال المفتاح',
  'notice.param.window.rolling': 'مدتها 5 ساعات',
  'notice.param.window.weekly': 'أسبوعية',
  'notice.param.window.monthly': 'شهرية',
  'notice.param.level.key': 'المفتاح',
  'notice.param.level.user': 'المستخدم',
  'notice.param.level.team': 'الفريق',
  'notice.param.dropped.zero': '{n} نافذة',
  'notice.param.dropped.one': 'نافذة واحدة ({n})',
  'notice.param.dropped.two': 'نافذتان ({n})',
  'notice.param.dropped.few': '{n} نوافذ',
  'notice.param.dropped.many': '{n} نافذة',
  'notice.param.dropped.other': '{n} نافذة',
  // The OpenCode database reasons are their own vocabulary: `not_found` there is a missing file.
  'notice.reason.opencode_db_unreadable.invalid_path': 'مسار مجلد البيانات غير صالح',
  'notice.reason.opencode_db_unreadable.not_found': 'لم يُعثر على ملف قاعدة البيانات',
  'notice.reason.opencode_db_unreadable.permission': 'لا توجد صلاحية لفتح ملف قاعدة البيانات',
  'notice.reason.opencode_db_unreadable.locked': 'قاعدة البيانات قيد الاستخدام من برنامج آخر',
  'notice.reason.opencode_db_unreadable.not_a_database': 'الملف الذي وُجد ليس قاعدة بيانات',
  'notice.reason.opencode_db_unreadable.schema': 'تغيّر تنسيق قاعدة البيانات',
  'notice.reason.opencode_db_unreadable.timeout': 'استغرقت قراءة قاعدة البيانات وقتًا طويلًا',
  'notice.reason.opencode_db_unreadable.other': 'سبب غير محدد',
  'notice.severity.info': 'معلومة',
  'notice.severity.warn': 'تحذير',
  'notice.severity.error': 'خطأ',
} as const satisfies Record<string, string>;
