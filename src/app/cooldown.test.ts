import { describe, expect, it } from 'vitest';
import { viewOf } from '../mock/fixtures';
import { cooldownLeft, nextAttemptText, waitText } from './cooldown';

const NOW = 1_800_000_000;

describe('cooldownLeft', () => {
  it('is the time left while the mark is ahead', () => {
    expect(cooldownLeft(viewOf('claude', { nextAttemptAt: NOW + 240 }), NOW)).toBe(240);
  });

  it.each([
    ['no mark', null],
    ['a mark that has passed', NOW - 1],
    ['a mark that is exactly now', NOW],
  ])('is null for %s', (_name, nextAttemptAt) => {
    expect(cooldownLeft(viewOf('claude', { nextAttemptAt }), NOW)).toBeNull();
  });
});

describe('the wording', () => {
  it.each([
    [240, '4 min'],
    [241, '5 min'],
    [59, '59 s'],
    [60, '1 min'],
    [1800, '30 min'],
    [3600, '1 h'],
  ])('rounds %s s up to whole minutes: %s', (left, text) => {
    expect(waitText(left)).toBe(text);
  });

  it('says when the next attempt is, in Portuguese', () => {
    expect(nextAttemptText(240)).toBe(
      'A fonte pediu um intervalo entre as consultas. Nova tentativa em 4 min.',
    );
  });
});
