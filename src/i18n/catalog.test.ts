import { describe, expect, it } from 'vitest';
import ipcDoc from '../../docs/ipc.md?raw';
import {
  ERROR_CODES,
  HARNESS_IDS,
  LITELLM_SCOPES,
  SEVERITIES,
  SHIM_STATES,
  WINDOW_KINDS,
} from '../lib/contract';
import { hasMessage, t } from './index';
import { partials, ptBR } from './pt-BR';

/** Rows of the "Notice codes" table in docs/ipc.md: the contract the catalog must cover. */
function documentedNoticeCodes(): string[] {
  const section = ipcDoc.split('### Notice codes')[1]?.split(/\n## /)[0] ?? '';
  return [...section.matchAll(/^\| `([a-z0-9_]+)` \| (?:info|warn|error) \|/gm)].map(
    (match) => match[1] ?? '',
  );
}

describe('catalog completeness', () => {
  it.each(ERROR_CODES)('has a title and a body for error code %s', (code) => {
    expect(hasMessage(`error.${code}.title`)).toBe(true);
    expect(hasMessage(`error.${code}.body`)).toBe(true);
    expect(t(`error.${code}.title`).length).toBeGreaterThan(0);
    expect(t(`error.${code}.body`).length).toBeGreaterThan(0);
  });

  it('reads the notice table of docs/ipc.md', () => {
    const codes = documentedNoticeCodes();
    expect(codes.length).toBeGreaterThanOrEqual(13);
    expect(new Set(codes).size).toBe(codes.length);
  });

  it.each(documentedNoticeCodes())('has a sentence for notice code %s', (code) => {
    expect(hasMessage(`notice.code.${code}`)).toBe(true);
  });

  it.each(HARNESS_IDS)('names harness %s', (id) => {
    expect(hasMessage(`harness.${id}`)).toBe(true);
  });

  it.each(WINDOW_KINDS)('titles window kind %s in full and as a row', (kind) => {
    expect(hasMessage(`window.${kind}`)).toBe(true);
    expect(hasMessage(`window.row.${kind}`)).toBe(true);
  });

  it.each(SHIM_STATES)('describes shim state %s', (state) => {
    expect(hasMessage(`shim.state.${state}`)).toBe(true);
  });

  it.each(LITELLM_SCOPES)('labels litellm scope %s in the form and in the chip', (scope) => {
    expect(hasMessage(`litellm.scope.${scope}`)).toBe(true);
    expect(hasMessage(`meta.scope.${scope}`)).toBe(true);
  });

  it.each(SEVERITIES)('labels notice severity %s', (severity) => {
    expect(hasMessage(`notice.severity.${severity}`)).toBe(true);
  });

  it.each(['ok', 'warn', 'crit', 'out'])('words status %s', (status) => {
    expect(hasMessage(`status.${status}`)).toBe(true);
  });

  it.each(['retry', 'settings', 'console', 'reinstall'])('labels action %s', (action) => {
    expect(hasMessage(`action.${action}`)).toBe(true);
  });
});

describe('catalog hygiene', () => {
  it('defines every key once across the partial files', () => {
    const owner = new Map<string, number>();
    const duplicates: string[] = [];
    partials.forEach((partial, index) => {
      for (const key of Object.keys(partial)) {
        if (owner.has(key)) duplicates.push(key);
        owner.set(key, index);
      }
    });
    expect(duplicates).toEqual([]);
  });

  it('has no empty message', () => {
    const empty = Object.entries(ptBR).filter(([, message]) => message.trim() === '');
    expect(empty).toEqual([]);
  });

  it('only uses {name} placeholders the formatter understands', () => {
    // A stray "{" would survive t() and reach the screen.
    for (const [key, message] of Object.entries(ptBR)) {
      const stripped = message.replace(/\{\w+\}/g, '');
      expect(stripped, key).not.toMatch(/[{}]/);
    }
  });
});

describe('t', () => {
  it('fills placeholders', () => {
    expect(t('footer.read_ago', { age: '4 min' })).toBe('Lido há 4 min');
    expect(t('live.warn', { name: 'Codex', pct: '18%' })).toBe('Codex: atenção, 18% disponível.');
  });

  it('prints a dash for a missing parameter instead of a raw placeholder', () => {
    expect(t('footer.read_ago')).toBe('Lido há —');
    expect(t('footer.read_ago', {})).not.toContain('{');
  });

  it('accepts numbers as parameters', () => {
    expect(t('error.http', { httpStatus: 429 })).toBe('Código HTTP: 429.');
  });

  it('never fills in more than the template asks for', () => {
    expect(t('footer.refresh', { age: 'ignored' })).toBe('Atualizar');
  });
});

describe('hasMessage', () => {
  it('is false for unknown and inherited keys', () => {
    expect(hasMessage('notice.code.never_heard_of_it')).toBe(false);
    expect(hasMessage('toString')).toBe(false);
    expect(hasMessage('__proto__')).toBe(false);
    expect(hasMessage('constructor')).toBe(false);
  });
});
