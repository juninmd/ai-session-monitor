import { describe, expect, it } from 'vitest';
import { CATALOGS } from './catalogs';
import { LOCALES } from './locale';

type Catalog = Readonly<Record<string, string>>;

const en = CATALOGS.en as Catalog;
const PLURAL = /\.(zero|one|two|few|many|other)$/;
const FORMS = ['zero', 'one', 'two', 'few', 'many', 'other'];
const placeholders = (text: string) => [...new Set(text.match(/\{\w+\}/g) ?? [])].sort();
const baseOf = (key: string) => key.replace(PLURAL, '');

/** English owns the list of messages; a plural message may use any CLDR form in another language. */
function expectedKeys(): { plain: string[]; plural: string[] } {
  const keys = Object.keys(en);
  return {
    plain: keys.filter((key) => !PLURAL.test(key)),
    plural: [...new Set(keys.filter((key) => PLURAL.test(key)).map(baseOf))],
  };
}

describe.each(LOCALES.filter((locale) => locale.id !== 'en').map((locale) => locale.id))(
  'catalog %s',
  (id) => {
    const catalog = (CATALOGS[id] ?? {}) as Catalog;

    it('is registered and translates every plain message', () => {
      expect(CATALOGS[id], 'locale has no catalog').toBeDefined();
      expect(expectedKeys().plain.filter((key) => !(key in catalog))).toEqual([]);
    });

    it('has at least the "other" form of every plural message', () => {
      expect(expectedKeys().plural.filter((base) => !(`${base}.other` in catalog))).toEqual([]);
    });

    it('invents no message English does not have', () => {
      const known = new Set(Object.keys(en).map(baseOf));
      const stray = Object.keys(catalog).filter(
        (key) => !(key in en) && !(PLURAL.test(key) && known.has(baseOf(key))),
      );
      expect(stray).toEqual([]);
    });

    it('keeps the placeholders of the English text, and nothing empty', () => {
      const wrong: string[] = [];
      for (const [key, text] of Object.entries(catalog)) {
        if (text.trim() === '') wrong.push(`${key}: empty`);
        const source = en[key] ?? en[`${baseOf(key)}.other`];
        if (source === undefined) continue;
        const allowed = new Set([
          ...placeholders(source),
          ...(PLURAL.test(key) ? ['{count}'] : []),
        ]);
        const used = placeholders(text);
        const missing = placeholders(source).filter((p) => !used.includes(p) && p !== '{count}');
        const extra = used.filter((p) => !allowed.has(p));
        if (missing.length > 0 || extra.length > 0) wrong.push(`${key}: ${missing}|${extra}`);
      }
      expect(wrong).toEqual([]);
    });

    it('uses only plural forms the language knows', () => {
      const rules = new Intl.PluralRules(id);
      const known = new Set(rules.resolvedOptions().pluralCategories);
      const stray = Object.keys(catalog).filter((key) => {
        const form = PLURAL.exec(key)?.[1];
        return form !== undefined && form !== 'other' && !known.has(form as Intl.LDMLPluralRule);
      });
      expect(stray, FORMS.join()).toEqual([]);
    });
  },
);

describe('English', () => {
  it('has the "other" form wherever it has any plural form', () => {
    const lacking = expectedKeys().plural.filter((base) => !(`${base}.other` in en));
    expect(lacking).toEqual([]);
  });
});
