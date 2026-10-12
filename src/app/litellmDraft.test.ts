import { describe, expect, it } from 'vitest';
import { defaultSettings, type LitellmSettings } from '../lib/contract';
import {
  checkBaseUrl,
  draftFieldOf,
  FIELD_ORDER,
  fromDraft,
  hostOf,
  isLoopbackHost,
  type LitellmDraft,
  toDraft,
  validateDraft,
} from './litellmDraft';

const saved: LitellmSettings = defaultSettings.litellm;
const draftOf = (change: Partial<LitellmDraft> = {}): LitellmDraft => ({
  ...toDraft(saved),
  baseUrl: 'https://gateway.exemplo.dev',
  ...change,
});
const NO_KEY = { saved: false, typed: false };
const TYPED_KEY = { saved: false, typed: true };

describe('isLoopbackHost', () => {
  it.each([
    ['localhost', true],
    ['app.localhost', true],
    ['127.0.0.1', true],
    ['127.12.0.9', true],
    ['[::1]', true],
    ['notlocalhost', false],
    ['localhost.exemplo.dev', false],
    ['127.0.0.1.exemplo.dev', false],
    ['128.0.0.1', false],
    ['10.0.0.5', false],
    ['gateway.exemplo.dev', false],
  ])('%s is loopback: %s', (host, expected) => {
    expect(isLoopbackHost(host)).toBe(expected);
  });
});

describe('checkBaseUrl', () => {
  it.each([
    ['https://gateway.exemplo.dev'],
    ['https://gateway.exemplo.dev:8443/proxy'],
    ['http://localhost:4000'],
    ['http://127.0.0.1:4000'],
    ['http://[::1]:4000'],
    ['  https://gateway.exemplo.dev  '],
  ])('accepts %j', (url) => {
    expect(checkBaseUrl(url)).toBeNull();
  });

  it.each([
    ['', 'Informe a URL do servidor.'],
    ['   ', 'Informe a URL do servidor.'],
    ['gateway.exemplo.dev', 'Informe uma URL completa, como https://litellm.suaempresa.com.'],
    ['not a url', 'Informe uma URL completa, como https://litellm.suaempresa.com.'],
    ['ftp://gateway.exemplo.dev', 'Informe uma URL completa, como https://litellm.suaempresa.com.'],
    ['javascript:alert(1)', 'Informe uma URL completa, como https://litellm.suaempresa.com.'],
    ['http://gateway.exemplo.dev', 'Use https://. HTTP só funciona para localhost.'],
    ['http://127.0.0.1.exemplo.dev', 'Use https://. HTTP só funciona para localhost.'],
    ['https://user:pass@gateway.exemplo.dev', 'Remova usuário, senha, parâmetros e # da URL.'],
    ['https://gateway.exemplo.dev?x=1', 'Remova usuário, senha, parâmetros e # da URL.'],
    ['https://gateway.exemplo.dev#frag', 'Remova usuário, senha, parâmetros e # da URL.'],
  ])('rejects %j with a reason', (url, message) => {
    expect(checkBaseUrl(url)).toBe(message);
  });
});

describe('validateDraft', () => {
  it('accepts a complete draft', () => {
    expect(validateDraft(draftOf(), TYPED_KEY)).toEqual({});
  });

  it('asks for a key only when none is saved or typed', () => {
    expect(validateDraft(draftOf(), NO_KEY).key).toBe('Cole a chave de API do LiteLLM.');
    expect(validateDraft(draftOf(), { saved: true, typed: false })).toEqual({});
  });

  it('asks for an id when the scope is a user or a team', () => {
    expect(validateDraft(draftOf({ scope: 'team' }), TYPED_KEY).entityId).toBe('Informe o ID.');
    expect(validateDraft(draftOf({ scope: 'user', entityId: '  ' }), TYPED_KEY).entityId).toBe(
      'Informe o ID.',
    );
    expect(validateDraft(draftOf({ scope: 'user', entityId: 'u-1' }), TYPED_KEY)).toEqual({});
    expect(validateDraft(draftOf({ scope: 'key' }), TYPED_KEY)).toEqual({});
  });

  it.each([
    ['X-Api-Key', false],
    ['Authorization', false],
    ['', true],
    ['Bad Header', true],
    ['Bad:Header', true],
    ['Cabeçalho', true],
  ])('header name %j is invalid: %s', (authHeader, invalid) => {
    expect(validateDraft(draftOf({ authHeader }), TYPED_KEY).authHeader !== undefined).toBe(
      invalid,
    );
  });

  it('asks for the CA file when a custom CA is chosen', () => {
    const errors = validateDraft(draftOf({ tlsMode: 'custom_ca', pemPath: ' ' }), TYPED_KEY);
    expect(errors.pemPath).toBe('Informe o caminho do arquivo .pem.');
  });

  it('requires a host and an explicit confirmation to skip certificate checks', () => {
    const bare = validateDraft(draftOf({ tlsMode: 'insecure' }), TYPED_KEY);
    expect(bare.host).toBe('Informe o servidor.');
    expect(bare.confirm).toBe('Confirme para continuar sem verificar o certificado.');
    const ready = draftOf({
      tlsMode: 'insecure',
      insecureHost: 'gateway.exemplo.dev',
      insecureConfirmed: true,
    });
    expect(validateDraft(ready, TYPED_KEY)).toEqual({});
  });

  it('reports every problem at once, in an order that matches the form', () => {
    const errors = validateDraft(
      draftOf({ baseUrl: '', scope: 'team', authHeader: '!', tlsMode: 'custom_ca' }),
      NO_KEY,
    );
    const reported = FIELD_ORDER.filter((field) => errors[field] !== undefined);
    expect(reported).toEqual(['baseUrl', 'key', 'entityId', 'authHeader', 'pemPath']);
  });
});

describe('draft <-> settings', () => {
  it('round-trips the defaults', () => {
    expect(fromDraft(saved, toDraft(saved))).toEqual(saved);
  });

  it('trims text and turns blanks into null', () => {
    const next = fromDraft(
      saved,
      draftOf({ baseUrl: '  https://gateway.exemplo.dev  ', authHeader: ' X-Key ' }),
    );
    expect(next.baseUrl).toBe('https://gateway.exemplo.dev');
    expect(next.authHeader).toBe('X-Key');
    expect(fromDraft(saved, draftOf({ baseUrl: '  ' })).baseUrl).toBeNull();
  });

  it('forgets the id when the scope goes back to this key', () => {
    const team = fromDraft(saved, draftOf({ scope: 'team', entityId: ' team-7 ' }));
    expect(team).toMatchObject({ scope: 'team', entityId: 'team-7' });
    expect(fromDraft(team, draftOf({ scope: 'key', entityId: 'team-7' })).entityId).toBeNull();
  });

  it.each([
    [{ mode: 'system' }],
    [{ mode: 'custom_ca', pemPath: 'C:\\certs\\ca.pem' }],
    [{ mode: 'insecure', host: 'gateway.exemplo.dev' }],
  ] as const)('keeps tls %j through a draft', (tls) => {
    const withTls: LitellmSettings = { ...saved, tls };
    expect(fromDraft(withTls, toDraft(withTls)).tls).toEqual(tls);
  });

  it('treats a saved insecure host as already confirmed', () => {
    const insecure: LitellmSettings = {
      ...saved,
      tls: { mode: 'insecure', host: 'h.exemplo.dev' },
    };
    expect(toDraft(insecure).insecureConfirmed).toBe(true);
    expect(toDraft(saved).insecureConfirmed).toBe(false);
  });

  it('drops the fields of the tls modes that are not selected', () => {
    const next = fromDraft(
      saved,
      draftOf({ tlsMode: 'system', pemPath: 'C:\\old.pem', insecureHost: 'old.exemplo.dev' }),
    );
    expect(next.tls).toEqual({ mode: 'system' });
  });
});

describe('field mapping', () => {
  it.each([
    ['baseUrl', 'baseUrl'],
    ['value', 'key'],
    ['entityId', 'entityId'],
    ['authHeader', 'authHeader'],
    ['pemPath', 'pemPath'],
    ['host', 'host'],
    ['somethingNew', 'form'],
    ['constructor', 'form'],
    [null, 'form'],
  ])('backend field %s lands on %s', (backend, field) => {
    expect(draftFieldOf(backend)).toBe(field);
  });

  it('extracts the host of a URL, or nothing', () => {
    expect(hostOf(' https://gateway.exemplo.dev:8443/x ')).toBe('gateway.exemplo.dev');
    expect(hostOf('not a url')).toBe('');
    expect(hostOf('')).toBe('');
  });
});
