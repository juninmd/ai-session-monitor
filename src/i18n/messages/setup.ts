// First-run connection surfaces: consent cards, secret fields, OpenCode and LiteLLM forms.
export const setupMessages = {
  'consent.skip': 'Agora não',
  'consent.details': 'O que será lido',

  'claude.connect.title': 'Conectar ao Claude Code',
  'claude.connect.lead':
    'O monitor adiciona um comando pequeno à barra de status do Claude Code. Ele guarda só os percentuais de limite (5 horas e semana) num arquivo local. Nenhum login é lido.',
  'claude.connect.detail':
    'Edita o settings.json do Claude Code, com cópia de segurança, e encadeia a sua barra de status atual. Grava apenas os limites em %LOCALAPPDATA%. Sem rede e sem tokens. Funciona em contas Pro e Max. Os números aparecem depois da primeira resposta de uma sessão e só atualizam com o Claude Code aberto.',
  'claude.connect.allow': 'Instalar comando',
  'claude.connect.tip':
    'Dica: o Windows esconde ícones novos no menu da seta. Arraste o ícone do monitor para a barra para deixá-lo sempre à vista.',

  'codex.connect.title': 'Conectar ao Codex',
  'codex.connect.lead':
    'O monitor inicia o codex app-server instalado aqui e pergunta quanto resta da janela de 5 horas e da semana.',
  'codex.connect.detail':
    'Roda o programa codex da sua máquina, só leitura. Nenhum dado sai deste computador e o app não toca no arquivo de login do Codex. A OpenAI marca o app-server como experimental, então o formato pode mudar entre versões.',
  'codex.connect.allow': 'Permitir leitura',

  'antigravity.connect.title': 'Conectar ao Antigravity',
  'antigravity.connect.lead':
    'A cota vem do agy (CLI) instalado ou do servidor local da IDE. Esse caminho não é oficial e pode parar de funcionar quando o Antigravity atualizar.',
  'antigravity.connect.detail':
    'Com sua permissão, o monitor pode rodar o agy instalado só para ler /usage (sem gastar tokens nem ler dados de login) e/ou ler a linha de comando do servidor da IDE para pegar o token de sessão local, falando só com 127.0.0.1. Nada é gravado em disco nem enviado à internet.',
  'antigravity.connect.allow': 'Permitir leitura local',

  'shim.title': 'Comando na barra de status',
  'shim.state.not_installed': 'Não instalado',
  'shim.state.installed': 'Instalado',
  'shim.state.installed_chained': 'Instalado, encadeado com a sua barra de status',
  'shim.state.foreign': 'Outra barra de status em uso',
  'shim.state.outdated': 'Desatualizado: o executável não foi encontrado',
  'shim.state.unreadable': 'settings.json ilegível',
  'shim.hint.foreign':
    'Ao instalar, a sua barra de status atual continua funcionando: o monitor a encadeia.',
  'shim.hint.unreadable':
    'O monitor não mexe num arquivo que não consegue ler. Corrija o settings.json e verifique de novo.',
  'shim.path': 'Arquivo: {path}',
  'shim.install': 'Instalar comando',
  'shim.reinstall': 'Reinstalar comando',
  'shim.uninstall': 'Remover comando',
  'shim.recheck': 'Verificar de novo',
  'shim.loading': 'Verificando o Claude Code',

  'secret.saved': 'Chave salva',
  'secret.not_saved': 'Sem chave salva',
  'secret.placeholder': 'Cole a chave',
  'secret.placeholder_keep': 'Deixe em branco para manter a chave salva',
  'secret.hint': 'Fica no Gerenciador de Credenciais do Windows e nunca aparece na interface.',
  'secret.save': 'Salvar chave',
  'secret.clear': 'Remover chave',
  'secret.empty': 'Cole a chave de API.',
  'secret.saved_ok': 'Chave salva.',
  'secret.cleared_ok': 'Chave removida.',

  'opencode.title': 'Conectar ao OpenCode Go',
  'opencode.lead':
    'Cole a chave de API do OpenCode Go para ler as janelas reais (5 horas, semana e mês). Sem a chave, você pode ver só uma estimativa local de consumo.',
  'opencode.key_label': 'Chave de API do OpenCode Go',
  'opencode.console': 'Abrir o console do OpenCode Go',
  'opencode.billing': 'Ver cobrança do Zen',
  'opencode.zen_note': 'O saldo do Zen (pré-pago) não tem API: veja no painel de cobrança.',
  'opencode.local_label': 'Estimativa local (só este PC)',
  'opencode.local_hint':
    'Abre o banco do OpenCode em modo somente leitura e soma só custo e tokens, a preço de tabela. Não é a cota real.',
  'opencode.data_dir': 'Pasta de dados do OpenCode',
  'opencode.data_dir_hint': 'Deixe em branco para usar a pasta padrão.',
  'opencode.save_settings': 'Salvar pasta',

  'litellm.title': 'Conectar ao LiteLLM',
  'litellm.title_edit': 'Conexão com o LiteLLM',
  'litellm.url': 'URL do servidor',
  'litellm.url_placeholder': 'https://litellm.suaempresa.com',
  'litellm.url_hint': 'HTTPS é obrigatório. HTTP só é aceito para localhost.',
  'litellm.url_required': 'Informe a URL do servidor.',
  'litellm.url_invalid': 'Informe uma URL completa, como https://litellm.suaempresa.com.',
  'litellm.url_http': 'Use https://. HTTP só funciona para localhost.',
  'litellm.url_parts': 'Remova usuário, senha, parâmetros e # da URL.',
  'litellm.key': 'Chave de API',
  'litellm.key_required': 'Cole a chave de API do LiteLLM.',
  'litellm.key_rebind':
    'A chave salva vale só para a conexão atual. Como ela mudou, cole a chave de novo.',
  'litellm.scope': 'O que medir',
  'litellm.scope.key': 'Esta chave',
  'litellm.scope.user': 'Usuário',
  'litellm.scope.team': 'Time',
  'litellm.entity.user': 'ID do usuário',
  'litellm.entity.team': 'ID do time',
  'litellm.entity_required': 'Informe o ID.',
  'litellm.advanced': 'Avançado',
  'litellm.auth_header': 'Cabeçalho de autenticação',
  'litellm.auth_header_hint': 'Mude só se o seu proxy usa um cabeçalho próprio para a chave.',
  'litellm.auth_header_invalid': 'Use só letras, números e hífen.',
  'litellm.poll': 'Atualizar a cada',
  'litellm.timeout': 'Tempo limite',
  'litellm.tls': 'Certificado TLS',
  'litellm.tls.system': 'Do Windows',
  'litellm.tls.custom_ca': 'CA própria',
  'litellm.tls.insecure': 'Não verificar',
  'litellm.pem': 'Arquivo da CA (.pem)',
  'litellm.pem_placeholder': 'C:\\certs\\ca.pem',
  'litellm.pem_required': 'Informe o caminho do arquivo .pem.',
  'litellm.insecure_host': 'Servidor sem verificação',
  'litellm.insecure_host_hint': 'O certificado só deixa de ser verificado para este servidor.',
  'litellm.insecure_host_required': 'Informe o servidor.',
  'litellm.insecure_warning':
    'A conexão não será verificada e a chave pode ser interceptada. Use só em rede confiável.',
  'litellm.insecure_confirm': 'Entendo o risco e quero ignorar o certificado deste servidor.',
  'litellm.insecure_confirm_required': 'Confirme para continuar sem verificar o certificado.',
  'litellm.save': 'Salvar e conectar',
  'litellm.saving': 'Salvando…',
  'litellm.saved': 'Configuração salva. Lendo o servidor…',
  'litellm.edit': 'Editar conexão',
  'litellm.delay_note':
    'Gasto e reset podem atrasar até cerca de 10 min no servidor. O teto pode estar no usuário ou no time, não só na chave.',
} as const satisfies Record<string, string>;
