// One title and one body for every ErrorCode (src/lib/contract/view.ts), plus the OpenCode
// ambiguity wording. src/i18n/catalog.test.ts fails when a code is missing.
export const errorMessages = {
  'error.internal.title': 'Algo deu errado',
  'error.internal.body':
    'Ocorreu um erro interno ao ler esta fonte. Tente de novo; se continuar, reinicie o monitor.',
  'error.timeout.title': 'A leitura demorou demais',
  'error.timeout.body': 'A fonte não respondeu a tempo. Tente de novo em instantes.',
  'error.network.title': 'Sem conexão com a fonte',
  'error.network.body': 'Não foi possível falar com o servidor. Confira a rede e tente de novo.',
  'error.bad_response.title': 'Resposta inesperada',
  'error.bad_response.body':
    'A fonte respondeu algo que o monitor não entende. Tente de novo mais tarde.',
  'error.response_too_large.title': 'Resposta grande demais',
  'error.response_too_large.body':
    'A resposta passou do tamanho aceito e foi descartada por segurança.',
  'error.rate_limited.title': 'Muitas consultas',
  'error.rate_limited.body':
    'A fonte pediu para esperar um pouco. O monitor tenta de novo sozinho, com intervalo maior.',
  'error.unauthorized.title': 'Chave ou login recusado',
  'error.unauthorized.body':
    'O servidor recusou as credenciais. Confira a chave e o escopo nas configurações.',
  'error.unauthorized_ambiguous.title': 'Não foi possível verificar a chave ou a assinatura Go',
  'error.unauthorized_ambiguous.body':
    'O serviço recusou a consulta. Isso acontece com chaves válidas e também sem a assinatura Go. Confira a chave e a assinatura no console do OpenCode.',
  'error.unauthorized_ambiguous.kimi.title': 'Não foi possível verificar a chave do Kimi Code',
  'error.unauthorized_ambiguous.kimi.body':
    'O serviço recusou a consulta. Isso acontece com chaves vencidas e também com a chave da plataforma aberta da Moonshot, que é outro produto. Confira no console do Kimi Code se a chave é do plano.',
  'error.unauthorized_ambiguous.zai.title': 'Não foi possível verificar a chave do z.ai',
  'error.unauthorized_ambiguous.zai.body':
    'O serviço recusou a consulta. Isso acontece com chaves válidas e também com contas sem o plano GLM Coding. Confira a chave e o plano no painel do z.ai.',
  'error.unauthorized_ambiguous.cursor.title': 'Não foi possível verificar o login do Cursor',
  'error.unauthorized_ambiguous.cursor.body':
    'O serviço recusou a consulta. Isso acontece com sessão vencida ou antiga e também com uma conta sem plano ativo. Abra o Cursor para renovar o login e confira o plano no painel do Cursor.',
  'error.unauthorized_ambiguous.copilot.title': 'Não foi possível verificar o token do GitHub',
  'error.unauthorized_ambiguous.copilot.body':
    'O GitHub recusou a consulta. Isso acontece com token vencido, token de granularidade fina ou de aplicativo e conta sem Copilot. Confira o token e a assinatura do Copilot nas configurações do GitHub.',
  'error.not_found.title': 'Endereço não encontrado',
  'error.not_found.body':
    'O servidor não conhece o caminho ou o ID informado. Confira a URL, o escopo e o ID.',
  'error.server_error.title': 'Erro no servidor',
  'error.server_error.body': 'O servidor da fonte falhou. Tente de novo mais tarde.',
  'error.tls_error.title': 'Falha no certificado TLS',
  'error.tls_error.body':
    'O certificado do servidor não foi aceito. Escolha a CA correta nas configurações.',
  'error.invalid_url.title': 'URL inválida',
  'error.invalid_url.body':
    'Use https://. HTTP só funciona para localhost, e a URL não pode ter usuário, senha, parâmetros ou #.',
  'error.invalid_input.title': 'Valor inválido',
  'error.invalid_input.body': 'Corrija o campo indicado e tente de novo.',
  'error.settings_unreadable.title': 'Arquivo de configurações ilegível',
  'error.settings_unreadable.body':
    'Não foi possível ler o settings.json. Nada foi alterado. Corrija o arquivo e tente de novo.',
  'error.install_failed.title': 'Falha ao instalar',
  'error.install_failed.body':
    'Não foi possível instalar o comando na barra de status do Claude Code. Nada foi alterado.',
  'error.unsupported_plan.title': 'Plano sem limites para ler',
  'error.unsupported_plan.body':
    'A conta usa um plano ou uma chave de API que não informa limites. Você pode desligar esta fonte.',
  'error.oauth_expired.title': 'Login do Claude Code expirou',
  'error.oauth_expired.body': 'Abra o Claude Code para renovar o login e tente de novo.',
  'error.binary_not_found.title': 'Programa não encontrado',
  'error.binary_not_found.body':
    'O monitor não achou o executável. Instale o programa ou informe o caminho nas configurações.',
  'error.spawn_failed.title': 'Não foi possível iniciar o programa',
  'error.spawn_failed.body': 'O processo não abriu. Confira o caminho do executável.',
  'error.handshake_failed.title': 'Falha ao conversar com o programa',
  'error.handshake_failed.body':
    'O programa abriu, mas não respondeu como esperado. Atualize-o e tente de novo.',
  'error.protocol_error.title': 'Formato inesperado',
  'error.protocol_error.body': 'O formato dos dados mudou. Atualize o monitor ou o programa.',
  'error.not_signed_in.title': 'Conta não conectada',
  'error.not_signed_in.body': 'Entre na sua conta no programa e tente de novo.',
  'error.not_running.title': 'Programa fechado',
  'error.not_running.body': 'Esta leitura precisa do programa aberto. Abra-o e tente de novo.',
  'error.endpoint_changed.title': 'A fonte mudou de formato',
  'error.endpoint_changed.body':
    'O serviço mudou e esta leitura não funciona mais como antes. Pode ser preciso atualizar o monitor.',
  'error.http': 'Código HTTP: {httpStatus}.',
} as const satisfies Record<string, string>;
