import { describe, expect, it } from 'vitest';
import ipcDoc from '../../docs/ipc.md?raw';
import { notice } from '../mock/fixtures';
import { noticeAction, noticeKey, noticeText, uniqueNotices } from './noticeText';

const documented = [...ipcDoc.matchAll(/^\| `([a-z0-9_]+)` \| (info|warn|error) \|/gm)].map(
  (match) => match[1] ?? '',
);

describe('noticeText', () => {
  it.each(documented)('translates the documented code %s', (code) => {
    const text = noticeText(notice(code, 'info'));
    expect(text).not.toContain('Aviso sem tradução');
    expect(text.length).toBeGreaterThan(10);
  });

  it('fills a window param with words', () => {
    expect(noticeText(notice('opencode_rate_limited', 'warn', { window: 'rolling' }))).toBe(
      'A janela de 5 horas está limitada (rate-limited).',
    );
    expect(noticeText(notice('opencode_window_idle', 'info', { window: 'monthly' }))).toContain(
      'mensal',
    );
  });

  it('shows an unknown window id as it came', () => {
    expect(noticeText(notice('opencode_rate_limited', 'warn', { window: 'hourly' }))).toBe(
      'A janela hourly está limitada (rate-limited).',
    );
  });

  it('names a pool with its display name', () => {
    expect(noticeText(notice('antigravity_pool_unknown', 'warn', { pool: 'claude_gpt' }))).toBe(
      'A cota Claude e GPT-OSS não retornou dados. Ela aparece como desconhecida, não como zero.',
    );
  });

  it('turns a failure reason into its title', () => {
    expect(noticeText(notice('claude_oauth_partial', 'info', { reason: 'timeout' }))).toContain(
      'falhou (A leitura demorou demais)',
    );
  });

  it('shows an unknown reason as it came', () => {
    expect(noticeText(notice('claude_oauth_partial', 'info', { reason: 'solar_flare' }))).toContain(
      'falhou (solar_flare)',
    );
  });

  it('renders an unknown code visibly, with the code in the message', () => {
    expect(noticeText(notice('some_future_code', 'warn'))).toBe(
      'Aviso sem tradução: some_future_code',
    );
  });

  it('does not trust the length of a code or a param', () => {
    const long = 'x'.repeat(400);
    const unknown = noticeText(notice(long, 'warn'));
    expect(unknown.length).toBeLessThan(100);
    expect(unknown).toContain('…');
    const known = noticeText(notice('antigravity_pool_unknown', 'warn', { pool: long }));
    expect(known.length).toBeLessThan(200);
  });

  it('does not look up inherited object keys as codes', () => {
    expect(noticeText(notice('__proto__', 'info'))).toBe('Aviso sem tradução: __proto__');
    expect(noticeText(notice('constructor', 'info'))).toBe('Aviso sem tradução: constructor');
  });
});

describe('noticeText with the params the collectors send', () => {
  const unreadable = (params: Record<string, string>) =>
    noticeText(notice('litellm_level_unreadable', 'warn', params));
  const truncated = (dropped: string) =>
    noticeText(notice('litellm_windows_truncated', 'info', { dropped }));
  const dbUnreadable = (reason: string) =>
    noticeText(notice('opencode_db_unreadable', 'warn', { reason }));

  it.each([
    ['key', 'da chave'],
    ['user', 'do usuário'],
    ['team', 'do time'],
  ])('says which budget it could not read for level %s', (level, words) => {
    expect(unreadable({ level, reason: 'timeout' })).toBe(
      `Não foi possível ler o orçamento ${words} (A leitura demorou demais). Pode haver um teto ali que não aparece aqui.`,
    );
  });

  it('turns the failure behind an unreadable level into words, and shows a stranger as it came', () => {
    expect(unreadable({ level: 'team', reason: 'unauthorized' })).toContain(
      '(Chave ou login recusado)',
    );
    expect(unreadable({ level: 'org', reason: 'solar_flare' })).toContain(
      'orçamento org (solar_flare)',
    );
  });

  it.each([
    ['1', '1 janela'],
    ['7', '7 janelas'],
    ['16', '16 janelas'],
    ['1234', '1.234 janelas'],
  ])('counts %s dropped window(s) as "%s"', (dropped, words) => {
    expect(truncated(dropped)).toBe(
      `Há mais janelas de orçamento do que cabem aqui. Ficaram de fora as menos consumidas: ${words}.`,
    );
  });

  it.each(['many', '-3', '1.5', '1e3', '12345678', '0x10'])(
    'does not format a dropped value that is not a plain count (%j)',
    (dropped) => {
      const text = truncated(dropped);
      expect(text).not.toMatch(/janelas?\.$/);
      expect(text).toContain(dropped);
    },
  );

  it('says why the Codex app-server was skipped', () => {
    expect(
      noticeText(notice('codex_rollout_fallback', 'info', { reason: 'binary_not_found' })),
    ).toBe(
      'O codex app-server não respondeu (Programa não encontrado). Lendo a última sessão gravada; os números podem estar antigos.',
    );
  });

  it.each([
    ['invalid_path', 'o caminho da pasta de dados é inválido'],
    ['not_found', 'o arquivo do banco não foi encontrado'],
    ['permission', 'sem permissão para abrir o arquivo do banco'],
    ['locked', 'o banco está em uso por outro programa'],
    ['not_a_database', 'o arquivo encontrado não é um banco de dados'],
    ['schema', 'o formato do banco mudou'],
    ['timeout', 'a leitura do banco demorou demais'],
    ['other', 'motivo não identificado'],
  ])('words the OpenCode database reason %s', (reason, words) => {
    expect(dbUnreadable(reason)).toBe(`Não foi possível ler o banco local do OpenCode: ${words}.`);
  });

  it('does not read a database reason as the error code that shares its name', () => {
    // `not_found` is also an ErrorCode ("Endereço não encontrado"): wrong for a missing file.
    expect(dbUnreadable('not_found')).not.toContain('Endereço');
    expect(dbUnreadable('timeout')).not.toContain('A leitura demorou demais');
  });

  it('shows a database reason it does not know as it came, clipped', () => {
    expect(dbUnreadable('solar_flare')).toContain('OpenCode: solar_flare.');
    expect(dbUnreadable('x'.repeat(400)).length).toBeLessThan(200);
  });

  it('never leaves a placeholder or a catalog key on the screen', () => {
    const samples = [
      unreadable({ level: 'user', reason: 'network' }),
      truncated('3'),
      dbUnreadable('locked'),
      noticeText(notice('codex_rollout_fallback', 'info', { reason: 'timeout' })),
    ];
    for (const text of samples) expect(text).not.toMatch(/[{}]|\b(?:notice|error)\.[a-z_.]+/);
  });
});

describe('noticeAction', () => {
  it.each([
    ['claude_shim_outdated', 'reinstall'],
    ['opencode_no_key', 'settings'],
    ['codex_no_limits', null],
    ['brand_new', null],
    ['constructor', null],
    ['toString', null],
  ])('%s offers %s', (code, action) => {
    expect(noticeAction(notice(code, 'info'))).toBe(action);
  });
});

describe('uniqueNotices', () => {
  it('collapses identical notices and keeps the first', () => {
    const a = notice('codex_no_limits', 'warn');
    expect(uniqueNotices([a, { ...a }, a])).toEqual([a]);
  });

  it('keeps notices that differ in a param, whatever the param order', () => {
    const one = notice('opencode_rate_limited', 'warn', { window: 'rolling' });
    const two = notice('opencode_rate_limited', 'warn', { window: 'weekly' });
    expect(uniqueNotices([one, two])).toHaveLength(2);
    const ab = notice('x', 'info', { a: '1', b: '2' });
    const ba = notice('x', 'info', { b: '2', a: '1' });
    expect(noticeKey(ab)).toBe(noticeKey(ba));
    expect(uniqueNotices([ab, ba])).toHaveLength(1);
  });
});
