// Settings view: general options and the per-source groups.
export const settingsMessages = {
  'settings.title': 'Configurações',
  'settings.back': 'Voltar',
  'settings.general': 'Geral',
  'settings.interval': 'Atualizar a cada',
  'settings.autostart': 'Iniciar com o Windows',
  'settings.tray_source': 'Ícone da bandeja mostra',
  'settings.tray_source_hint': 'A cor e o arco do ícone seguem essa leitura.',
  'settings.tray_lowest': 'Menor disponível',
  'settings.time_format': 'Formato do tempo',
  'settings.time_hm': '1h 53m',
  'settings.time_colon': '1:53',
  'settings.time_decimal': '1,9 h',
  'settings.week_format': 'Semana em',
  'settings.week_hours': 'horas',
  'settings.week_days_hours': 'dias e horas',
  'settings.theme': 'Tema',
  'settings.theme_hint': 'Segue o Windows.',
  'settings.alerts': 'Alertas',
  'settings.warn_at': 'Avisar quando restar até',
  'settings.warn_at_hint': 'Abaixo disso a janela fica em atenção.',
  'settings.notify_out': 'Avisar ao esgotar',
  'settings.notify_reset': 'Avisar quando a janela resetar',
  'settings.sources': 'Fontes de dados',
  'settings.enabled': 'Ler esta fonte',
  'settings.unofficial_chip': 'não oficial',
  'settings.on': 'Ligado',
  'settings.off': 'Desligado',
  'settings.secrets_note':
    'Tokens e chaves ficam no Gerenciador de Credenciais do Windows e nunca chegam à interface.',
  'settings.secrets_unreadable':
    'Não foi possível ler o Gerenciador de Credenciais do Windows. As chaves salvas não aparecem aqui e salvar uma nova pode falhar. O resto das configurações continua funcionando.',
  'settings.help': 'Como cada fonte funciona',
  'settings.loading': 'Carregando as configurações',
  'settings.saved': 'Salvo.',

  'claude.oauth.label': 'API de uso da conta',
  'claude.oauth.chip': 'Também: API de uso, não oficial',
  'claude.oauth.hint':
    'Mostra Opus e Sonnet por modelo. Usa o login local numa API sem documentação e sem autorização explícita nos termos da Anthropic. Desligada por padrão.',
  'claude.oauth.warning':
    'Atenção: esta API não é documentada e os termos da Anthropic não autorizam acessos automatizados fora da API oficial. A conta pode ser limitada ou bloqueada. Ative só se aceitar esse risco.',
  'claude.oauth.confirm': 'Ativar mesmo assim',
  'claude.oauth.cancel': 'Cancelar',
  'claude.config_dir': 'Pasta de configuração do Claude Code',
  'claude.config_dir_hint':
    'Use o caminho completo da pasta. Deixe em branco para seguir a variável CLAUDE_CONFIG_DIR (lida quando o monitor abre) ou, sem ela, a pasta .claude do seu perfil.',
  'claude.config_dir_placeholder': 'C:\\Users\\voce\\.claude-trabalho',
  'claude.config_dir_save': 'Salvar pasta do Claude Code',
  'claude.locked.title': 'Semana por modelo',
  'claude.locked.body':
    'Opus e Sonnet só aparecem pela API de uso da conta. Ela não é oficial e está desligada.',

  'codex.consent': 'Permitir que o monitor inicie o codex app-server',
  'codex.consent_hint': 'Roda o programa codex da sua máquina, só leitura.',
  'codex.binary': 'Caminho do executável do Codex',
  'codex.binary_hint': 'Deixe em branco para procurar o codex no PATH.',
  'codex.binary_placeholder': 'C:\\Users\\voce\\AppData\\Roaming\\npm\\codex.cmd',
  'codex.binary_save': 'Salvar caminho',

  'antigravity.consent': 'Permitir a leitura do Antigravity (CLI ou IDE)',
  'antigravity.consent_hint':
    'Leitura não oficial: pode rodar o agy instalado só para ler /usage (sem gastar tokens nem ler dados de login) e/ou ler o servidor local da IDE, que só conversa com 127.0.0.1.',
  'antigravity.note':
    'A leitura vem do agy instalado (nada precisa estar aberto) ou, sem ele, do servidor local da IDE: aí mantenha o Antigravity aberto.',
} as const satisfies Record<string, string>;
