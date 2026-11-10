import { catalogOf, ENGLISH, type MessageKey } from './catalogs';
import { getLocale } from './locale';

export type { MessageKey } from './catalogs';

export type MessageParams = Readonly<Record<string, string | number>>;

// A placeholder without a parameter renders as a dash: never a raw "{name}" in the UI.
const MISSING = '—';
const PLACEHOLDER = /\{(\w+)\}/g;

/** Looks up a message in the active language (English when it lacks the key) and fills `{name}`. */
export function t(key: MessageKey, params?: MessageParams): string {
  const template: string = catalogOf(getLocale())?.[key] ?? ENGLISH[key] ?? '';
  return template.replace(PLACEHOLDER, (_match, name: string) => {
    const value = params?.[name];
    return value === undefined ? MISSING : String(value);
  });
}

/**
 * A message that depends on a count. The catalog holds one key per plural category the language
 * uses (`key.one`, `key.few`, `key.other`, ...); `key.other` must always exist. `{count}` is filled.
 */
export function tn(base: string, count: number, params?: MessageParams): string {
  const category = pluralRules(getLocale()).select(count);
  const key = `${base}.${category}`;
  const picked = isDefined(key) ? key : `${base}.other`;
  return t(picked as MessageKey, { ...params, count });
}

// One Intl.PluralRules per locale: constructing one per message showed up in list renders.
const rules = new Map<string, Intl.PluralRules>();
function pluralRules(locale: string): Intl.PluralRules {
  let cached = rules.get(locale);
  if (cached === undefined) {
    cached = new Intl.PluralRules(locale);
    rules.set(locale, cached);
  }
  return cached;
}

const isDefined = (key: string): boolean =>
  Object.hasOwn(catalogOf(getLocale()) ?? {}, key) || Object.hasOwn(ENGLISH, key);

export function hasMessage(key: string): key is MessageKey {
  return Object.hasOwn(ENGLISH, key);
}
