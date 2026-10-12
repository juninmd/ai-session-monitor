import { describe, expect, it } from 'vitest';
import { problem } from '../mock/fixtures';
import { describeError } from './errorText';

describe('describeError with the params collectors send', () => {
  const GENERIC_INSTALL = 'Falha ao instalar';

  it.each([
    ['install_failed', 'status_line_too_large', 'grande demais para o monitor encadear'],
    [
      'install_failed',
      'path_unsupported',
      'tem caracteres que o comando da barra de status não aceita',
    ],
    ['install_failed', 'serialize_failed', 'montar o novo settings.json'],
    ['install_failed', 'state_failed', 'guardar o estado da instalação'],
    ['install_failed', 'concurrent_modification', 'mudou durante a operação'],
    ['install_failed', 'write_failed', 'gravar o settings.json'],
    ['install_failed', 'shim_missing', 'não foi encontrado junto do monitor'],
    ['install_failed', 'copy_failed', 'copiar o executável do comando'],
    ['install_failed', 'backup_failed', 'cópia de segurança do settings.json'],
    ['tls_error', 'ca_missing', 'arquivo da CA configurado não foi encontrado'],
    ['tls_error', 'ca_too_large', 'grande demais para ser lido'],
    ['tls_error', 'ca_unreadable', 'Não foi possível ler o arquivo da CA'],
    ['tls_error', 'ca_invalid', 'não tem um certificado válido'],
    ['endpoint_changed', 'not_an_object', 'não tem o formato esperado'],
    ['endpoint_changed', 'no_record', 'não traz o registro da chave, do usuário ou do time'],
    ['endpoint_changed', 'id_missing', 'não confirma o ID consultado'],
    ['endpoint_changed', 'id_mismatch', 'é de outro ID'],
  ] as const)('says what %s meant by %s', (code, reason, fragment) => {
    const text = describeError(problem(code, { reason }));
    expect(text.body).toContain(fragment);
    expect(text.body).not.toBe(describeError(problem(code)).body);
    // The title keeps naming the family of the failure; only the body gets specific.
    expect(text.title).toBe(describeError(problem(code)).title);
    expect(`${text.title} ${text.body}`).not.toContain(reason);
  });

  it('keeps the CA problems apart from "the server certificate was refused"', () => {
    const generic = describeError(problem('tls_error')).body;
    expect(generic).toContain('certificado do servidor não foi aceito');
    expect(describeError(problem('tls_error', { reason: 'ca_missing' })).body).not.toContain(
      'certificado do servidor',
    );
  });

  describe('where the local server of the IDE is the problem', () => {
    it('says the server is still starting, not that the program is closed', () => {
      const text = describeError(
        problem('not_running', { reason: 'no_port' }, true),
        'antigravity',
      );
      expect(text.title).toBe('Servidor local ainda não está pronto');
      expect(text.body).toContain('O Antigravity está aberto');
      expect(text.action).toBe('retry');
      expect(describeError(problem('not_running', {}, true)).title).toBe('Programa fechado');
    });

    it('says the session token could not be read, not that a key was refused', () => {
      const text = describeError(problem('unauthorized', { reason: 'no_token' }), 'antigravity');
      expect(text.title).toBe('Token local indisponível');
      expect(text.body).toContain('Reinicie o Antigravity');
      expect(text.action).toBe('retry');
      expect(`${text.title} ${text.body}`).not.toMatch(/chave|configurações/i);
    });
  });

  it('says a removal was refused, not that an installation failed', () => {
    const text = describeError(problem('install_failed', { reason: 'foreign_status_line' }));
    expect(text.title).toBe('Não foi possível remover o comando');
    expect(text.body).toContain('não é a do monitor');
    expect(`${text.title} ${text.body}`).not.toContain(GENERIC_INSTALL);
  });

  it.each([
    ['install_failed', 'write_failed', 'reinstall'],
    ['install_failed', 'foreign_status_line', 'retry'],
    ['tls_error', 'ca_missing', 'settings'],
    ['endpoint_changed', 'not_an_object', 'settings'],
    ['endpoint_changed', 'no_record', 'settings'],
    ['endpoint_changed', 'id_missing', 'settings'],
    ['endpoint_changed', 'id_mismatch', 'settings'],
    ['endpoint_changed', 'something_new', 'retry'],
    ['unauthorized', 'no_token', 'retry'],
    ['unauthorized', 'rejected', 'settings'],
    ['not_running', 'no_port', 'retry'],
  ] as const)('offers the right way forward for %s / %s: %s', (code, reason, action) => {
    expect(describeError(problem(code, { reason })).action).toBe(action);
  });

  it.each([
    ['install_failed', 'solar_flare'],
    ['install_failed', ''],
    ['install_failed', '__proto__'],
    ['install_failed', 'constructor'],
    ['tls_error', 'x'.repeat(500)],
    ['network', 'ca_missing'],
    ['timeout', 'no_port'],
    ['server_error', 'write_failed'],
  ] as const)('does not echo or act on %s / %j, which it cannot word', (code, reason) => {
    const plain = describeError(problem(code));
    const text = describeError(problem(code, { reason }));
    expect(text).toEqual(plain);
    expect(JSON.stringify(text)).not.toContain('solar_flare');
  });

  it('lets the OpenCode ambiguity win over any reason', () => {
    const text = describeError(problem('unauthorized', { reason: 'no_token', ambiguous: 'true' }));
    expect(text.title).toBe('Não foi possível verificar a chave ou a assinatura Go');
    expect(text.action).toBe('console');
  });

  describe('details that come after the body', () => {
    it('says a redirect was refused', () => {
      const text = describeError(
        problem('bad_response', { redirect: 'true', host: 'h.exemplo.dev' }),
      );
      expect(text.body).toContain('tentou redirecionar a consulta');
      expect(text.body).not.toContain('h.exemplo.dev');
    });

    it('ignores a redirect flag that is not "true" or sits on another failure', () => {
      expect(describeError(problem('bad_response', { redirect: 'yes' })).body).not.toContain(
        'redirecionar',
      );
      expect(describeError(problem('timeout', { redirect: 'true' })).body).not.toContain(
        'redirecionar',
      );
    });

    it.each([
      ['30', '30 s'],
      ['60', '1 min'],
      ['90', '90 s'],
      ['3600', '1 h'],
    ])('tells how long the source asked to wait (%s s)', (retryAfterSecs, wait) => {
      const text = describeError(
        problem('rate_limited', { retryAfterSecs, httpStatus: '429' }, true),
      );
      expect(text.body).toContain(`Tempo pedido pela fonte: ${wait}.`);
      expect(text.body).toMatch(/ Código HTTP: 429\.$/);
    });

    it.each(['0', '-5', '1.5', '10000', 'soon', '', '1e3', ' 30'])(
      'does not word a wait of %j',
      (retryAfterSecs) => {
        const text = describeError(problem('rate_limited', { retryAfterSecs }, true));
        expect(text.body).not.toContain('Tempo pedido');
      },
    );

    it('only words a wait for a rate limit', () => {
      expect(describeError(problem('timeout', { retryAfterSecs: '30' }, true)).body).not.toContain(
        'Tempo pedido',
      );
    });

    it.each([
      ['value', 'chave de API'],
      ['baseUrl', 'URL do servidor'],
      ['entityId', 'ID do usuário ou do time'],
      ['authHeader', 'cabeçalho de autenticação'],
      ['pemPath', 'arquivo da CA (.pem)'],
      ['host', 'servidor sem verificação de certificado'],
      ['configDir', 'pasta de configuração do Claude Code'],
      ['binaryPath', 'caminho do executável do Codex'],
      ['dataDir', 'pasta de dados do OpenCode'],
      ['shimConsent', 'consentimento para instalar o comando'],
    ])('names the field %s that was refused', (field, label) => {
      expect(describeError(problem('invalid_input', { field })).body).toBe(
        `Corrija o campo indicado e tente de novo. Campo com problema: ${label}.`,
      );
    });

    it.each(['novo', 'constructor', '__proto__', 'x'.repeat(300), ''])(
      'does not name a field it does not know (%j)',
      (field) => {
        expect(describeError(problem('invalid_input', { field })).body).toBe(
          'Corrija o campo indicado e tente de novo.',
        );
      },
    );

    it('names a field only for invalid input', () => {
      expect(describeError(problem('network', { field: 'value' })).body).not.toContain('Campo');
    });

    it('keeps the HTTP status last, after every other detail', () => {
      const text = describeError(
        problem('rate_limited', { retryAfterSecs: '30', httpStatus: '429' }, true),
      );
      expect(text.body.endsWith('Código HTTP: 429.')).toBe(true);
      expect(text.body.indexOf('Tempo pedido')).toBeLessThan(text.body.indexOf('Código HTTP'));
    });
  });

  it('never prints the diagnostics only a bug report needs', () => {
    const text = describeError(
      problem('handshake_failed', {
        phase: 'initialize',
        rpcCode: '-32600',
        code: 'UNAVAILABLE',
        host: 'h',
      }),
    );
    expect(JSON.stringify(text)).not.toMatch(/initialize|32600|UNAVAILABLE/);
  });
});
