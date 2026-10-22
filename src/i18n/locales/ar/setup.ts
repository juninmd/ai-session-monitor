// First-run connection surfaces: consent cards, secret fields, OpenCode and LiteLLM forms.
export const setupMessages = {
  'consent.skip': 'لاحقًا',
  'consent.details': 'ما الذي سيُقرأ',

  'claude.connect.title': 'الربط مع Claude Code',
  'claude.connect.lead':
    'يضيف المراقب أمرًا صغيرًا إلى شريط حالة Claude Code. ويخزّن نسب الحدود فقط (5 ساعات والأسبوع) في ملف محلي. لا تتم قراءة أي تسجيل دخول.',
  'claude.connect.detail':
    'يعدّل settings.json الخاص بـ Claude Code مع نسخة احتياطية، ويربط شريط الحالة الحالي لديك بسلسلة. يكتب الحدود فقط في %LOCALAPPDATA%. بلا شبكة وبلا رموز. يعمل مع حسابات Pro وMax. تظهر الأرقام بعد أول ردّ في الجلسة ولا تتحدّث إلا أثناء فتح Claude Code.',
  'claude.connect.allow': 'تثبيت الأمر',
  'claude.connect.tip':
    'نصيحة: يخفي Windows الأيقونات الجديدة في قائمة السهم الإضافية. اسحب أيقونة المراقب إلى شريط المهام لتبقى ظاهرة دائمًا.',

  'codex.connect.title': 'الربط مع Codex',
  'codex.connect.lead':
    'يشغّل المراقب codex app-server المثبّت هنا ويسأل عن المتبقي من نافذة 5 ساعات والأسبوع.',
  'codex.connect.detail':
    'يشغّل برنامج codex على جهازك، للقراءة فقط. لا تغادر أي بيانات هذا الكمبيوتر ولا يلمس التطبيق ملف تسجيل دخول Codex. تصنّف OpenAI الخادم app-server على أنه تجريبي، لذا قد يتغير التنسيق بين الإصدارات.',
  'codex.connect.allow': 'السماح بالقراءة',

  'antigravity.connect.title': 'الربط مع Antigravity',
  'antigravity.connect.lead':
    'تأتي الحصة من agy المثبّت (CLI) أو من الخادم المحلي للبيئة. هذا المسار غير رسمي وقد يتوقف عن العمل عند تحديث Antigravity.',
  'antigravity.connect.detail':
    'بإذنك، يمكن للمراقب تشغيل agy المثبّت فقط لقراءة /usage (دون استهلاك رموز أو قراءة بيانات تسجيل الدخول) و/أو قراءة سطر أوامر خادم البيئة للحصول على رمز الجلسة المحلي، مع التواصل مع 127.0.0.1 فقط. لا يُكتب شيء على القرص ولا يُرسل شيء إلى الإنترنت.',
  'antigravity.connect.allow': 'السماح بالقراءة المحلية',

  'shim.title': 'أمر شريط الحالة',
  'shim.state.not_installed': 'غير مثبّت',
  'shim.state.installed': 'مثبّت',
  'shim.state.installed_chained': 'مثبّت، ومربوط بشريط الحالة الخاص بك',
  'shim.state.foreign': 'يُستخدم شريط حالة آخر',
  'shim.state.outdated': 'قديم: لم يُعثر على الملف التنفيذي',
  'shim.state.unreadable': 'settings.json غير قابل للقراءة',
  'shim.hint.foreign': 'عند التثبيت، يستمر عمل شريط الحالة الحالي لديك: يربطه المراقب بسلسلة.',
  'shim.hint.unreadable':
    'لا يلمس المراقب ملفًا لا يستطيع قراءته. أصلح settings.json وتحقق مرة أخرى.',
  'shim.path': 'الملف: {path}',
  'shim.install': 'تثبيت الأمر',
  'shim.reinstall': 'إعادة تثبيت الأمر',
  'shim.uninstall': 'إزالة الأمر',
  'shim.recheck': 'التحقق مرة أخرى',
  'shim.loading': 'جارٍ فحص Claude Code',

  'secret.saved': 'تم حفظ المفتاح',
  'secret.not_saved': 'لا يوجد مفتاح محفوظ',
  'secret.placeholder': 'الصق المفتاح',
  'secret.placeholder_keep': 'اتركه فارغًا للاحتفاظ بالمفتاح المحفوظ',
  'secret.hint': 'يبقى في Windows Credential Manager ولا يظهر في الواجهة أبدًا.',
  'secret.save': 'حفظ المفتاح',
  'secret.clear': 'إزالة المفتاح',
  'secret.empty': 'الصق مفتاح API.',
  'secret.saved_ok': 'تم حفظ المفتاح.',
  'secret.cleared_ok': 'تمت إزالة المفتاح.',

  'opencode.title': 'الربط مع OpenCode Go',
  'opencode.lead':
    'الصق مفتاح OpenCode Go API لقراءة النوافذ الحقيقية (5 ساعات والأسبوع والشهر). وبدون المفتاح لا يمكنك رؤية سوى تقدير محلي للاستخدام.',
  'opencode.key_label': 'مفتاح OpenCode Go API',
  'opencode.console': 'فتح وحدة تحكم OpenCode Go',
  'opencode.billing': 'عرض فوترة Zen',
  'opencode.zen_note': 'رصيد Zen (المدفوع مسبقًا) ليس له API: راجع لوحة الفوترة.',
  'opencode.local_label': 'تقدير محلي (هذا الكمبيوتر فقط)',
  'opencode.local_hint':
    'يفتح قاعدة بيانات OpenCode للقراءة فقط ويجمع التكلفة والرموز فقط، بسعر القائمة. وهي ليست الحصة الحقيقية.',
  'opencode.data_dir': 'مجلد بيانات OpenCode',
  'opencode.data_dir_hint': 'اتركه فارغًا لاستخدام المجلد الافتراضي.',
  'opencode.save_settings': 'حفظ المجلد',

  'kimi.title': 'الربط مع Kimi Code',
  'kimi.lead':
    'الصق مفتاح Kimi Code API لقراءة حصة الباقة: نافذة 5 ساعات والأسبوع وإجمالي الشهر، عندما تُبلغ الباقة عنها.',
  'kimi.detail':
    'قراءة غير رسمية: واجهة الاستخدام غير موثّقة وقد تتغير دون إشعار. يُرسل المفتاح إلى api.kimi.com فقط ويبقى في Windows Credential Manager. لا يُستخدم أي ملف تعريف ارتباط أو تسجيل دخول من أداة أخرى. مفتاح منصة Moonshot المفتوحة منتج مختلف ولا يعمل هنا.',
  'kimi.key_label': 'مفتاح Kimi Code API',
  'kimi.console': 'فتح وحدة تحكم Kimi Code',
  'kimi.note':
    'قراءة غير رسمية لواجهة استخدام Kimi Code. لا يظهر الأسبوع والشهر إلا عندما تُبلغ الباقة عنهما.',

  'zai.title': 'الربط مع z.ai',
  'zai.lead':
    'الصق مفتاح API الخاص بباقة GLM Coding لقراءة نافذة 5 ساعات والأسبوع وحصة أدوات MCP الشهرية.',
  'zai.detail':
    'قراءة غير رسمية: واجهة الحصة غير موثّقة وقد تتغير دون إشعار. يُرسل المفتاح إلى api.z.ai فقط ويبقى في Windows Credential Manager. لا يُستخدم أي ملف تعريف ارتباط أو تسجيل دخول من أداة أخرى. حسابات BigModel (open.bigmodel.cn) لا تُقرأ.',
  'zai.key_label': 'مفتاح z.ai API',
  'zai.console': 'فتح مفاتيح z.ai',
  'zai.note': 'قراءة غير رسمية لواجهة حصة باقة GLM Coding، على الخادم العالمي (api.z.ai).',

  'openrouter.title': 'الربط مع OpenRouter',
  'openrouter.lead':
    'الصق مفتاح OpenRouter API لرؤية حد إنفاق المفتاح والاستخدام، وكذلك رصيد الائتمان إن سمح به المفتاح.',
  'openrouter.detail':
    'قراءة رسمية عبر واجهة OpenRouter الموثّقة. يُرسل المفتاح إلى openrouter.ai فقط ويبقى في Windows Credential Manager. يتطلب رصيد الائتمان مفتاح إدارة؛ ومع مفتاح عادي يظهر حد المفتاح والاستخدام فقط.',
  'openrouter.key_label': 'مفتاح OpenRouter API',
  'openrouter.console': 'فتح مفاتيح OpenRouter',
  'openrouter.note':
    'قراءة رسمية لواجهة OpenRouter. بدون حد على المفتاح لا توجد نسبة مئوية، بل الاستخدام فقط. ولا يظهر الرصيد إلا مع مفتاح إدارة.',

  'deepseek.title': 'الربط مع DeepSeek',
  'deepseek.lead': 'الصق مفتاح DeepSeek API لرؤية رصيد الحساب، في كل عملة يتوفر فيها رصيد.',
  'deepseek.detail':
    'قراءة رسمية عبر واجهة DeepSeek الموثّقة. يُرسل المفتاح إلى api.deepseek.com فقط ويبقى في Windows Credential Manager. ليس للرصيد حد ولا وقت إعادة ضبط، فلا توجد نسبة مئوية؛ ولا يُجمع اليوان والدولار معًا أبدًا.',
  'deepseek.key_label': 'مفتاح DeepSeek API',
  'deepseek.console': 'فتح مفاتيح DeepSeek',
  'deepseek.note':
    'قراءة رسمية لواجهة رصيد DeepSeek. إنه مال وليس حصة: لا توجد نسبة مئوية ولا إعادة ضبط.',

  'deepinfra.title': 'الربط مع DeepInfra',
  'deepinfra.lead': 'الصق مفتاح DeepInfra API لرؤية الرصيد وحد الإنفاق وإنفاق هذا الشهر.',
  'deepinfra.detail':
    'قراءة رسمية عبر واجهة فوترة DeepInfra. يُرسل المفتاح إلى api.deepinfra.com فقط ويبقى في Windows Credential Manager. بدون حد إنفاق لا توجد نسبة مئوية، بل مبالغ بالدولار فقط.',
  'deepinfra.key_label': 'مفتاح DeepInfra API',
  'deepinfra.console': 'فتح مفاتيح DeepInfra',
  'deepinfra.note':
    'قراءة رسمية لواجهة فوترة DeepInfra. تُقرأ القيم بالدولار؛ وبدون حد إنفاق لا توجد نسبة مئوية.',

  'moonshot.title': 'الربط مع Moonshot',
  'moonshot.lead': 'الصق مفتاح API من منصة Moonshot (platform.moonshot.ai) لرؤية رصيد الحساب.',
  'moonshot.detail':
    'قراءة رسمية عبر واجهة الرصيد الموثّقة من Moonshot. يُرسل المفتاح إلى api.moonshot.ai فقط ويبقى في Windows Credential Manager. مفاتيح منصة kimi.com منفصلة ولا تعمل هنا. الرصيد مال: لا توجد نسبة مئوية ولا إعادة ضبط.',
  'moonshot.key_label': 'مفتاح Moonshot API',
  'moonshot.console': 'فتح مفاتيح Moonshot',
  'moonshot.note':
    'قراءة رسمية لواجهة رصيد Moonshot، بالدولار. قد يصبح الرصيد النقدي سالبًا عند وجود مبلغ مستحق.',

  'novita.title': 'الربط مع Novita',
  'novita.lead': 'الصق مفتاح Novita API لرؤية رصيد الحساب وحد الائتمان والرسوم المعلّقة.',
  'novita.detail':
    'قراءة رسمية عبر واجهة الفوترة الموثّقة من Novita. يُرسل المفتاح إلى api.novita.ai فقط ويبقى في Windows Credential Manager. الرصيد مال: لا توجد نسبة مئوية ولا إعادة ضبط.',
  'novita.key_label': 'مفتاح Novita API',
  'novita.console': 'فتح مفاتيح Novita',
  'novita.note':
    'قراءة رسمية لواجهة رصيد Novita، بالدولار. قد يصبح الرصيد سالبًا عند وجود مبلغ مستحق.',

  'ollama.title': 'الربط مع Ollama Cloud',
  'ollama.lead': 'الصق مفتاح Ollama Cloud API لرؤية الرصيد المضمّن في الباقة والرصيد المشترى.',
  'ollama.detail':
    'قراءة رسمية عبر واجهة رصيد Ollama Cloud الموثّقة. أنشئ المفتاح من ollama.com/settings/keys: لا يُقرأ مفتاح اشتراك CLI أبدًا. يُرسل المفتاح إلى ollama.com فقط ويبقى في Windows Credential Manager. يعتمد تنسيق الاستجابة على الباقة (رصيد شهري أو نوافذ جلسة وأسبوع).',
  'ollama.key_label': 'مفتاح Ollama Cloud API',
  'ollama.console': 'فتح مفاتيح Ollama Cloud',
  'ollama.note':
    'قراءة رسمية لواجهة رصيد Ollama Cloud. تعرض الباقات الحالية رصيدًا بالدولار؛ وتعرض الباقات الأقدم نسب الجلسة والأسبوع.',

  'minimax.title': 'الربط مع MiniMax',
  'minimax.lead': 'الصق مفتاح MiniMax Token Plan لرؤية حصص كل نموذج: الفترة الحالية والأسبوع.',
  'minimax.detail':
    'قراءة غير رسمية: ليس لمسار حصة Token Plan مخطط منشور وقد يتغير دون إشعار. يُرسل المفتاح إلى api.minimax.io (الخادم العالمي) فقط ويبقى في Windows Credential Manager. قارن الأرقام بوحدة تحكم MiniMax في المرة الأولى.',
  'minimax.key_label': 'مفتاح MiniMax Token Plan',
  'minimax.console': 'فتح مفاتيح MiniMax',
  'minimax.note':
    'قراءة غير رسمية لحصة MiniMax Token Plan، لكل نموذج. النموذج الذي لا حصة له في الباقة لا يظهر.',

  'litellm.title': 'الربط مع LiteLLM',
  'litellm.title_edit': 'اتصال LiteLLM',
  'litellm.url': 'رابط الخادم',
  'litellm.url_placeholder': 'https://litellm.yourcompany.com',
  'litellm.url_hint': 'HTTPS مطلوب. لا يُقبل HTTP إلا مع localhost.',
  'litellm.url_required': 'أدخل رابط الخادم.',
  'litellm.url_invalid': 'أدخل رابطًا كاملًا، مثل https://litellm.yourcompany.com.',
  'litellm.url_http': 'استخدم https://. لا يعمل HTTP إلا مع localhost.',
  'litellm.url_parts': 'أزل اسم المستخدم وكلمة المرور ومعلمات الاستعلام و# من الرابط.',
  'litellm.key': 'مفتاح API',
  'litellm.key_required': 'الصق مفتاح LiteLLM API.',
  'litellm.key_rebind':
    'المفتاح المحفوظ ينطبق على الاتصال الحالي فقط. بما أنه تغيّر، الصق المفتاح مرة أخرى.',
  'litellm.scope': 'ما الذي يُقاس',
  'litellm.scope.key': 'هذا المفتاح',
  'litellm.scope.user': 'المستخدم',
  'litellm.scope.team': 'الفريق',
  'litellm.entity.user': 'معرّف المستخدم',
  'litellm.entity.team': 'معرّف الفريق',
  'litellm.entity_required': 'أدخل المعرّف.',
  'litellm.advanced': 'متقدم',
  'litellm.auth_header': 'ترويسة المصادقة',
  'litellm.auth_header_hint': 'غيّرها فقط إذا كان الوكيل لديك يستخدم ترويسة خاصة للمفتاح.',
  'litellm.auth_header_invalid': 'استخدم الحروف والأرقام والشرطات فقط.',
  'litellm.poll': 'التحديث كل',
  'litellm.timeout': 'مهلة الاتصال',
  'litellm.tls': 'شهادة TLS',
  'litellm.tls.system': 'من Windows',
  'litellm.tls.custom_ca': 'CA مخصص',
  'litellm.tls.insecure': 'عدم التحقق',
  'litellm.pem': 'ملف CA (.pem)',
  'litellm.pem_placeholder': 'C:\\certs\\ca.pem',
  'litellm.pem_required': 'أدخل مسار ملف .pem.',
  'litellm.insecure_host': 'خادم بلا تحقق',
  'litellm.insecure_host_hint': 'يتم تخطي الشهادة لهذا الخادم فقط.',
  'litellm.insecure_host_required': 'أدخل الخادم.',
  'litellm.insecure_warning':
    'لن يتم التحقق من الاتصال وقد يُعترض المفتاح. استخدمه فقط على شبكة موثوقة.',
  'litellm.insecure_confirm': 'أفهم المخاطرة وأريد تجاهل شهادة هذا الخادم.',
  'litellm.insecure_confirm_required': 'أكّد للمتابعة دون التحقق من الشهادة.',
  'litellm.save': 'حفظ والاتصال',
  'litellm.saving': 'جارٍ الحفظ…',
  'litellm.saved': 'تم حفظ الإعدادات. جارٍ قراءة الخادم…',
  'litellm.edit': 'تعديل الاتصال',
  'litellm.delay_note':
    'قد يتأخر الإنفاق وإعادة الضبط نحو 10 دقائق على الخادم. وقد يكون السقف على المستخدم أو الفريق، لا على المفتاح فقط.',
} as const satisfies Record<string, string>;
