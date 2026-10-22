// Shared Intl plumbing for the formatters: the locale tag with Latin digits, and a per-locale cache.
import { getLocale } from './locale';

/** The BCP 47 tag Intl gets: the active language with Latin digits (Arabic, Urdu, Bengali, Hindi). */
export function intlTag(): string {
  const locale = getLocale();
  return locale === 'zh-CN' ? locale : `${locale}-u-nu-latn`;
}

const cache = new Map<string, unknown>();

/** One Intl object per locale and kind: building them is expensive, the language rarely changes. */
export function cached<T>(kind: string, make: (tag: string) => T): T {
  const tag = intlTag();
  const key = `${kind}|${tag}`;
  let found = cache.get(key) as T | undefined;
  if (found === undefined) {
    found = make(tag);
    cache.set(key, found);
  }
  return found;
}

/** The locale's decimal separator ("," in pt-BR, "." in English). */
export function decimalSeparator(): string {
  return cached(
    'decimal',
    (tag) =>
      new Intl.NumberFormat(tag).formatToParts(1.1).find((part) => part.type === 'decimal')
        ?.value ?? '.',
  );
}
