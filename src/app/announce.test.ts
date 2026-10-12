import { describe, expect, it } from 'vitest';
import { claudeView, codexView } from '../mock/builders';
import { asStale, failed, problem, viewOf } from '../mock/fixtures';
import { ctxOf, envOf } from '../test/helpers';
import { announcementFor, type StatusKey, statusKeyOf } from './announce';
import { summaryOf } from './tabModel';

const ctx = ctxOf();
const env = envOf();
const keyOf = (view: Parameters<typeof statusKeyOf>[0]) => statusKeyOf(view, summaryOf(view, ctx));

describe('statusKeyOf', () => {
  it('classifies a view by what the user would want to hear about it', () => {
    expect(keyOf(claudeView(env, 'ok'))).toBe('ok');
    expect(keyOf(codexView(env, 'warn'))).toBe('warn');
    expect(keyOf(claudeView(env, 'crit'))).toBe('out');
    expect(keyOf(failed('codex', problem('network')))).toBe('error');
    expect(keyOf(asStale(codexView(env, 'ok'), env, 30))).toBe('stale');
    expect(keyOf(viewOf('codex', { state: 'loading' }))).toBe('other');
    expect(keyOf(viewOf('codex', { state: 'needs_consent' }))).toBe('other');
  });
});

describe('announcementFor', () => {
  const summary = summaryOf(codexView(env, 'warn'), ctx);

  it('stays silent for the first reading and for no change', () => {
    expect(announcementFor('codex', undefined, 'crit', summary)).toBeNull();
    expect(announcementFor('codex', 'warn', 'warn', summary)).toBeNull();
  });

  it.each([
    ['ok', 'warn', 'Codex: atenção, 18% disponível.'],
    ['warn', 'crit', 'Codex: crítico, 18% disponível.'],
    ['crit', 'out', 'Codex: esgotado.'],
    ['ok', 'error', 'Codex: falha na leitura.'],
    ['ok', 'stale', 'Codex: dados desatualizados.'],
  ] as const)('speaks the move from %s to %s', (previous, next, sentence) => {
    expect(announcementFor('codex', previous, next, summary)).toBe(sentence);
  });

  it.each(['warn', 'crit', 'out', 'error', 'stale'] as StatusKey[])(
    'tells the user when %s clears up',
    (previous) => {
      expect(announcementFor('codex', previous, 'ok', summary)).toBe(
        'Codex: 18% disponível, sem alertas.',
      );
    },
  );

  it('does not celebrate a source that was never in trouble', () => {
    expect(announcementFor('codex', 'other', 'ok', summary)).toBeNull();
  });

  it('does not speak when a source simply stops being readable', () => {
    expect(announcementFor('codex', 'ok', 'other', null)).toBeNull();
  });
});
