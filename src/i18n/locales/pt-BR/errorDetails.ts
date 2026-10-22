// What the params of a failure add to its message: the `reason` a collector gives (closed
// vocabularies, worded per error code), the sentences for a redirect and a wait, and the names
// of the settings fields. A reason with a `.title` replaces the title of the code as well.
export const errorDetailMessages = {
  'error.install_failed.reason.status_line_too_large.body':
    'O comando atual da sua barra de status é grande demais para o monitor encadear. Nada foi alterado.',
  'error.install_failed.reason.path_unsupported.body':
    'O caminho da pasta de dados tem caracteres que o comando da barra de status não aceita. Nada foi alterado.',
  'error.install_failed.reason.serialize_failed.body':
    'Não foi possível montar o novo settings.json. Nada foi alterado.',
  'error.install_failed.reason.state_failed.body':
    'Não foi possível guardar o estado da instalação. O settings.json não foi alterado.',
  'error.install_failed.reason.concurrent_modification.body':
    'O settings.json mudou durante a operação, então o monitor não o alterou. Tente de novo.',
  'error.install_failed.reason.write_failed.body':
    'Não foi possível gravar o settings.json. O arquivo não foi alterado; confira se ele está liberado e tente de novo.',
  'error.install_failed.reason.shim_missing.body':
    'O executável do comando não foi encontrado junto do monitor. Reinstale o monitor.',
  'error.install_failed.reason.copy_failed.body':
    'Não foi possível copiar o executável do comando para a pasta de dados. O settings.json não foi alterado.',
  'error.install_failed.reason.backup_failed.body':
    'Não foi possível guardar a cópia de segurança do settings.json, então ele não foi alterado.',
  // Only a removal is refused for this reason.
  'error.install_failed.reason.foreign_status_line.title': 'Não foi possível remover o comando',
  'error.install_failed.reason.foreign_status_line.body':
    'A barra de status configurada não é a do monitor (talvez uma cópia editada à mão), então nada foi removido.',

  // LiteLLM: the CA file the user chose, not the certificate of the server.
  'error.tls_error.reason.ca_missing.body':
    'O arquivo da CA configurado não foi encontrado. Escolha o arquivo certo nas configurações.',
  'error.tls_error.reason.ca_too_large.body':
    'O arquivo da CA é grande demais para ser lido. Escolha um arquivo .pem menor nas configurações.',
  'error.tls_error.reason.ca_unreadable.body':
    'Não foi possível ler o arquivo da CA. Confira o caminho e a permissão nas configurações.',
  'error.tls_error.reason.ca_invalid.body':
    'O arquivo da CA não tem um certificado válido. Escolha um arquivo .pem válido nas configurações.',

  // LiteLLM: which part of the answer did not look like a key, user or team record.
  'error.endpoint_changed.reason.not_an_object.body':
    'A resposta do servidor não tem o formato esperado. Confira se a URL aponta para o LiteLLM.',
  'error.endpoint_changed.reason.no_record.body':
    'A resposta não traz o registro da chave, do usuário ou do time. Confira a URL e o escopo.',
  'error.endpoint_changed.reason.id_missing.body':
    'A resposta não confirma o ID consultado. Confira o ID do usuário ou do time.',
  'error.endpoint_changed.reason.id_mismatch.body':
    'A resposta é de outro ID, não do informado. Confira o ID do usuário ou do time.',

  // Antigravity: the IDE's local server is there, but there is nothing to call.
  'error.not_running.reason.no_port.title': 'Servidor local ainda não está pronto',
  'error.not_running.reason.no_port.body':
    'O Antigravity está aberto, mas o servidor local ainda não oferece um endereço. Isso costuma passar em instantes: tente de novo.',
  'error.unauthorized.reason.no_token.title': 'Token local indisponível',
  'error.unauthorized.reason.no_token.body':
    'O monitor não conseguiu ler o token de sessão do servidor local da IDE. Reinicie o Antigravity e tente de novo.',

  'error.timeout.reason.locked.body':
    'O banco de dados local do Cursor está em uso agora. Tente de novo em instantes.',
  'error.timeout.reason.timeout.body':
    'A leitura do banco de dados local do Cursor demorou demais. Tente de novo em instantes.',
  'error.endpoint_changed.reason.schema.body':
    'O banco de dados local do Cursor tem um formato diferente do que o monitor espera. O monitor pode precisar de uma atualização.',
  'error.settings_unreadable.reason.permission.body':
    'Sem permissão para abrir o banco de dados local do Cursor. Confira as permissões do arquivo.',
  'error.settings_unreadable.reason.not_a_database.body':
    'O arquivo do Cursor encontrado não é um banco de dados.',
  'error.settings_unreadable.reason.other.body':
    'Não foi possível ler o banco de dados local do Cursor.',
  'error.not_signed_in.reason.not_found.body':
    'Nenhum token do GitHub foi encontrado na pasta da extensão do Copilot. Entre no Copilot no seu editor ou cole um token nas configurações.',
  'error.not_signed_in.reason.no_token.body':
    'Os arquivos da extensão do Copilot não têm token para github.com. Entre no Copilot no seu editor ou cole um token nas configurações.',
  'error.not_signed_in.reason.unreadable.body':
    'Não foi possível ler o arquivo de login da extensão do Copilot. Confira as permissões da pasta dele.',

  'error.redirect':
    'O servidor tentou redirecionar a consulta, e o monitor não segue redirecionamentos.',
  'error.retry_after': 'Tempo pedido pela fonte: {wait}.',
  'error.field_hint': 'Campo com problema: {field}.',
  'field.value': 'chave de API',
  'field.baseUrl': 'URL do servidor',
  'field.entityId': 'ID do usuário ou do time',
  'field.authHeader': 'cabeçalho de autenticação',
  'field.pemPath': 'arquivo da CA (.pem)',
  'field.host': 'servidor sem verificação de certificado',
  'field.configDir': 'pasta de configuração do Claude Code',
  'field.binaryPath': 'caminho do executável do Codex',
  'field.dataDir': 'pasta de dados do OpenCode',
  'field.shimConsent': 'consentimento para instalar o comando',
  'field.limit': 'quantidade de janelas passadas',
} as const satisfies Record<string, string>;
