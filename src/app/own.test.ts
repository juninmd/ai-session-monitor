import { describe, expect, it } from 'vitest';
import { own } from './own';

describe('own', () => {
  const table = { a: 1, b: 2 };

  it('finds its own keys', () => {
    expect(own(table, 'a')).toBe(1);
  });

  it.each(['c', 'constructor', 'toString', '__proto__', 'hasOwnProperty', '', null, undefined])(
    'does not invent a value for %j',
    (key) => {
      expect(own(table, key)).toBeUndefined();
    },
  );
});
