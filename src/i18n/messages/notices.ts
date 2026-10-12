// Notice codes from docs/ipc.md. An unknown code renders `notice.unknown` with the code in it,
// so a missing translation is visible instead of silent.
export const noticeMessages = {
  'notice.code.claude_waiting_first_session':
    'Comando instalado. Os limites chegam depois da primeira resposta de uma sessão do Claude Code.',
  'notice.code.claude_no_rate_limits':
    'O Claude Code não informa limites (cobrança por chave de API ou plano sem limites).',
  'notice.code.claude_shim_outdated':
    'O comando instalado na barra de status do Claude Code não foi encontrado. Reinstale para voltar a ler.',
  'notice.code.claude_oauth_partial':
    'A API de uso da conta está ligada, mas falhou ({reason}). Mostrando só os dados da barra de status.',
  'notice.code.codex_rollout_fallback':
    'O codex app-server não respondeu ({reason}). Lendo a última sessão gravada; os números podem estar antigos.',
  'notice.code.codex_no_limits': 'A conta do Codex não informa limites (login por chave de API).',
  'notice.code.antigravity_pool_unknown':
    'A cota {pool} não retornou dados. Ela aparece como desconhecida, não como zero.',
  'notice.code.litellm_no_budget': 'Esta chave, usuário ou time não tem teto de orçamento.',
  'notice.code.litellm_level_unreadable':
    'Não foi possível ler o orçamento {level} ({reason}). Pode haver um teto ali que não aparece aqui.',
  'notice.code.litellm_windows_truncated':
    'Há mais janelas de orçamento do que cabem aqui. Ficaram de fora as menos consumidas: {dropped}.',
  'notice.code.opencode_window_idle':
    'A janela {window} está em 0%: ela só começa a contar na próxima requisição.',
  'notice.code.opencode_rate_limited': 'A janela {window} está limitada (rate-limited).',
  'notice.code.opencode_no_key':
    'Mostrando só a estimativa local. Cole a chave de API do Go para ver as janelas reais.',
  'notice.code.opencode_local_estimate':
    'Os valores locais são uma estimativa a preço de tabela, só deste PC. Não são a cota.',
  'notice.code.opencode_db_unreadable': 'Não foi possível ler o banco local do OpenCode: {reason}.',
  'notice.unknown': 'Aviso sem tradução: {code}',
  'notice.action.reinstall': 'Reinstalar',
  'notice.action.settings': 'Informar a chave',
  'notice.param.window.rolling': 'de 5 horas',
  'notice.param.window.weekly': 'semanal',
  'notice.param.window.monthly': 'mensal',
  'notice.param.level.key': 'da chave',
  'notice.param.level.user': 'do usuário',
  'notice.param.level.team': 'do time',
  'notice.param.dropped.one': '{n} janela',
  'notice.param.dropped.other': '{n} janelas',
  // The OpenCode database reasons are their own vocabulary: `not_found` there is a missing file.
  'notice.reason.opencode_db_unreadable.invalid_path': 'o caminho da pasta de dados é inválido',
  'notice.reason.opencode_db_unreadable.not_found': 'o arquivo do banco não foi encontrado',
  'notice.reason.opencode_db_unreadable.permission': 'sem permissão para abrir o arquivo do banco',
  'notice.reason.opencode_db_unreadable.locked': 'o banco está em uso por outro programa',
  'notice.reason.opencode_db_unreadable.not_a_database':
    'o arquivo encontrado não é um banco de dados',
  'notice.reason.opencode_db_unreadable.schema': 'o formato do banco mudou',
  'notice.reason.opencode_db_unreadable.timeout': 'a leitura do banco demorou demais',
  'notice.reason.opencode_db_unreadable.other': 'motivo não identificado',
  'notice.severity.info': 'Informação',
  'notice.severity.warn': 'Atenção',
  'notice.severity.error': 'Erro',
} as const satisfies Record<string, string>;
