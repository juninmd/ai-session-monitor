// What the params of a failure add to its message: the `reason` a collector gives (closed
// vocabularies, worded per error code), the sentences for a redirect and a wait, and the names
// of the settings fields. A reason with a `.title` replaces the title of the code as well.
export const errorDetailMessages = {
  'error.install_failed.reason.status_line_too_large.body':
    'أمر شريط الحالة الحالي لديك كبير جدًا بحيث لا يستطيع المراقب ربطه بسلسلة. لم يتغير شيء.',
  'error.install_failed.reason.path_unsupported.body':
    'يحتوي مسار مجلد البيانات على أحرف لا يقبلها أمر شريط الحالة. لم يتغير شيء.',
  'error.install_failed.reason.serialize_failed.body':
    'تعذّر إنشاء settings.json الجديد. لم يتغير شيء.',
  'error.install_failed.reason.state_failed.body': 'تعذّر حفظ حالة التثبيت. لم يتغير settings.json.',
  'error.install_failed.reason.concurrent_modification.body':
    'تغيّر settings.json أثناء العملية، فتركه المراقب كما هو. حاول مرة أخرى.',
  'error.install_failed.reason.write_failed.body':
    'تعذّرت الكتابة في settings.json. لم يتغير الملف؛ تأكد من أنه غير مقفل وحاول مرة أخرى.',
  'error.install_failed.reason.shim_missing.body':
    'لم يُعثر على الملف التنفيذي للأمر بجوار المراقب. أعد تثبيت المراقب.',
  'error.install_failed.reason.copy_failed.body':
    'تعذّر نسخ الملف التنفيذي للأمر إلى مجلد البيانات. لم يتغير settings.json.',
  'error.install_failed.reason.backup_failed.body':
    'تعذّر حفظ نسخة settings.json الاحتياطية، لذا لم يتغير الملف.',
  // Only a removal is refused for this reason.
  'error.install_failed.reason.foreign_status_line.title': 'تعذّرت إزالة الأمر',
  'error.install_failed.reason.foreign_status_line.body':
    'شريط الحالة المُعدّ ليس تابعًا للمراقب (ربما نسخة معدّلة يدويًا)، لذا لم تتم إزالة شيء.',

  // LiteLLM: the CA file the user chose, not the certificate of the server.
  'error.tls_error.reason.ca_missing.body':
    'لم يُعثر على ملف CA المُعدّ. اختر الملف الصحيح في الإعدادات.',
  'error.tls_error.reason.ca_too_large.body':
    'ملف CA كبير جدًا بحيث لا يمكن قراءته. اختر ملف .pem أصغر في الإعدادات.',
  'error.tls_error.reason.ca_unreadable.body':
    'تعذّرت قراءة ملف CA. تحقق من المسار والأذونات في الإعدادات.',
  'error.tls_error.reason.ca_invalid.body':
    'لا يحتوي ملف CA على شهادة صالحة. اختر ملف .pem صالحًا في الإعدادات.',

  // LiteLLM: which part of the answer did not look like a key, user or team record.
  'error.endpoint_changed.reason.not_an_object.body':
    'استجابة الخادم ليست بالتنسيق المتوقع. تأكد من أن الرابط يشير إلى LiteLLM.',
  'error.endpoint_changed.reason.no_record.body':
    'لا تحتوي الاستجابة على سجل مفتاح أو مستخدم أو فريق. تحقق من الرابط والنطاق.',
  'error.endpoint_changed.reason.id_missing.body':
    'لا تؤكد الاستجابة المعرّف الذي تم الاستعلام عنه. تحقق من معرّف المستخدم أو الفريق.',
  'error.endpoint_changed.reason.id_mismatch.body':
    'الاستجابة تخص معرّفًا غير الذي أدخلته. تحقق من معرّف المستخدم أو الفريق.',

  // Antigravity: the IDE's local server is there, but there is nothing to call.
  'error.not_running.reason.no_port.title': 'الخادم المحلي ليس جاهزًا بعد',
  'error.not_running.reason.no_port.body':
    'Antigravity مفتوح، لكن خادمه المحلي لم يوفّر عنوانًا بعد. يزول هذا عادةً بعد لحظات: حاول مرة أخرى.',
  'error.unauthorized.reason.no_token.title': 'الرمز المحلي غير متاح',
  'error.unauthorized.reason.no_token.body':
    'تعذّر على المراقب قراءة رمز الجلسة الخاص بالخادم المحلي للبيئة. أعد تشغيل Antigravity وحاول مرة أخرى.',

  // Cursor: its local state.vscdb database.
  'error.timeout.reason.locked.body':
    'قاعدة بيانات Cursor المحلية قيد الاستخدام الآن. حاول مرة أخرى بعد قليل.',
  'error.timeout.reason.timeout.body':
    'استغرقت قراءة قاعدة بيانات Cursor المحلية وقتًا طويلًا. حاول مرة أخرى بعد قليل.',
  'error.endpoint_changed.reason.schema.body':
    'تنسيق قاعدة بيانات Cursor المحلية مختلف عما يتوقعه المراقب. قد يحتاج المراقب إلى تحديث.',
  'error.settings_unreadable.reason.permission.body':
    'لا توجد صلاحية لفتح قاعدة بيانات Cursor المحلية. تحقق من أذونات الملف.',
  'error.settings_unreadable.reason.not_a_database.body': 'ملف Cursor الذي وُجد ليس قاعدة بيانات.',
  'error.settings_unreadable.reason.other.body': 'تعذّرت قراءة قاعدة بيانات Cursor المحلية.',

  // GitHub Copilot: the token the extension keeps on disk.
  'error.not_signed_in.reason.not_found.body':
    'لم يُعثر على رمز GitHub في مجلد إضافة Copilot. سجّل الدخول إلى Copilot في محررك، أو الصق رمزًا في الإعدادات.',
  'error.not_signed_in.reason.no_token.body':
    'لا تحتوي ملفات إضافة Copilot على رمز لـ github.com. سجّل الدخول إلى Copilot في محررك، أو الصق رمزًا في الإعدادات.',
  'error.not_signed_in.reason.unreadable.body':
    'تعذّرت قراءة ملف تسجيل الدخول الخاص بإضافة Copilot. تحقق من أذونات مجلدها.',

  'error.redirect': 'حاول الخادم إعادة توجيه الطلب، والمراقب لا يتبع عمليات إعادة التوجيه.',
  'error.retry_after': 'مدة الانتظار التي طلبها المصدر: {wait}.',
  'error.field_hint': 'الحقل الذي به مشكلة: {field}.',
  'field.value': 'مفتاح API',
  'field.baseUrl': 'رابط الخادم',
  'field.entityId': 'معرّف المستخدم أو الفريق',
  'field.authHeader': 'ترويسة المصادقة',
  'field.pemPath': 'ملف CA (.pem)',
  'field.host': 'خادم بلا تحقق من الشهادة',
  'field.configDir': 'مجلد إعدادات Claude Code',
  'field.binaryPath': 'مسار الملف التنفيذي لـ Codex',
  'field.dataDir': 'مجلد بيانات OpenCode',
  'field.shimConsent': 'الموافقة على تثبيت الأمر',
  'field.limit': 'عدد النوافذ السابقة',
} as const satisfies Record<string, string>;
