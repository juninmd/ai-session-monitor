// First-run connection surfaces: consent cards, secret fields, OpenCode and LiteLLM forms.
export const setupMessages = {
  'consent.skip': 'Ahora no',
  'consent.details': 'Qué se leerá',

  'claude.connect.title': 'Conectar con Claude Code',
  'claude.connect.lead':
    'El monitor añade un pequeño comando a la línea de estado de Claude Code. Solo guarda los porcentajes de los límites (5 horas y semana) en un archivo local. No se lee ningún inicio de sesión.',
  'claude.connect.detail':
    'Edita el settings.json de Claude Code, con copia de seguridad, y encadena tu línea de estado actual. Solo escribe los límites en %LOCALAPPDATA%. Sin red y sin tokens. Funciona con cuentas Pro y Max. Las cifras aparecen tras la primera respuesta de una sesión y solo se actualizan mientras Claude Code está abierto.',
  'claude.connect.allow': 'Instalar comando',
  'claude.connect.tip':
    'Consejo: Windows oculta los iconos nuevos en el menú de la flecha de desbordamiento. Arrastra el icono del monitor a la barra de tareas para tenerlo siempre visible.',

  'codex.connect.title': 'Conectar con Codex',
  'codex.connect.lead':
    'El monitor inicia el app-server de codex instalado aquí y consulta cuánto queda de la ventana de 5 horas y de la semana.',
  'codex.connect.detail':
    'Ejecuta el programa codex en tu equipo, en modo de solo lectura. Ningún dato sale de este equipo y la aplicación no toca el archivo de inicio de sesión de Codex. OpenAI marca el app-server como experimental, así que el formato puede cambiar entre versiones.',
  'codex.connect.allow': 'Permitir lectura',

  'antigravity.connect.title': 'Conectar con Antigravity',
  'antigravity.connect.lead':
    'La cuota proviene del agy instalado (CLI) o del servidor local del IDE. Esta vía no es oficial y puede dejar de funcionar cuando Antigravity se actualice.',
  'antigravity.connect.detail':
    'Con tu permiso, el monitor puede ejecutar el agy instalado solo para leer /usage (sin gastar tokens ni leer datos de inicio de sesión) y/o leer la línea de comandos del servidor del IDE para obtener el token de sesión local, comunicándose únicamente con 127.0.0.1. No se escribe nada en el disco ni se envía nada a internet.',
  'antigravity.connect.allow': 'Permitir lectura local',

  'shim.title': 'Comando de la línea de estado',
  'shim.state.not_installed': 'No instalado',
  'shim.state.installed': 'Instalado',
  'shim.state.installed_chained': 'Instalado, encadenado con tu línea de estado',
  'shim.state.foreign': 'Se usa otra línea de estado',
  'shim.state.outdated': 'Desactualizado: no se encontró el ejecutable',
  'shim.state.unreadable': 'settings.json ilegible',
  'shim.hint.foreign':
    'Al instalar, tu línea de estado actual sigue funcionando: el monitor la encadena.',
  'shim.hint.unreadable':
    'El monitor no toca un archivo que no puede leer. Corrige settings.json y vuelve a comprobar.',
  'shim.path': 'Archivo: {path}',
  'shim.install': 'Instalar comando',
  'shim.reinstall': 'Reinstalar comando',
  'shim.uninstall': 'Quitar comando',
  'shim.recheck': 'Comprobar de nuevo',
  'shim.loading': 'Comprobando Claude Code',

  'secret.saved': 'Clave guardada',
  'secret.not_saved': 'Sin clave guardada',
  'secret.placeholder': 'Pega la clave',
  'secret.placeholder_keep': 'Déjalo en blanco para conservar la clave guardada',
  'secret.hint':
    'Se guarda en el Administrador de credenciales de Windows y nunca aparece en la interfaz.',
  'secret.save': 'Guardar clave',
  'secret.clear': 'Quitar clave',
  'secret.empty': 'Pega la clave de API.',
  'secret.saved_ok': 'Clave guardada.',
  'secret.cleared_ok': 'Clave quitada.',

  'opencode.title': 'Conectar con OpenCode Go',
  'opencode.lead':
    'Pega tu clave de API de OpenCode Go para leer las ventanas reales (5 horas, semana y mes). Sin la clave, solo se ve una estimación local del uso.',
  'opencode.key_label': 'Clave de API de OpenCode Go',
  'opencode.console': 'Abrir la consola de OpenCode Go',
  'opencode.billing': 'Ver la facturación de Zen',
  'opencode.zen_note': 'El saldo de Zen (prepago) no tiene API: consulta el panel de facturación.',
  'opencode.local_label': 'Estimación local (solo este PC)',
  'opencode.local_hint':
    'Abre la base de datos de OpenCode en modo de solo lectura y suma únicamente el costo y los tokens, a precio de lista. No es la cuota real.',
  'opencode.data_dir': 'Carpeta de datos de OpenCode',
  'opencode.data_dir_hint': 'Déjalo en blanco para usar la carpeta predeterminada.',
  'opencode.save_settings': 'Guardar carpeta',

  'kimi.title': 'Conectar con Kimi Code',
  'kimi.lead':
    'Pega tu clave de API de Kimi Code para leer la cuota del plan: la ventana de 5 horas, la semana y el total del mes, cuando el plan los informa.',
  'kimi.detail':
    'Lectura no oficial: la API de uso no está documentada y puede cambiar sin previo aviso. La clave se envía solo a api.kimi.com y se guarda en el Administrador de credenciales de Windows. No se usa ninguna cookie ni inicio de sesión de otra herramienta. Una clave de la plataforma abierta de Moonshot es otro producto y no funciona aquí.',
  'kimi.key_label': 'Clave de API de Kimi Code',
  'kimi.console': 'Abrir la consola de Kimi Code',
  'kimi.note':
    'Lectura no oficial de la API de uso de Kimi Code. La semana y el mes solo aparecen cuando el plan los informa.',

  'zai.title': 'Conectar con z.ai',
  'zai.lead':
    'Pega la clave de API de tu plan GLM Coding para leer la ventana de 5 horas, la semana y la cuota mensual de herramientas MCP.',
  'zai.detail':
    'Lectura no oficial: la API de cuota no está documentada y puede cambiar sin previo aviso. La clave se envía solo a api.z.ai y se guarda en el Administrador de credenciales de Windows. No se usa ninguna cookie ni inicio de sesión de otra herramienta. Las cuentas de BigModel (open.bigmodel.cn) no se leen.',
  'zai.key_label': 'Clave de API de z.ai',
  'zai.console': 'Abrir las claves de z.ai',
  'zai.note':
    'Lectura no oficial de la API de cuota del plan GLM Coding, en el servidor global (api.z.ai).',

  'openrouter.title': 'Conectar con OpenRouter',
  'openrouter.lead':
    'Pega tu clave de API de OpenRouter para ver el límite de gasto de la clave, el uso y, si la clave lo permite, el saldo de créditos.',
  'openrouter.detail':
    'Lectura oficial, mediante la API documentada de OpenRouter. La clave se envía solo a openrouter.ai y se guarda en el Administrador de credenciales de Windows. El saldo de créditos requiere una clave de administración; con una clave normal solo aparecen el límite de la clave y el uso.',
  'openrouter.key_label': 'Clave de API de OpenRouter',
  'openrouter.console': 'Abrir las claves de OpenRouter',
  'openrouter.note':
    'Lectura oficial de la API de OpenRouter. Sin un límite en la clave no hay porcentaje, solo uso. El saldo solo aparece con una clave de administración.',

  'deepseek.title': 'Conectar con DeepSeek',
  'deepseek.lead':
    'Pega tu clave de API de DeepSeek para ver el saldo de la cuenta, en cada moneda en la que tenga crédito.',
  'deepseek.detail':
    'Lectura oficial, mediante la API documentada de DeepSeek. La clave se envía solo a api.deepseek.com y se guarda en el Administrador de credenciales de Windows. El saldo no tiene límite ni hora de restablecimiento, así que no hay porcentaje; los yuanes y los dólares nunca se suman.',
  'deepseek.key_label': 'Clave de API de DeepSeek',
  'deepseek.console': 'Abrir las claves de DeepSeek',
  'deepseek.note':
    'Lectura oficial de la API de saldo de DeepSeek. Es dinero, no una cuota: no hay porcentaje ni restablecimiento.',

  'deepinfra.title': 'Conectar con DeepInfra',
  'deepinfra.lead':
    'Pega tu clave de API de DeepInfra para ver el saldo, el límite de gasto y el gasto del mes.',
  'deepinfra.detail':
    'Lectura oficial, mediante la API de facturación de DeepInfra. La clave se envía solo a api.deepinfra.com y se guarda en el Administrador de credenciales de Windows. Sin límite de gasto no hay porcentaje, solo importes en dólares.',
  'deepinfra.key_label': 'Clave de API de DeepInfra',
  'deepinfra.console': 'Abrir las claves de DeepInfra',
  'deepinfra.note':
    'Lectura oficial de la API de facturación de DeepInfra. Los valores se leen como dólares; sin límite de gasto no hay porcentaje.',

  'moonshot.title': 'Conectar con Moonshot',
  'moonshot.lead':
    'Pega tu clave de API de la plataforma Moonshot (platform.moonshot.ai) para ver el saldo de la cuenta.',
  'moonshot.detail':
    'Lectura oficial, mediante la API de saldo documentada de Moonshot. La clave se envía solo a api.moonshot.ai y se guarda en el Administrador de credenciales de Windows. Las claves de la plataforma kimi.com son independientes y no funcionan aquí. El saldo es dinero: no hay porcentaje ni restablecimiento.',
  'moonshot.key_label': 'Clave de API de Moonshot',
  'moonshot.console': 'Abrir las claves de Moonshot',
  'moonshot.note':
    'Lectura oficial de la API de saldo de Moonshot, en dólares. El saldo en efectivo puede ser negativo cuando se debe un importe.',

  'novita.title': 'Conectar con Novita',
  'novita.lead':
    'Pega tu clave de API de Novita para ver el saldo de la cuenta, el límite de crédito y los cargos pendientes.',
  'novita.detail':
    'Lectura oficial, mediante la API de facturación documentada de Novita. La clave se envía solo a api.novita.ai y se guarda en el Administrador de credenciales de Windows. El saldo es dinero: no hay porcentaje ni restablecimiento.',
  'novita.key_label': 'Clave de API de Novita',
  'novita.console': 'Abrir las claves de Novita',
  'novita.note':
    'Lectura oficial de la API de saldo de Novita, en dólares. El saldo puede ser negativo cuando se debe un importe.',

  'ollama.title': 'Conectar con Ollama Cloud',
  'ollama.lead':
    'Pega una clave de API de Ollama Cloud para ver los créditos incluidos del plan y el saldo comprado.',
  'ollama.detail':
    'Lectura oficial, mediante la API de saldo documentada de Ollama Cloud. Crea la clave en ollama.com/settings/keys: la clave de suscripción de la CLI nunca se lee. La clave se envía solo a ollama.com y se guarda en el Administrador de credenciales de Windows. El formato de la respuesta depende del plan (créditos mensuales o ventanas de sesión y semana).',
  'ollama.key_label': 'Clave de API de Ollama Cloud',
  'ollama.console': 'Abrir las claves de Ollama Cloud',
  'ollama.note':
    'Lectura oficial de la API de saldo de Ollama Cloud. Los planes actuales muestran créditos en dólares; los planes antiguos muestran porcentajes de sesión y semana.',

  'minimax.title': 'Conectar con MiniMax',
  'minimax.lead':
    'Pega tu clave del Token Plan de MiniMax para ver las cuotas por modelo: el intervalo actual y la semana.',
  'minimax.detail':
    'Lectura no oficial: la ruta de cuota del Token Plan no tiene un esquema publicado y puede cambiar sin previo aviso. La clave se envía solo a api.minimax.io (servidor global) y se guarda en el Administrador de credenciales de Windows. La primera vez, compara las cifras con la consola de MiniMax.',
  'minimax.key_label': 'Clave del Token Plan de MiniMax',
  'minimax.console': 'Abrir las claves de MiniMax',
  'minimax.note':
    'Lectura no oficial de la cuota del Token Plan de MiniMax, por modelo. Un modelo sin cuota en el plan no aparece.',

  'litellm.title': 'Conectar con LiteLLM',
  'litellm.title_edit': 'Conexión de LiteLLM',
  'litellm.url': 'URL del servidor',
  'litellm.url_placeholder': 'https://litellm.yourcompany.com',
  'litellm.url_hint': 'Se requiere HTTPS. HTTP solo se acepta para localhost.',
  'litellm.url_required': 'Introduce la URL del servidor.',
  'litellm.url_invalid': 'Introduce una URL completa, como https://litellm.yourcompany.com.',
  'litellm.url_http': 'Usa https://. HTTP solo funciona con localhost.',
  'litellm.url_parts':
    'Quita de la URL el usuario, la contraseña, los parámetros de consulta y el #.',
  'litellm.key': 'Clave de API',
  'litellm.key_required': 'Pega la clave de API de LiteLLM.',
  'litellm.key_rebind':
    'La clave guardada solo se aplica a la conexión actual. Como cambió, pega la clave de nuevo.',
  'litellm.scope': 'Qué medir',
  'litellm.scope.key': 'Esta clave',
  'litellm.scope.user': 'Usuario',
  'litellm.scope.team': 'Equipo',
  'litellm.entity.user': 'ID de usuario',
  'litellm.entity.team': 'ID de equipo',
  'litellm.entity_required': 'Introduce el ID.',
  'litellm.advanced': 'Avanzado',
  'litellm.auth_header': 'Encabezado de autenticación',
  'litellm.auth_header_hint': 'Cámbialo solo si tu proxy usa un encabezado propio para la clave.',
  'litellm.auth_header_invalid': 'Usa solo letras, números y guiones.',
  'litellm.poll': 'Actualizar cada',
  'litellm.timeout': 'Tiempo de espera',
  'litellm.tls': 'Certificado TLS',
  'litellm.tls.system': 'Del sistema',
  'litellm.tls.custom_ca': 'CA personalizada',
  'litellm.tls.insecure': 'No verificar',
  'litellm.pem': 'Archivo de CA (.pem)',
  'litellm.pem_placeholder': 'C:\\certs\\ca.pem',
  'litellm.pem_required': 'Introduce la ruta del archivo .pem.',
  'litellm.insecure_host': 'Servidor sin verificación',
  'litellm.insecure_host_hint': 'El certificado se omite solo para este servidor.',
  'litellm.insecure_host_required': 'Introduce el servidor.',
  'litellm.insecure_warning':
    'La conexión no se verificará y la clave podría ser interceptada. Úsalo solo en una red de confianza.',
  'litellm.insecure_confirm':
    'Entiendo el riesgo y quiero ignorar el certificado de este servidor.',
  'litellm.insecure_confirm_required': 'Confirma para continuar sin verificar el certificado.',
  'litellm.save': 'Guardar y conectar',
  'litellm.saving': 'Guardando…',
  'litellm.saved': 'Ajustes guardados. Leyendo el servidor…',
  'litellm.edit': 'Editar conexión',
  'litellm.delay_note':
    'El gasto y el restablecimiento pueden tardar hasta unos 10 min en reflejarse en el servidor. El límite puede estar en el usuario o el equipo, no solo en la clave.',
} as const satisfies Record<string, string>;
