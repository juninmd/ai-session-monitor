// Portuguese (Brazil) catalog: flyout (onboarding, command palette, provider detail, history board).
export const flyoutMessages = {
  // Onboarding wizard
  'onboarding.step.choose': 'Escolher',
  'onboarding.step.connect': 'Conectar',
  'onboarding.step.verify': 'Validar',
  'onboarding.step.done': 'Pronto',
  'onboarding.choose.title': 'O que você quer acompanhar?',
  'onboarding.choose.note':
    'Os itens marcados foram encontrados nesta máquina. Nada é lido antes de você avançar.',
  'onboarding.tag.responds': 'já responde',
  'onboarding.tag.silent': 'sem leitura',
  'onboarding.tag.needs_key': 'precisa de chave',
  'onboarding.tag.needs_consent': 'precisa de permissão',
  'onboarding.connect.title': 'Credenciais e permissões',
  'onboarding.connect.consent_note':
    'Esta fonte lê arquivos ou processos deste PC. Nada é lido antes do seu ok.',
  'onboarding.connect.allow': 'Permitir leitura',
  'onboarding.connect.details': 'Ver detalhes',
  'onboarding.connect.litellm_note': 'Precisa do endereço do proxy, do escopo e da chave.',
  'onboarding.connect.multi_note':
    'Tem mais de uma forma de conectar: chave ou leitura do banco local.',
  'onboarding.connect.configure': 'Configurar',
  'onboarding.connect.nothing': 'Nada a conectar: as fontes escolhidas já têm o que precisam.',
  'onboarding.verify.title': 'O que cada fonte respondeu',
  'onboarding.verify.reading': 'Lendo…',
  'onboarding.verify.read_now': 'Ler agora',
  'onboarding.verify.first.one': 'uma cota · a primeira em {pct} usado',
  'onboarding.verify.first.other': '{count} cotas · a primeira em {pct} usado',
  'onboarding.verify.back_to_key': 'Voltar e colar a chave',
  'onboarding.done.title': 'Monitorando {done} de {total}',
  'onboarding.done.body':
    'O painel abre ordenado por urgência: o que está perto de acabar aparece primeiro. O que não respondeu fica em "Sem leitura", sem inventar número.',
  'onboarding.done.go': 'Ir para o painel',
  'onboarding.done.again': 'Ler de novo',
  'onboarding.back': 'Voltar',
  'onboarding.continue': 'Continuar com {count}',
  'onboarding.connected': 'Já conectei',
  'onboarding.finish': 'Concluir',
  'onboarding.outcome.loading': 'Lendo…',
  'onboarding.outcome.disabled': 'Desligado',
  'onboarding.outcome.needs_consent': 'Aguardando sua permissão',
  'onboarding.outcome.needs_setup': 'Falta a credencial',
  'onboarding.outcome.no_data': 'Ainda sem dados',
  'onboarding.outcome.failed': 'Falha na leitura',
  'onboarding.outcome.stale': 'Leitura antiga',
  'onboarding.outcome.stale_ago': 'Leitura antiga há {age}',
  'onboarding.outcome.no_windows': 'Lendo valores, sem cotas',
  'onboarding.outcome.quotas.one': '{count} cota',
  'onboarding.outcome.quotas.other': '{count} cotas',

  // Ctrl+K palette
  'palette.group.providers': 'Providers',
  'palette.group.actions': 'Ações',
  'palette.group.settings': 'Configurações',
  'palette.group.demo': 'Demonstração',
  'palette.refresh.label': 'Atualizar todas as fontes',
  'palette.refresh.hint': 'releitura imediata',
  'palette.refresh.keywords': 'refresh atualizar recarregar',
  'palette.copy.label': 'Copiar resumo do estado',
  'palette.copy.hint.one': '{count} fonte em texto',
  'palette.copy.hint.other': '{count} fontes em texto',
  'palette.copy.keywords': 'clipboard copiar status relatorio',
  'palette.general.label': 'Configurações gerais',
  'palette.general.hint': 'exibição, limites, bandeja',
  'palette.general.keywords': 'preferencias opcoes',
  'palette.configure.label': 'Configurar {name}',
  'palette.configure.hint_stale': 'leitura antiga',
  'palette.configure.hint_default': 'fonte e credenciais',
  'palette.configure.keywords': 'configurar ajustar fonte credencial',
  'palette.scenario.label': 'Cenário: {name}',
  'palette.scenario.hint': 'dados fictícios',
  'palette.scenario.keywords': 'mock demo dados',
  'palette.close_aria': 'Fechar o comando rápido',
  'palette.dialog_aria': 'Comando rápido',
  'palette.input_aria': 'Buscar comando',
  'palette.placeholder': 'Ir para provider, atualizar, copiar resumo…',
  'palette.empty': 'Nada encontrado',
  'palette.foot.navigate': 'navegar',
  'palette.foot.run': 'executar',

  // Provider actions (preview only)
  'provider_actions.outcome.reset': 'Reset simulado. Cotas e saldo atualizados na prévia.',
  'provider_actions.outcome.noCredit':
    'Nenhum reset disponível. O saldo mudou; atualize a leitura.',
  'provider_actions.outcome.nothingToReset': 'Não há cota a renovar na prévia.',
  'provider_actions.outcome.unavailable': 'A ação não está disponível nesta leitura.',
  'provider_actions.resets.one': '{count} reset disponível',
  'provider_actions.resets.other': '{count} resets disponíveis',
  'provider_actions.use_reset': 'Usar reset',
  'provider_actions.confirm_aria': 'Confirmar reset de cota',
  'provider_actions.confirm_body':
    'Usar 1 reset do Codex? O consumo é definitivo. Nesta prévia, a ação é simulada.',
  'provider_actions.confirm': 'Confirmar reset',
  'provider_actions.cancel': 'Cancelar',
  'provider_actions.reset_failed': 'Não foi possível simular o reset. Tente novamente.',
  'provider_actions.low_priority_offer': 'Oferta de baixa prioridade · prévia',
  'provider_actions.copy_command': 'Copiar /low-priority',
  'provider_actions.low_priority_hint':
    'Use {command} na sessão interativa que ofereceu esse modo.',
  'provider_actions.copied': 'Comando copiado. O modo ainda precisa ser ativado no Claude Code.',
  'provider_actions.copy_failed':
    'Não foi possível copiar. Selecione o comando acima e copie manualmente.',

  // History board and trend
  'history.board.no_readable': 'Nenhuma cota desta fonte guarda uma leitura com hora de renovação.',
  'history.board.failed': 'Não foi possível ler o histórico.',
  'history.board.loading': 'Carregando as leituras…',
  'history.board.empty': 'Ainda não há leituras guardadas para esta cota.',
  'history.trend.none': 'Sem leitura nesta janela.',
  'history.trend.first': 'Pico de {peak} até agora, sem renovação anterior para comparar.',
  'history.trend.compare.one':
    'Pico de {peak} até agora · a renovação anterior fechou em média em {avg}.',
  'history.trend.compare.other':
    'Pico de {peak} até agora · as {count} renovações anteriores fecharam em média em {avg}.',
  'history.trend.projection_desc': 'A projeção segue o ritmo atual até acabar em {duration}.',

  // Quota sheet
  'quota.status.ok': 'tranquilo',
  'quota.status.warn': 'no limite',
  'quota.status.crit': 'crítico',
  'quota.status.out': 'esgotado',
  'quota.amount.unlimited': '{used} gastos, sem teto',
  'quota.amount.of': '{used} de {limit}',
  'quota.amount.free': '{free} livres',
  'quota.renew.restarted': 'reiniciou',
  'quota.renew.restarted_at': 'reiniciou às {time}',
  'quota.renew.in': 'renova em {duration}',
  'quota.spoken_left': '{duration} até renovar.',
  'quota.reading.expired': 'Expirou',
  'quota.reading.stale': 'Antigo',
  'quota.spoken_stale': '{name}: leitura desatualizada.',
  'quota.forecast.out': 'Esgotado',
  'quota.forecast.risk': 'Esgota em {duration}',
  'quota.forecast.fine': 'Dura até renovar',
  'quota.forecast.unknown': 'Sem base para prever',

  // Provider detail drawer
  'drawer.close': 'Fechar os detalhes',
  'drawer.refresh': 'Atualizar {name}',
  'drawer.pin': 'Fixar {name} no topo',
  'drawer.unpin': 'Desafixar {name}',
  'drawer.settings': 'Configurar {name}',

  // Candidate sources (not wired yet)
  'candidates.amp.credential': 'CLI local (`amp`)',
  'candidates.amp.offer':
    '`amp usage` devolve saldo de créditos e uso incluído. Mesmo padrão de CLI fixo já usado por Claude e Antigravity.',
  'candidates.copilot.credential': 'PAT com escopo de leitura de plano',
  'candidates.copilot.offer':
    'Requisições premium, chat e completions, com plano e data de reset; faturamento de créditos de IA pela API REST de billing.',
  'candidates.kiro.credential': 'Credencial do próprio CLI',
  'candidates.kiro.offer':
    'GetUsageLimits devolve um único pool de créditos com usado, limite, percentual e reset.',
  'candidates.cline.credential': 'Chave de API',
  'candidates.cline.offer':
    '`/users/me/plan/usage-limits`: janelas de 5 h, semanal e mensal com percentual e resetsAt.',
  'candidates.windsurf.credential': 'Service key da equipe',
  'candidates.windsurf.offer':
    'GetTeamCreditBalance e GetUsageConfig pela API oficial. Só no plano Enterprise.',
  'candidates.command_code.credential': 'CLI local (`commandcode`)',
  'candidates.command_code.offer':
    '`/alpha/billing/credits` e `/subscriptions`: janelas de gasto de 5 h e semanais, plano e créditos mensais.',
} as const;
