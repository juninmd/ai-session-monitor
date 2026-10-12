// The screen check in emitted.test.tsx must go red when the catalog loses a label. Here it does:
// four labels are removed from the catalog before anything renders.
import { describe, expect, it, vi } from 'vitest';
import { describeError } from '../../app/errorText';
import { hasMessage } from '../../i18n';
import { litellmEmitted } from '../../mock/emitted';
import { problem } from '../../mock/fixtures';
import { ctxOf, envOf, renderPanel } from '../../test/helpers';
import { readable, shownWireWords } from '../../test/screen';
import { rawTokens } from '../../test/vocabulary';
import { HarnessPanel } from '../HarnessPanel';

const LOST = vi.hoisted(() => [
  'fact.expires',
  'meta.level.team',
  'notice.code.litellm_windows_truncated',
  'field.pemPath',
]);

vi.mock('../../i18n/pt-BR', async (importOriginal) => {
  const real = await importOriginal<typeof import('../../i18n/pt-BR')>();
  const kept = Object.entries(real.ptBR).filter(([key]) => !LOST.includes(key));
  return { ...real, ptBR: Object.fromEntries(kept) };
});

describe('a catalog that lost labels', () => {
  const view = litellmEmitted(envOf());

  it('really lacks them', () => {
    for (const key of LOST) expect(hasMessage(key)).toBe(false);
  });

  it('puts the ids behind them on screen, where the check reports them', () => {
    renderPanel(<HarnessPanel view={view} ctx={ctxOf()} />);
    const text = readable(document.body);
    expect(shownWireWords(view, text)).toEqual(
      expect.arrayContaining(['expires', 'level', 'litellm_windows_truncated']),
    );
    expect(rawTokens(text)).toContain('litellm_windows_truncated');
  });

  it('still never prints a catalog key', () => {
    renderPanel(<HarnessPanel view={view} ctx={ctxOf()} />);
    expect(readable(document.body)).not.toMatch(
      /\b(?:fact|facts|meta|notice|field|window|error)\.[a-z]/,
    );
  });

  it('names no field it has no word for', () => {
    const { body } = describeError(problem('invalid_input', { field: 'pemPath' }));
    expect(body).not.toContain('pemPath');
    expect(body).not.toContain('Campo com problema');
  });
});
