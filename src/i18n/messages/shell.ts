// Shell chrome: header, tabs, footer, live announcements, panel states.
export const shellMessages = {
  'app.title': 'AI Session Monitor',
  'harness.claude': 'Claude Code',
  'harness.codex': 'Codex',
  'harness.antigravity': 'Antigravity',
  'harness.opencode': 'OpenCode',
  'harness.kimi': 'Kimi Code',
  'harness.zai': 'z.ai',
  'harness.litellm': 'LiteLLM',

  'header.refresh': 'Atualizar agora',
  'header.settings': 'Configurações',
  'header.tray': 'Menor disponível: {pct}',
  'header.tray_used': 'Maior uso: {pct}',
  'header.tray_none': 'Sem leitura disponível',

  'tabs.label': 'Ferramentas de IA',
  'tab.connect': 'Conectar',
  'tab.no_data': 'Sem dados',
  'tab.disabled': 'Desligado',
  'tab.error': 'Falha',
  'tab.stale': 'Antigo',
  'tab.expired': 'Reiniciou',
  'tab.unknown': '—',
  'tab.no_windows': 'Sem cota',
  'tab.sr.pct': '{pct} disponível, {status}',
  'tab.sr.pct_used': '{pct} usado, {status}',
  'tab.sr.connect': 'precisa ser conectado',
  'tab.sr.no_data': 'sem dados ainda',
  'tab.sr.disabled': 'desligado',
  'tab.sr.error': 'falha na leitura',
  'tab.sr.stale': 'dados desatualizados',
  'tab.sr.expired': 'a janela reiniciou, aguardando nova leitura',
  'tab.sr.unknown': 'percentual desconhecido',
  'tab.sr.no_windows': 'sem janelas de cota',
  'tab.sr.loading': 'carregando',

  'status.ok': 'dentro do ritmo',
  'status.warn': 'atenção',
  'status.crit': 'crítico',
  'status.out': 'esgotado',

  'footer.read_ago': 'Lido há {age}',
  'footer.never': 'Ainda sem leitura',
  'footer.refresh': 'Atualizar',
  'footer.quit': 'Sair',

  'live.warn': '{name}: atenção, {pct} disponível.',
  'live.warn_used': '{name}: atenção, {pct} usado.',
  'live.crit': '{name}: crítico, {pct} disponível.',
  'live.crit_used': '{name}: crítico, {pct} usado.',
  'live.out': '{name}: esgotado.',
  'live.ok': '{name}: {pct} disponível, sem alertas.',
  'live.ok_used': '{name}: {pct} usado, sem alertas.',
  'live.error': '{name}: falha na leitura.',
  'live.stale': '{name}: dados desatualizados.',

  'boundary.title': 'A interface falhou',
  'boundary.body':
    'Algo inesperado aconteceu ao desenhar esta tela. Seus dados não foram alterados.',
  'boundary.retry': 'Recarregar a tela',
  'load.title': 'Não foi possível carregar o monitor',
  'load.events': 'Não foi possível acompanhar as atualizações. Os números podem ficar parados.',

  'disabled.title': '{name} está desligado',
  'disabled.body': 'Ative a fonte para voltar a ler os limites.',
  'disabled.enable': 'Ativar',
  'disabled.settings': 'Abrir configurações',

  'loading.label': 'Carregando a primeira leitura',

  'no_data.title': 'Aguardando a primeira leitura',
  'no_data.body.claude':
    'Comando instalado. Abra o Claude Code e envie uma mensagem: os limites só chegam depois da primeira resposta de uma sessão.',
  'no_data.body.default':
    'A fonte está funcionando, mas ainda não informou nada. Tente de novo em instantes.',
  'no_data.retry': 'Ler agora',

  'stale.banner': 'Mostrando os últimos números lidos, de {age} atrás.',
  'stale.banner_noage': 'Mostrando os últimos números lidos.',
  'stale.failed': 'A última atualização falhou: {title}.',

  'cooldown.next': 'A fonte pediu um intervalo entre as consultas. Nova tentativa em {wait}.',
  'cooldown.refresh_blocked':
    '{name}: a fonte pediu um intervalo, então a leitura manual espera. Nova tentativa em {wait}.',

  'action.retry': 'Tentar de novo',
  'action.settings': 'Abrir configurações',
  'action.console.opencode': 'Abrir console do OpenCode',
  'action.console.kimi': 'Abrir console do Kimi Code',
  'action.console.zai': 'Abrir chaves do z.ai',
  'action.reinstall': 'Reinstalar',
  'action.failed': 'Não foi possível concluir a ação.',
} as const satisfies Record<string, string>;
