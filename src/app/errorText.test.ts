import { describe, expect, it } from 'vitest';
import { ERROR_CODES, type HarnessError } from '../lib/contract';
import { problem } from '../mock/fixtures';
import { describeError, type ErrorAction, fieldOf, isAmbiguousAuth } from './errorText';
import { fieldMessage } from './fieldErrors';

const ACTIONS: readonly ErrorAction[] = ['retry', 'settings', 'console', 'reinstall'];

describe('describeError', () => {
  it.each(ERROR_CODES)('gives %s a pt-BR message and exactly one known action', (code) => {
    const text = describeError(problem(code));
    expect(text.title.length).toBeGreaterThan(0);
    expect(text.body.length).toBeGreaterThan(0);
    expect(ACTIONS).toContain(text.action);
    // The machine code is for logs, not for people.
    expect(`${text.title} ${text.body}`).not.toContain(code);
  });

  it.each([
    ['timeout', 'retry'],
    ['network', 'retry'],
    ['rate_limited', 'retry'],
    ['unauthorized', 'settings'],
    ['tls_error', 'settings'],
    ['invalid_url', 'settings'],
    ['binary_not_found', 'settings'],
    ['install_failed', 'reinstall'],
  ] as const)('maps %s to the %s action', (code, action) => {
    expect(describeError(problem(code)).action).toBe(action);
  });

  it('appends a numeric HTTP status', () => {
    expect(describeError(problem('server_error', { httpStatus: '503' })).body).toMatch(
      / Código HTTP: 503\.$/,
    );
  });

  it.each([['<img src=x>'], ['5030'], ['50'], ['abc'], [''], ['503 '], ['x'.repeat(500)]])(
    'does not print an HTTP status that is not three digits (%j)',
    (httpStatus) => {
      const text = describeError(problem('server_error', { httpStatus }));
      expect(text.body).not.toContain('Código HTTP');
      expect(text.body).not.toContain(httpStatus.trim() || '\u0000');
    },
  );

  it('never prints other params', () => {
    const text = describeError(
      problem('network', { host: 'gateway.exemplo.dev', token: 's3cret' }),
    );
    expect(JSON.stringify(text)).not.toMatch(/gateway\.exemplo\.dev|s3cret/);
  });

  it('falls back to the internal message for a code the catalog does not know', () => {
    const unknown = {
      code: 'from_the_future',
      retryable: false,
      params: {},
    } as unknown as HarnessError;
    const text = describeError(unknown);
    expect(text.title).toBe('Algo deu errado');
    expect(text.body).toContain('erro interno');
    expect(text.action).toBe('retry');
  });
});

describe('OpenCode authentication errors', () => {
  const rejected = problem('unauthorized', { httpStatus: '403', ambiguous: 'true' });

  it('say the key or the Go subscription could not be verified, and offer the console', () => {
    const text = describeError(rejected, 'opencode');
    expect(text.title).toBe('Não foi possível verificar a chave ou a assinatura Go');
    expect(text.body).toContain('console do OpenCode');
    expect(text.body).toContain('Código HTTP: 403.');
    expect(text.action).toBe('console');
  });

  it('never claim the key is certainly wrong', () => {
    const text = describeError(rejected, 'opencode');
    expect(`${text.title} ${text.body}`).not.toMatch(/inválid|incorret|errad|recusado/i);
  });

  it('treat the harness itself as ambiguous even without the flag', () => {
    expect(isAmbiguousAuth(problem('unauthorized'), 'opencode')).toBe(true);
    expect(describeError(problem('unauthorized'), 'opencode').action).toBe('console');
  });

  it('honour the flag on any harness', () => {
    expect(describeError(rejected, 'litellm').action).toBe('console');
  });

  it('leave other sources with the plain wording and the settings action', () => {
    const text = describeError(problem('unauthorized', { httpStatus: '401' }), 'litellm');
    expect(text.title).toBe('Chave ou login recusado');
    expect(text.action).toBe('settings');
    expect(isAmbiguousAuth(problem('network'), 'opencode')).toBe(false);
  });
});

describe('fieldOf', () => {
  it.each([
    [problem('invalid_url'), 'baseUrl'],
    [problem('invalid_url', { field: 'baseUrl' }), 'baseUrl'],
    [problem('invalid_input', { field: 'authHeader' }), 'authHeader'],
    [problem('invalid_input'), null],
    [problem('network', { field: 'baseUrl' }), null],
  ])('%j belongs to %s', (error, field) => {
    expect(fieldOf(error)).toBe(field);
  });
});

describe('fieldMessage', () => {
  it.each([
    ['value', 'Cole a chave de API.'],
    ['entityId', 'Informe o ID.'],
    ['authHeader', 'Use só letras, números e hífen.'],
    ['pemPath', 'Informe o caminho do arquivo .pem.'],
    ['host', 'Informe o servidor.'],
  ])('words an invalid %s next to its field', (field, message) => {
    expect(fieldMessage(problem('invalid_input', { field }), field)).toBe(message);
  });

  it('explains the URL rules for a rejected URL', () => {
    expect(fieldMessage(problem('invalid_url', { field: 'baseUrl' }), 'baseUrl')).toContain(
      'Use https://',
    );
  });

  it('ignores inherited object keys sent as a field name', () => {
    const error = problem('invalid_input', { field: 'constructor' });
    expect(fieldMessage(error, 'constructor')).toBe(
      'Valor inválido. Corrija o campo indicado e tente de novo.',
    );
  });

  it('falls back to title and body when the field has no specific wording', () => {
    expect(fieldMessage(problem('invalid_input', { field: 'novo' }), 'novo')).toBe(
      'Valor inválido. Corrija o campo indicado e tente de novo.',
    );
    expect(fieldMessage(problem('timeout'), null)).toBe(
      'A leitura demorou demais. A fonte não respondeu a tempo. Tente de novo em instantes.',
    );
  });
});
