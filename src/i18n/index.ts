import { CATALOGS, type MessageKey } from './catalogs';
import { DEFAULT_LOCALE, getLocale } from './locale';

export type { MessageKey } from './catalogs';

export type MessageParams = Readonly<Record<string, string | number>>;

// A placeholder without a parameter renders as a dash: never a raw "{name}" in the UI.
const MISSING = '—';
const PLACEHOLDER = /\{(\w+)\}/g;

/** Looks up a message in the active language (English when it lacks the key) and fills `{name}`. */
export function t(key: MessageKey, params?: MessageParams): string {
  const template: string =
    CATALOGS[getLocale()]?.[key] ??
    CATALOGS[DEFAULT_LOCALE]?.[key] ??
    CATALOGS['pt-BR']?.[key] ??
    '';
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
  const category = new Intl.PluralRules(getLocale()).select(count);
  const key = `${base}.${category}`;
  const picked = isDefined(key) ? key : `${base}.other`;
  return t(picked as MessageKey, { ...params, count });
}

const isDefined = (key: string): boolean =>
  Object.hasOwn(CATALOGS[getLocale()] ?? {}, key) ||
  Object.hasOwn(CATALOGS[DEFAULT_LOCALE] ?? {}, key);

export function hasMessage(key: string): key is MessageKey {
  return Object.hasOwn(CATALOGS['pt-BR'] ?? {}, key);
}
