// One title and one body for every ErrorCode (src/lib/contract/view.ts), plus the OpenCode
// ambiguity wording. src/i18n/catalog.test.ts fails when a code is missing.
export const errorMessages = {
  'error.internal.title': 'حدث خطأ ما',
  'error.internal.body':
    'وقع خطأ داخلي أثناء قراءة هذا المصدر. حاول مرة أخرى؛ وإن تكرر ذلك فأعد تشغيل المراقب.',
  'error.timeout.title': 'استغرقت القراءة وقتًا طويلًا',
  'error.timeout.body': 'لم يستجب المصدر في الوقت المحدد. حاول مرة أخرى بعد قليل.',
  'error.network.title': 'تعذّر الوصول إلى المصدر',
  'error.network.body': 'تعذّر الاتصال بالخادم. تحقق من الشبكة وحاول مرة أخرى.',
  'error.bad_response.title': 'استجابة غير متوقعة',
  'error.bad_response.body': 'ردّ المصدر بما لا يفهمه المراقب. حاول مرة أخرى لاحقًا.',
  'error.response_too_large.title': 'الاستجابة كبيرة جدًا',
  'error.response_too_large.body': 'تجاوزت الاستجابة الحجم المقبول وتم تجاهلها للأمان.',
  'error.rate_limited.title': 'طلبات كثيرة جدًا',
  'error.rate_limited.body':
    'طلب المصدر الانتظار قليلًا. سيعيد المراقب المحاولة تلقائيًا بفاصل أطول.',
  'error.unauthorized.title': 'تم رفض المفتاح أو تسجيل الدخول',
  'error.unauthorized.body': 'رفض الخادم بيانات الاعتماد. تحقق من المفتاح ونطاقه في الإعدادات.',
  'error.unauthorized_ambiguous.title': 'تعذّر التحقق من المفتاح أو من اشتراك Go',
  'error.unauthorized_ambiguous.body':
    'رفضت الخدمة الطلب. يحدث هذا مع المفاتيح الصحيحة وأيضًا بدون اشتراك Go. تحقق من المفتاح والاشتراك في وحدة تحكم OpenCode.',
  'error.unauthorized_ambiguous.kimi.title': 'تعذّر التحقق من مفتاح Kimi Code',
  'error.unauthorized_ambiguous.kimi.body':
    'رفضت الخدمة الطلب. يحدث هذا مع المفاتيح المنتهية، وأيضًا مع مفتاح من منصة Moonshot المفتوحة، وهي منتج مختلف. تأكد من وحدة تحكم Kimi Code أن المفتاح يخص الباقة.',
  'error.unauthorized_ambiguous.zai.title': 'تعذّر التحقق من مفتاح z.ai',
  'error.unauthorized_ambiguous.zai.body':
    'رفضت الخدمة الطلب. يحدث هذا مع المفاتيح الصحيحة وأيضًا مع الحسابات التي لا تملك باقة GLM Coding. تحقق من المفتاح والباقة في لوحة z.ai.',
  'error.not_found.title': 'العنوان غير موجود',
  'error.not_found.body':
    'لا يتعرف الخادم على المسار أو المعرّف المُدخل. تحقق من الرابط والنطاق والمعرّف.',
  'error.server_error.title': 'خطأ في الخادم',
  'error.server_error.body': 'تعطّل خادم المصدر. حاول مرة أخرى لاحقًا.',
  'error.tls_error.title': 'فشل في شهادة TLS',
  'error.tls_error.body': 'لم تُقبل شهادة الخادم. اختر مرجع الشهادات (CA) الصحيح في الإعدادات.',
  'error.invalid_url.title': 'رابط غير صالح',
  'error.invalid_url.body':
    'استخدم https://. لا يعمل HTTP إلا مع localhost، ولا يجوز أن يحتوي الرابط على اسم مستخدم أو كلمة مرور أو معلمات استعلام أو #.',
  'error.invalid_input.title': 'قيمة غير صالحة',
  'error.invalid_input.body': 'صحّح الحقل المشار إليه وحاول مرة أخرى.',
  'error.settings_unreadable.title': 'ملف الإعدادات غير قابل للقراءة',
  'error.settings_unreadable.body':
    'تعذّرت قراءة settings.json. لم يتغير شيء. أصلح الملف وحاول مرة أخرى.',
  'error.install_failed.title': 'فشل التثبيت',
  'error.install_failed.body': 'تعذّر تثبيت الأمر في شريط حالة Claude Code. لم يتغير شيء.',
  'error.unsupported_plan.title': 'الباقة لا تتضمن حدودًا للقراءة',
  'error.unsupported_plan.body':
    'يستخدم الحساب باقة أو مفتاح API لا يُبلغ عن الحدود. يمكنك إيقاف هذا المصدر.',
  'error.oauth_expired.title': 'انتهت صلاحية تسجيل الدخول إلى Claude Code',
  'error.oauth_expired.body': 'افتح Claude Code لتجديد تسجيل الدخول ثم حاول مرة أخرى.',
  'error.binary_not_found.title': 'البرنامج غير موجود',
  'error.binary_not_found.body':
    'تعذّر على المراقب العثور على الملف التنفيذي. ثبّت البرنامج أو حدّد مساره في الإعدادات.',
  'error.spawn_failed.title': 'تعذّر تشغيل البرنامج',
  'error.spawn_failed.body': 'لم تبدأ العملية. تحقق من مسار الملف التنفيذي.',
  'error.handshake_failed.title': 'تعذّر الاتصال بالبرنامج',
  'error.handshake_failed.body': 'بدأ البرنامج لكنه لم يستجب كما هو متوقع. حدّثه وحاول مرة أخرى.',
  'error.protocol_error.title': 'تنسيق غير متوقع',
  'error.protocol_error.body': 'تغيّر تنسيق البيانات. حدّث المراقب أو البرنامج.',
  'error.not_signed_in.title': 'الحساب غير مرتبط',
  'error.not_signed_in.body': 'سجّل الدخول إلى حسابك في البرنامج وحاول مرة أخرى.',
  'error.not_running.title': 'البرنامج مغلق',
  'error.not_running.body': 'تحتاج هذه القراءة إلى أن يكون البرنامج مفتوحًا. افتحه وحاول مرة أخرى.',
  'error.endpoint_changed.title': 'غيّر المصدر تنسيقه',
  'error.endpoint_changed.body':
    'تغيّرت الخدمة ولم تعد هذه القراءة تعمل كما كانت. قد يحتاج المراقب إلى تحديث.',
  'error.http': 'رمز HTTP: {httpStatus}.',
} as const satisfies Record<string, string>;
